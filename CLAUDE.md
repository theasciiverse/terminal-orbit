# terminal-orbit

A Rust TUI conversion of ORBIT, a 1999 OpenGL space combat simulator by Steve Belczyk.

## What this is

The original C source lives at `../../reference/space-orbit/` (~15,348 lines, 34 files). This project replaces all OpenGL rendering with a terminal UI: a 3D braille-rendered viewport via the `drawille` crate, and HUD panels via `ratatui`. Physics, AI, mission logic, and vector math are ported directly from C.

**Why:** Preserve and modernize a real-physics educational game (Newtonian flight, real solar system, campaigns from Mars to Pluto) without requiring a 1999 OpenGL binary.

## Stack

- **Rust** — OpenGL removal leaves nothing to salvage in the C rendering path
- **ratatui 0.29 + crossterm 0.28** — TUI windowing and HUD panels
- **drawille** (latest) — Unicode braille sub-pixel 3D viewport
- **serde / bincode** — save files and network packets
- **tokio** — Phase 10 multiplayer only
- Target: 30fps fixed-timestep game loop

## Key source files (original C)

| File | Lines | Role |
|---|---|---|
| `orbit.h` | ~755 | Master header — all structs, constants, array sizes. Primary reference for `types.rs` |
| `util.c` | ~255 | Vector math kernel — `RotateAbout` (Rodrigues), `Dotp`, `Crossp`, `Normalize`, etc. |
| `player.c` | ~300 | Newtonian and arcade flight physics, orientation via `RotateAbout` |
| `think.c` | ~405 | 8 AI strategies: DoNothing, Sit1–4, Hunt1–4; TurnToward, MoveToward, FindEnemy |
| `mission.c` | ~800 | Mission tokenizer + all keyword handlers; cursor relative/absolute system |
| `event.c` | ~200 | DoEvents trigger/action table |
| `hud.c` | ~750 | HUD rendering; `RadarCoords()` at lines 211–258 — port verbatim |
| `planet.c` | ~700 | Planet orbital mechanics and real solar system data |
| `orbit.c` | ~700 | Main entry point, DrawScene game loop, gravity |
| `ac3d.c` | ~700 | AC3D model loader |
| `model.c` | — | `.tri` model loader (9 floats + hex color per line, coords ÷100) |
| `stars.h` | — | Static C array of 2000 stars as [x, y, z, magnitude] unit vectors |

## Key constants (from `orbit.h`)

```
NPLANETS=32, NTARGETS=32, NMSLS=32, NBOOMS=32, NWAYPOINTS=32, NSTARS=2000
NWEAPONS=10, NCLIENTS=16
KM_TO_UNITS1=6000.0, KM_TO_UNITS2=6000.0/1e6
THETA=1.6          (rad/s rotation rate)
DELTAV=0.2         (km/s/s acceleration)
G=0.025, RMIN=2.0
BOOM_TIME=1.0s, CONSAGE=3.0s (console message fade), MAXDELTAT=0.1s
```

## Mission file format (`.msn`)

- Whitespace-tokenized, `/* */` comments, `Include filename` recursion
- Cursor system: bare value = absolute; `+value` = relative offset; planet name = planet position
- After `Cursor { Earth }`, position IS Earth's position; `{ Earth -15000 }` = Earth + offset
- `\\` in text = newline
- 52 mission files at `../../reference/space-orbit/missions/`

## 3D model formats

- `.tri`: rows of `x1 y1 z1  x2 y2 z2  x3 y3 z3  0xRRGGBB` — divide coords by 100
- `.ac`: AC3D format — MATERIAL/OBJECT/numvert/numsurf/SURF blocks, polygon soup
- Models at `../../reference/space-orbit/models/` — copied to `models/`
- Planet texture maps (PPM P6 256×256) at `../../reference/space-orbit/maps/` — copied to `maps/`

## Key bindings (current)

| Key | Action |
|-----|--------|
| W/S | Thrust forward/backward |
| A/D | Yaw left/right |
| I/K | Pitch up/down |
| J/L | Roll left/right |
| X | Toggle warp engines |
| H | Hold (station-keeping; StopNear docking needs zero velocity) |
| Space | Fire weapon |
| U | Lock nearest enemy |
| Y | Cycle targets |
| F | Cycle weapons |
| G | Toggle orbit autopilot (flies to standard orbit, 1.25 planet radii) |
| C | Toggle third-person camera; in orbit cycles shot Above, shot Below, cockpit |
| T | Toggle planet textures |
| B | Toggle constellation lines |
| N | Toggle names (planets, targets, constellation labels) |
| O | Toggle orrery |
| Z | Toggle dense stars (200/2000) |
| M | Recall/dismiss last mission message |
| P | Pause |
| Q | Quit to title |

