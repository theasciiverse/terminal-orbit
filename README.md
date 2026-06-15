# terminal-orbit

A terminal UI port of [ORBIT](http://genesis.nred.ma.us/orbit/) — Steve Belczyk's 1999 freeware space combat simulator — rendered entirely in the terminal using Unicode braille characters and [ratatui](https://github.com/ratatui/ratatui).

The original game taught real Newtonian orbital mechanics through combat. This port preserves that physics and the full original campaign while replacing the OpenGL renderer with a 3D braille viewport and TUI HUD panels.

## Build

```sh
cargo build --release
```

Requires Rust 1.75+.

## Run

Must be run from the directory that contains the `missions/` and `models/` folders:

```sh
cargo run --release
```

or after building:

```sh
./target/release/terminal-orbit
```

## Controls

| Key | Action |
|-----|--------|
| `W` / `↑` | Thrust forward |
| `S` / `↓` | Brake / reverse |
| `A` / `←` | Yaw left |
| `D` / `→` | Yaw right |
| `I` | Pitch up |
| `K` | Pitch down |
| `J` | Roll left |
| `L` | Roll right |
| `Space` | Fire weapon |
| `Z` | Toggle dense star field (200 → 2000 stars) |
| `Q` / `Esc` | Return to title (auto-saves progress) |

Navigation on the title and load screens uses `↑`/`↓` and `Enter`.

## Campaign

28 missions across 8 star systems, in order:

| System | Missions |
|--------|----------|
| Earth orbit (training) | train01–04 |
| Mars | mars1–5 |
| Jupiter | jupiter1–5 |
| Saturn | saturn01–04 |
| Titan (Saturn moon) | titan01–03, dione01 |
| Uranus | uranus01 |
| Neptune | neptune1–2 |
| Pluto | pluto1 |

## HUD

```
┌─────────────────────────────────────────────────────┐
│                                                     │
│   (3D braille viewport — ships, planets, stars)    │  ← 2× horizontal, 4× vertical sub-pixel
│                                                     │
│  console messages appear here in green             │
├─────────────┬───────────────────────────────────────┤
│   Radar     │  Shields ████████░░░  80%             │
│  ·          │  Throttle ███░░░░░░░  30%             │
│   +         │  Spd:0.123  Pos:(x,y,z)               │
│       ·     │  Score:42  Wpn:MedMissile              │
│  Radar      │  Target: enemy1  1.2u [enemy]          │
│  11×21 grid │  WP#0 ↗ 3.5u                          │
│             │  train01.msn  [WASD/IJKL fly  Q quit] │
└─────────────┴───────────────────────────────────────┘
```

Radar blips: `·` planets, `+` enemies, `f` friendlies, `X` locked, `o` missiles, `W` waypoint.

## Config & Saves

All persistent data lives in `~/.config/terminal-orbit/`:

| File | Contents |
|------|----------|
| `config.toml` | Player name, gravity, dense stars, god mode |
| `saves.toml` | Up to 10 save slots (mission name + timestamp) |
| `scores.toml` | Top 5 scores per mission |

Edit `config.toml` directly to change your player name or toggle options:

```toml
player_name = "Ace"
gravity = false
dense_stars = false
vulnerable = false   # set true for god mode
```

## Architecture

```
src/
  app.rs          — game loop & screen state machine
  config.rs       — prefs (player name, gravity, etc.)
  save.rs         — save slots
  scores.rs       — per-mission high scores
  physics.rs      — Newtonian flight + gravity
  ai.rs           — 8 AI strategies (DoNothing, Sit1-4, Hunt1-4)
  math.rs         — Vec3 + Rodrigues rotation (ported from util.c)
  planet_data.rs  — real solar system constants
  mission/        — .msn tokenizer, keyword handlers, event engine
  combat/         — weapons, missiles, explosions, scoring
  renderer/       — braille canvas, projection, planets, stars
  hud/            — radar (RadarCoords port), stats panels
  model/          — .tri and AC3D wireframe loaders
  ui/             — title screen, briefing, console
```

## Credits

Original ORBIT game © 1999 Steve Belczyk — released as freeware.  
Terminal port written in Rust using [ratatui](https://github.com/ratatui/ratatui) and [drawille](https://github.com/asciimoo/drawille).
