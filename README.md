# WOPR TUI 2026

A modern terminal reimagining of the WarGames WOPR (War Operation Plan Response) system. Built with Rust, ratatui, and LLM-powered scenario generation.

![Rust](https://img.shields.io/badge/Rust-2024-orange) ![License](https://img.shields.io/badge/license-MIT-blue)

## What is this?

An interactive Cold War simulation TUI where you play as a military advisor responding to escalating geopolitical crises. The AI generates scenarios with multi-language communications (English, Russian, Chinese), and your decisions shift the DEFCON level toward peace or nuclear war.

## Features

- **World Map** — ASCII continental outlines with 9 strategic locations, threat overlays, and missile trajectories
- **DEFCON System** — Levels 5→1 with ±1 step enforcement and visual gauge
- **Comms Panel** — Multi-language intercepts (Cyrillic, Chinese simplified, English) with priority coloring and garble effects
- **Scenario Engine** — LLM-generated crises with 4 player options per turn, consequence heuristics, and endgame detection
- **LLM Providers** — Stub (6 canned scenarios), Anthropic, and Minimax provider support
- **Animations** — Braille spinners, typewriter text, pulse/radar overlays, nerd font detection with fallbacks

## Install

```bash
curl -fsSL https://raw.githubusercontent.com/ankurCES/WOPR_TUI_2026/main/WOPR_TUI_2026/install.sh | bash
```

Installs Rust (if needed), system deps, builds from source, and puts `wopr` on your PATH. Then just run:

```bash
wopr
```

### Controls

| Key | Action |
|-----|--------|
| `Tab` / `Shift+Tab` | Cycle modes (MainMap → Comms → Settings → Scenario → Defcon) |
| `Up` / `Down` | Select option in scenario view |
| `Enter` | Submit decision |
| `?` | Help |
| `q` | Quit |

## Configuration

Reads LLM settings from `~/.blumi/settings.json` (shared with other blumi apps). The relevant fields:

```json
{
  "llm": { "provider": "minimax", "model": "MiniMax-M3" },
  "providers": {
    "minimax": {
      "api_key": "sk-...",
      "base_url": "https://api.minimax.io/anthropic",
      "kind": "anthropic"
    }
  }
}
```

Falls back to stub provider (no API key needed) if `~/.blumi/settings.json` is missing.

## Architecture

```
src/
├── app.rs          # Main event loop + game flow
├── main.rs         # Entry point
├── state.rs        # AppState (DEFCON, scenarios, comms, threats)
├── event.rs        # AppEvent channel
├── config.rs       # Settings loader
├── terminal.rs     # Terminal init/restore, capability detection
├── game/
│   ├── types.rs    # Country, ScenarioCategory, CommPriority
│   ├── scenario.rs # Scenario parsing from LLM output
│   ├── context.rs  # GameContext accumulator (JSON history)
│   ├── comms.rs    # Multi-language comm generation
│   ├── consequence.rs # Decision → game event mapping
│   ├── defcon.rs   # DEFCON level transitions
│   ├── endgame.rs  # Win/loss detection + ASCII art
│   ├── events.rs   # GameEvent enum
│   └── prompts.rs  # System + scenario prompts for LLM
├── llm/
│   ├── types.rs    # LlmProvider trait, LlmRequest/Response
│   ├── stub.rs     # 6 canned scenarios (Russian/Chinese comms)
│   ├── anthropic.rs # Anthropic Claude provider
│   └── minimax.rs  # Minimax provider
└── ui/
    ├── layout.rs       # Mode-based layout routing
    ├── world_map.rs    # ASCII continents + city markers
    ├── threat_overlay.rs # Missiles, threats, bases
    ├── comms_panel.rs  # Scrollable comms feed
    ├── decision.rs     # Scenario + options panel
    ├── icons.rs        # Nerd font / ASCII fallback icons
    └── anim.rs         # Braille spinner, typewriter, pulse
```

## Tech Stack

- **[ratatui](https://ratatui.rs)** 0.30 — terminal UI framework
- **[crossterm](https://github.com/crossterm-rs/crossterm)** 0.28 — terminal backend
- **[tokio](https://tokio.rs)** — async runtime
- **[reqwest](https://docs.rs/reqwest)** — HTTP client for LLM providers
- **[serde](https://serde.rs)** — JSON serialization

## Status

Wave 1+2 complete. The game shell, rendering pipeline, and scenario engine are fully wired. Currently runs with the stub provider (6 canned scenarios). Live LLM streaming and full game-loop integration are next.

## License

MIT