## Port vs. rewrite

| Direct port from C | Rewrite fresh |
|---|---|
| `util.c` vector math | DrawScene → crossterm game loop |
| `player.c` Newtonian/arcade physics | All OpenGL rendering |
| `think.c` AI strategies | HUD layout → ratatui |
| `mission.c` tokenizer logic | Planet rendering → wireframe spheres + raycasted texture |
| `event.c` trigger/action table | `ac3d.c` → edge-only wireframe loader |
| `hud.c` RadarCoords() math | Network → tokio async |
| Solar system constants from `planet.c` | `keyboard.c` → crossterm |

**Skip entirely:** `lights.c`, `sound.c` (terminal bell only), `joystick.c`, `screenshot.c`

## Current module tree

```
terminal-orbit/
  Cargo.toml
  build.rs                    (parses stars.h → star_data.rs; Fibonacci fallback)
  missions/                   (copied + updated for terminal-orbit controls)
  models/                     (copied from space-orbit)
  maps/                       (38 PPM planet texture maps, copied from space-orbit)
  src/
    main.rs
    app.rs                    (game loop, key bindings, screen state machine)
    autopilot.rs              (orbit insertion: Z×r̂ prograde, radial damping)
    constants.rs
    types.rs
    math.rs
    input.rs
    physics.rs                (Newtonian + warp speed scaling)
    ai.rs
    planet_data.rs
    star_data.rs              (generated by build.rs from Yale catalog)
    config.rs                 (prefs: player name, gravity, dense stars, god mode)
    save.rs
    scores.rs
    combat/
      mod.rs, weapons.rs, missile.rs, explosions.rs, scoring.rs
    mission/
      mod.rs, parser.rs, loader.rs, events.rs, waypoints.rs
    model/
      mod.rs, tri_loader.rs, ac3d_loader.rs
    renderer/
      mod.rs
      projection.rs           (Camera: first-person + third-person)
      canvas.rs               (BrailleCanvas: set/line/rows)
      viewport.rs             (draw_scene, draw_edge_world, draw_player_ship_3p)
      planets.rs              (5-LOD wireframe, per-planet Color::Rgb, draw_single_planet)
      stars.rs                (3-band magnitude rendering, constellation lines + labels)
      texture.rs              (PPM P6 loader, raycast Lambert-lit textures)
    hud/
      mod.rs
      radar.rs                (RadarCoords port from hud.c)
      panels.rs               (layer compositor, message display, all overlays)
    ui/
      mod.rs, title_screen.rs, briefing.rs, console.rs, orrery.rs
    net/                      (Phase 10 — not yet implemented)
```

## Implementation phases

### Phase 0 — Math Kernel ✓ DONE
Port `util.c` to `src/math.rs`: `Vec3`, `RotateAbout` (Rodrigues formula), `Dotp`, `Crossp`, `Normalize`, `Mag`, `Perp`, `Vadd/Vsub/Vmul/Vdiv`.

### Phase 1 — Core State & Physics ✓ DONE
`types.rs`, `constants.rs`, `physics.rs`, `ai.rs`, `planet_data.rs`

### Phase 2 — Terminal Window, Game Loop, Braille Viewport ✓ DONE
`app.rs`, `input.rs`, `renderer/projection.rs`, `renderer/canvas.rs`, `renderer/viewport.rs`, `hud/panels.rs`

### Phase 3 — Model Loading & Wireframe Ships ✓ DONE
`model/tri_loader.rs`, `model/ac3d_loader.rs`, `renderer/viewport.rs` draw_target_ship

### Phase 4 — Mission System & Event Engine ✓ DONE
`mission/parser.rs`, `mission/loader.rs`, `mission/events.rs` — full tokenizer, cursor system, all triggers/actions

### Phase 5 — Combat, Weapons, Explosions ✓ DONE
`combat/` — fire_missile, move_missiles, destroy_target, spawn_boom, draw_booms

### Phase 6 — Planets & Solar System ✓ DONE
`renderer/planets.rs` — 5-LOD wireframe spheres, oblicity, rings, orbital path circles

### Phase 7 — Stars, Radar, Full HUD ✓ DONE
`build.rs` star codegen, `renderer/stars.rs`, `hud/radar.rs` RadarCoords port, `hud/panels.rs` full HUD

