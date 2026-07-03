# kojira

TUI for Jira (backlog & boards) built with Rust, ratatui and crossterm.

## Build & Run

```bash
cargo run       # run the app
cargo check     # type-check
cargo build --release  # release build
```

## Architecture

Same patterns as lazyglab (`../lazyglab/`):

- **Single state owner**: `App` struct in `src/app.rs` holds all state
- **Async event loop**: tokio + crossterm EventStream in `src/event.rs`
- **Immediate-mode rendering**: click regions re-recorded each frame in `src/ui/`
- **Config**: TOML at `~/.config/kojira/config.toml`, loaded via `src/config/`
- **Themes**: 12 shared themes in `src/theme.rs` (same palette as lazyglab)

## Module Layout

```
src/
  main.rs          -- entry point, terminal setup
  app.rs           -- App state, key/mouse handling
  event.rs         -- async event loop (key, mouse, tick)
  theme.rs         -- 12 theme palettes
  config/
    mod.rs         -- load/save TOML config
    types.rs       -- AppConfig, JiraConfig, UiConfig, AuthConfig
  ui/
    mod.rs         -- render dispatcher
    main_view.rs   -- header, tabs, content, footer
    click_regions.rs -- clickable area tracking
```
