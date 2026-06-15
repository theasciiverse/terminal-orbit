# terminal-orbit

A terminal UI port of [ORBIT](http://genesis.nred.ma.us/orbit/) — Steve Belczyk's 1999 freeware space combat simulator — rendered entirely in the terminal using Unicode braille characters and [ratatui](https://github.com/ratatui/ratatui).

The original game taught real Newtonian orbital mechanics through combat. This port preserves that physics and the full original campaign while replacing the OpenGL renderer with a 3D braille viewport, raycasted planet textures, a real star catalog with constellation lines, and TUI HUD panels.

## Build

```sh
cargo build --release
```

Requires Rust 1.75+.

## Run

Must be run from the directory that contains the `missions/`, `models/`, and `maps/` folders:

```sh
cargo run --release
```

or after building:

```sh
./target/release/terminal-orbit
```

## Controls

### Flight

| Key | Action |
|-----|--------|
| `W` / `↑` | Thrust forward |
| `S` / `↓` | Brake / reverse thrust |
| `A` / `←` | Yaw left |
| `D` / `→` | Yaw right |
| `I` | Pitch up |
| `K` | Pitch down |
| `J` | Roll left |
| `L` | Roll right |
| `X` | Toggle warp engines (high-speed thrust) |

### Combat

| Key | Action |
|-----|--------|
| `Space` | Fire weapon |
| `U` | Lock nearest enemy |
| `Y` | Cycle through targets |
| `F` | Cycle weapons |

### View & Navigation

| Key | Action |
|-----|--------|
| `G` | Toggle orbit autopilot (locks onto nearest planet) |
| `C` | Toggle third-person camera |
| `T` | Toggle planet textures (raycasted PPM maps) |
| `B` | Toggle constellation lines |
| `N` | Toggle names (planets, targets, constellation labels) |
| `O` | Toggle orrery (top-down solar system view) |
| `Z` | Toggle dense star field (200 → 2000 stars) |
| `M` | Recall last mission message / dismiss current |
| `P` | Pause |
| `Q` / `Esc` | Return to title (auto-saves progress) |

Navigation on title and load screens uses `↑`/`↓` and `Enter`.

## Features

### Braille 3D Viewport
Ships, planets, and orbital paths rendered as Unicode braille dot clusters at 2× horizontal and 4× vertical sub-pixel resolution. Wireframe models loaded from the original `.tri` and AC3D formats.

### Real Star Field
2000 stars from the Yale Bright Star Catalogue (embedded in the original ORBIT source), rendered with magnitude-based sizing and color:
- Bright stars (mag < 1.5): white 2×2 braille cluster
- Medium stars (1.5–3.5): light gray
- Dim stars (3.5+): dark gray single dot

Press `B` to overlay IAU stick-figure constellation lines for 14 major constellations. Press `N` to add constellation name labels.

### Planet Textures
Press `T` to switch from wireframe to full-viewport raycasted textures. Each planet is ray-sphere intersected per terminal cell, UV-mapped with oblicity rotation, and lit with Lambert diffuse from Sol (40% ambient so night side stays visible). 38 planet maps included.

### Orbit Autopilot
Press `G` to engage — the ship automatically inserts into a stable circular orbit around the nearest planet. Prograde direction is computed via Z×r̂ (not velocity-dependent), with separate radial and out-of-plane velocity damping. Press `C` for a cinematic third-person camera looking from behind the ship toward the planet.

### Mission System & Campaign
Full `.msn` tokenizer with Include recursion, cursor-relative positioning, and the complete event engine: Alarm/Approach/Depart/Score/Destroy/Shields triggers driving Message/Hide/Unhide/LoadMission/Boom/Stop actions.

## Campaign

28 missions across 8 star systems:

| System | Missions |
|--------|----------|
| Earth orbit (training) | train01–04 |
| Mars | mars1–5 |
| Jupiter | jupiter1–5 |
| Saturn | saturn01–04 |
| Titan / Dione | titan01–03, dione01 |
| Uranus | uranus01 |
| Neptune | neptune1–2 |
| Pluto | pluto1 |

## HUD

```
┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│   (3D braille viewport — ships, textured planets, real starfield)  │
│                                                                     │
│  console messages appear here in green (3s fade)                   │
├─────────────┬───────────────────────────────────────────────────────┤
│   Radar     │  Shields ████████░░░  80%                             │
│  ·          │  Throttle ███░░░░░░░  30%                             │
│   +         │  Spd:0.123  Pos:(x,y,z)                               │
│       ·     │  Score:42  Wpn:Laser                                  │
│             │  Target: enemy1  1.2u [enemy]                         │
│  11×21 grid │  train01.msn  [WASD/IJKL fly  SPC fire  U lock ...]  │
└─────────────┴───────────────────────────────────────────────────────┘
```

Radar blips: `·` planets, `+` enemies, `f` friendlies, `X` locked, `o` missiles, `W` waypoint.

## Config & Saves

All persistent data lives in `~/.config/terminal-orbit/`:

| File | Contents |
|------|----------|
| `config.toml` | Player name, gravity, dense stars, god mode |
| `saves.toml` | Up to 10 save slots (mission name + timestamp) |
| `scores.toml` | Top 5 scores per mission |

```toml
player_name = "Ace"
gravity = false
dense_stars = false
vulnerable = false   # set true for god mode
```

## Architecture

```
src/
  app.rs          — game loop, key bindings, screen state machine
  autopilot.rs    — orbit insertion (Z×r̂ prograde, radial damping)
  config.rs       — prefs (player name, gravity, etc.)
  save.rs         — save slots
  scores.rs       — per-mission high scores
  physics.rs      — Newtonian flight + gravity + warp
  ai.rs           — 8 AI strategies (DoNothing, Sit1-4, Hunt1-4)
  math.rs         — Vec3 + Rodrigues rotation (ported from util.c)
  planet_data.rs  — real solar system constants
  mission/        — .msn tokenizer, keyword handlers, event engine
  combat/         — weapons, missiles, explosions, scoring
  renderer/
    projection.rs — camera (first-person + third-person)
    canvas.rs     — drawille braille adapter
    viewport.rs   — scene draw, wireframe ships
    planets.rs    — 5-LOD wireframe spheres, per-planet color
    stars.rs      — magnitude-band rendering, constellation lines
    texture.rs    — PPM P6 loader, raycasted Lambert-lit textures
  hud/
    radar.rs      — RadarCoords port from hud.c
    panels.rs     — stats panel, message display, layer compositor
  model/          — .tri and AC3D wireframe loaders
  ui/             — title screen, briefing, orrery, console
build.rs          — parses stars.h → star_data.rs (Yale catalog)
```

## Credits

Original ORBIT game © 1999 Steve Belczyk — released as freeware.  
Terminal port written in Rust using [ratatui](https://github.com/ratatui/ratatui) and [drawille](https://github.com/asciimoo/drawille).