### Phase 8 — Full Campaign ✓ DONE
`save.rs`, `scores.rs`, `ui/title_screen.rs`, `ui/briefing.rs`, `ui/console.rs`, `config.rs`
- Title screen with New/Load/Quit, save slots, per-mission score leaderboard
- Win/lose mission flow

### Phase 9 — Polish & Visual Features ✓ DONE
- **Planet textures** (`renderer/texture.rs`): PPM P6 loader, full-viewport raycast, Lambert diffuse from Sol, spherical UV with oblicity, 38 maps
- **Per-planet wireframe colors**: `Color::Rgb` representative of real appearance
- **Real star field**: Yale catalog (2000 stars), magnitude-band rendering (bright/medium/dim), `build.rs` parser fix
- **Constellation lines** (B key): 14 IAU stick figures as braille lines; name labels with N key
- **Orbit autopilot** (G key): `autopilot.rs` — Z×r̂ prograde, separate radial/tangential/Z-plane damping
- **Third-person camera** (C key): `Camera::third_person` — behind ship looking toward orbited planet
- **Orrery** (O key): top-down 2D solar system view
- **Name overlays** (N key): planets, targets, constellation labels
- **Space-junk motion cue**: 8 dots parallax-drifting opposite player velocity
- **Controls**: U=lock enemy, Y=cycle targets, F=cycle weapons, X=warp toggle, M=recall message
- **Mission messages**: full multi-line display, M key recall, last_text persistence
- Training missions rewritten for terminal-orbit controls

### Phase 9b: Episode Two, part one (2026-10-10)
- Engine: `Action Score`/`Action Destroy` made real, `Shields` value split into name + threshold
  (as mission.c does), H hold key (the original's fullstop; StopNear docking was unreachable),
  `Ferry`/`Convoy` route strategies with `Route`/`Cruise`, `Reach` trigger, `Cargo` and `Waypoint`
  actions, lenient cursor numbers (squad5.inc's `+=` typo stopped mars5.msn loading)
- Missions `ep2_01`..`ep2_06` + `ep2_end`, entered from `win.msn` or the title screen's
  "Episode Two"; station/traffic includes (`shackleton`, `earthtraffic`, `marstraffic`,
  `europastation`, `jovetraffic`, `titantraffic`)
- Direction: scientific slant, inspired by real ESA/NASA missions; briefings are fiction and
  avoid real-world dates or status claims (Mars Sample Return and Gateway are in flux).
  Opponents: salvager pirates, the Jovian Combine (rival faction), the returning aliens
- Next: more Episode Two missions, then a free-contract mode (delivery and bounty jobs at
  stations) reusing the same stations and traffic
- `loader.rs` test loads every mission and checks LoadMission/Enable targets and route stops
- Standard orbit: G flies to a westward equatorial orbit at 1.25 radii, level with top to north
  (`autopilot.rs`; westward so the shots below come out with north up), orbit speed
  derived from `physics::gravity` (per-frame pull with RMIN clamp, as orbit.c; the old
  sqrt(Gm/r) was ~5x too slow), centripetal feed-forward when prefs gravity is off;
  `Camera::standard_orbit` has two `OrbitShot`s looking across the ship at the planet, north
  up: Above (behind, planet left) and Below (in front, planet right), all in planet radii
- Warnings cleaned to zero: dead C-header constants/fields removed; Arcade flight model and
  `draw_orbits` (orbital paths, `World::orbit_planets`) kept with `allow(dead_code)`, ported
  but not yet bound to a key

### Phase 10 — Multiplayer (not started)
- `src/net/protocol.rs`: PositionUpdate, FireMissile, TargetDestroyed, MissionLoad, Ping/Pong, PlayerJoin/Leave
- `src/net/server.rs`: tokio async, accept connections, broadcast state
- `src/net/client.rs`: tokio async, periodic position reports, apply remote updates

**Done when:** two instances on LAN see each other's ships and can score kills.

## Game scale reference

- `KM_TO_UNITS1 = 6000` — 1 game unit = 6000 km (for planet radii)
- `KM_TO_UNITS2 = 0.006` — for compressed orbital distances
- Earth: radius = 6371/6000 = 1.062 units; dist_compressed = 14.96 → dist = 14.96/0.006 = 2493 units
- Low Earth orbit: ~1.4 units from Earth center
- Camera third-person: CAM_3P_DIST=2.0, CAM_3P_HEIGHT=0.5 (calibrated for 1.062-unit planet)
- Star coordinates: unit vectors from RA/Dec — x=cos(dec)cos(ra), y=cos(dec)sin(ra), z=sin(dec); ra in radians from decimal hours × π/12
