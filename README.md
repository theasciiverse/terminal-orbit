# terminal-orbit

**THE ALIENS ARE ALREADY HERE.**

It started with a strange alien construction, quietly gathering energy, and
then the alien fighters came. Fly a small starfighter through the real solar
system with real Newtonian physics: thrust, brake, coast, swing into orbit,
match speeds and dock, then do it again with aliens on your tail, from the
Earth and the Moon to Mars, Jupiter, the rings of Saturn, Neptune and Pluto.

This is a terminal port of [ORBIT](http://genesis.nred.ma.us/orbit/), Steve
Belczyk's 1999 freeware space combat simulator, from his own C source: the
whole original campaign, the first part of a new Episode Two, and space drawn
in Unicode Braille with [ratatui](https://github.com/ratatui/ratatui), with
raycast planet surfaces and a real star catalogue.

## Play

With [av](https://asciiverse.sh), the Asciiverse client:

```
av install terminal-orbit
av play terminal-orbit
```

Or download the archive for your system from this repository's Releases,
unpack it and run `./terminal-orbit` from that folder (it reads the
`missions/`, `models/` and `maps/` beside it).

To build it yourself: `cargo build --release` (Rust 1.75 or newer), then run
it from the repository folder.

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
| `H` | Hold: station-keeping, zeroes velocity until you thrust again (needed to dock) |

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
| `C` | Toggle third-person camera (in orbit: cycle the two orbit shots and the cockpit) |
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
- Medium stars (1.5-3.5): light gray
- Dim stars (3.5+): dark gray single dot

Press `B` to overlay IAU stick-figure constellation lines for 14 major constellations. Press `N` to add constellation name labels.

### Planet Textures
Press `T` to switch from wireframe to full-viewport raycasted textures. Each planet is ray-sphere intersected per terminal cell, UV-mapped with oblicity rotation, and lit with Lambert diffuse from Sol (40% ambient so night side stays visible). 38 planet maps included.

### Orbit Autopilot
Press `G` and the ship flies itself into standard orbit around the nearest planet: a westward equatorial circle at 1.25 planet radii, flying level along the equator with its top to north. The orbit speed matches the game's own gravity, so the orbit holds after you press `G` again to release it (with gravity on in the prefs). `G` also switches to the standard-orbit camera, the classic starship-in-orbit shot looking across the ship at the planet, north up. `C` cycles two shots and the cockpit: from above and slightly behind with the planet filling the left, then from below and slightly in front with the planet on the right. Both scale with the planet, so they look the same at Earth or Phobos. Press `T` for the textured planet.

### Mission System & Campaign
Full `.msn` tokenizer with Include recursion, cursor-relative positioning, and the complete event engine: Alarm/Approach/Depart/StopNear/Score/Destroy/Shields/Reach triggers driving Message/Hide/Unhide/Score/Destroy/Cargo/Waypoint/LoadMission/Boom/Stop actions.

## Campaign

28 missions across 8 star systems:

| System | Missions |
|--------|----------|
| Earth orbit (training) | train01-04 |
| Mars | mars1-5 |
| Jupiter | jupiter1-5 |
| Saturn | saturn01-04 |
| Titan / Dione | titan01-03, dione01 |
| Uranus | uranus01 |
| Neptune | neptune1-2 |
| Pluto | pluto1 |

### Episode Two: Open Skies

A new campaign after Steve Belczyk's `win.msn`, also startable from the title screen. Science missions inspired by real ESA and NASA programmes, with salvager pirates, the breakaway Jovian Combine and the returning aliens as opponents. The briefings are fiction built on real science, and make no claims about real mission dates or status.

| Mission | Where | Inspired by | Gameplay |
|---------|-------|-------------|----------|
| ep2_01 Lunar Link | Earth, Moon | ESA Moonlight, Lunar Pathfinder | Dock, carry a relay satellite to the far side |
| ep2_02 Sample Return | Mars | NASA/ESA Mars Sample Return studies | Race salvagers to a sample container, deliver it |
| ep2_03 Clipper Run | Europa | NASA Europa Clipper | Escort a freighter through raiders |
| ep2_04 Ice Shell | Europa | Hubble plume sightings, ESA JUICE | Deploy three samplers; the Combine turns hostile |
| ep2_05 Dragonfly | Titan | NASA Dragonfly | Drop a lander through a Combine blockade |
| ep2_06 Faint Signal | Pluto | New Horizons | Guard a listening post at the alien portal |

Stations and traffic (`*.inc`) keep the solar system busy: freighters run between Earth Base and Shackleton Station, Mars Science and Deimos Depot, Europa Station and the Jovian relay, Titan Base and Kraken Depot.

### Mission format additions

On top of the original ORBIT keywords:

- `Object { ... Strategy Ferry Route { A B } Cruise 1500 }`: unarmed traffic flying between named objects in a loop. `Strategy Convoy` flies the route once and holds at the end. `Cruise` is in km/s.
- `Trigger Reach Value { Ship 1000 }`: fires when the named ship comes within 1000 km of the event's cursor.
- `Action Cargo Value { text }` shows cargo on the HUD; `Value none` clears it.
- `Action Waypoint Value N` points the HUD waypoint arrow at waypoint N.
- `Action Score` and `Action Destroy` work as in the original (they were placeholders before).

## HUD

```
┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│   (3D braille viewport: ships, textured planets, real starfield)  │
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
  app.rs:          game loop, key bindings, screen state machine
  autopilot.rs:    orbit insertion (Z×r̂ prograde, radial damping)
  config.rs:       prefs (player name, gravity, etc.)
  save.rs:         save slots
  scores.rs:       per-mission high scores
  physics.rs:      Newtonian flight + gravity + warp
  ai.rs:           8 AI strategies (DoNothing, Sit1-4, Hunt1-4)
  math.rs:         Vec3 + Rodrigues rotation (ported from util.c)
  planet_data.rs:  real solar system constants
  mission/:        .msn tokenizer, keyword handlers, event engine
  combat/:         weapons, missiles, explosions, scoring
  renderer/
    projection.rs: camera (first-person + third-person)
    canvas.rs:     drawille braille adapter
    viewport.rs:   scene draw, wireframe ships
    planets.rs:    5-LOD wireframe spheres, per-planet color
    stars.rs:      magnitude-band rendering, constellation lines
    texture.rs:    PPM P6 loader, raycasted Lambert-lit textures
  hud/
    radar.rs:      RadarCoords port from hud.c
    panels.rs:     stats panel, message display, layer compositor
  model/:          .tri and AC3D wireframe loaders
  ui/:             title screen, briefing, orrery, console
build.rs:          parses stars.h → star_data.rs (Yale catalog)
```

## Credits

Original ORBIT game © 1999 Steve Belczyk, released under the GNU General Public License.  
Terminal port written in Rust using [ratatui](https://github.com/ratatui/ratatui) and [drawille](https://github.com/asciimoo/drawille).

## Licence

GNU General Public License, version 2 or (at your option) any later version,
as ORBIT itself; see [COPYING](COPYING).
