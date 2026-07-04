# kojira

A terminal UI (TUI) for Jira — browse backlogs and boards, manage issues, and create tasks without leaving your terminal.

Built with Rust. Mouse-first interaction. Heavily inspired by [lazyglab](https://github.com/ragamo/lazyglab).

<!-- screenshot placeholder
<video src="" autoplay loop muted playsinline></video>
-->

<div>
  <!-- <a href="caps/01_backlog.png"><img src="caps/01_backlog.png" width="24%" alt="Backlog" /></a> -->
  <!-- <a href="caps/02_board.png"><img src="caps/02_board.png" width="24%" alt="Board" /></a> -->
  <!-- <a href="caps/03_issue_detail.png"><img src="caps/03_issue_detail.png" width="24%" alt="Issue detail" /></a> -->
  <!-- <a href="caps/04_create_issue.png"><img src="caps/04_create_issue.png" width="24%" alt="Create issue" /></a> -->
</div>

## Installation

### From source

Requires Rust 1.85+ (edition 2024).

```bash
git clone https://github.com/ragamo/kojira
cd kojira
cargo build --release
./target/release/kojira
```

## Configuration

Config is stored at `~/.config/kojira/config.toml`.

```toml
[auth]
token = "your-jira-api-token"
email = "you@example.com"

[jira]
base_url = "https://yourcompany.atlassian.net"
```

The first time you run kojira you can enter your credentials directly from the login screen.

## Features

- **Backlog** — list issues per project with status filters
- **Board view** — kanban-style columns with drag-and-drop to change issue status
- **Issue detail** — tabbed panel with Overview, Comments, and Transitions; editable title and description
- **Create issue** — bottom panel with title, description, type, priority, epic, and assignee selectors
- **Inline field editing** — click assignee, parent, or priority in the detail sidebar to update them directly
- **Transitions** — change issue status via the transition button or keyboard shortcut
- **Multi-tab** — open multiple projects and boards as tabs; reorder by drag-and-drop
- **Per-user tab persistence** — open tabs are saved per Jira account and restored on login
- **12 color themes** — One Dark, Catppuccin, Tokyo Night, Dracula, Nord, Gruvbox, Solarized, and more
- **Settings** — theme and UI options, persisted to config
- **Mouse support** — every interactive element is clickable

## Views

### Backlog

<!-- <div align="center">
  <a href="caps/01_backlog.png"><img src="caps/01_backlog.png" width="80%" alt="Backlog" /></a>
</div> -->

### Board

<!-- <div align="center">
  <a href="caps/02_board.png"><img src="caps/02_board.png" width="80%" alt="Board" /></a>
</div> -->

### Issue detail

<!-- <table>
  <tr>
    <td><a href="caps/03_issue_overview.png"><img src="caps/03_issue_overview.png" alt="Overview" /></a></td>
    <td><a href="caps/04_issue_comments.png"><img src="caps/04_issue_comments.png" alt="Comments" /></a></td>
  </tr>
  <tr>
    <td><a href="caps/05_issue_transitions.png"><img src="caps/05_issue_transitions.png" alt="Transitions" /></a></td>
    <td><a href="caps/06_issue_edit.png"><img src="caps/06_issue_edit.png" alt="Edit" /></a></td>
  </tr>
</table> -->

### Create issue

<!-- <div align="center">
  <a href="caps/07_create_issue.png"><img src="caps/07_create_issue.png" width="80%" alt="Create issue" /></a>
</div> -->

### Themes

<!-- <table>
  <tr>
    <td><a href="caps/10_theme_one_dark.png"><img src="caps/10_theme_one_dark.png" alt="One Dark" /></a></td>
    <td><a href="caps/11_theme_catppuccin.png"><img src="caps/11_theme_catppuccin.png" alt="Catppuccin" /></a></td>
    <td><a href="caps/12_theme_dracula.png"><img src="caps/12_theme_dracula.png" alt="Dracula" /></a></td>
  </tr>
</table> -->

## Keybindings

| Key | Action |
|-----|--------|
| `Tab` / `←` / `→` | Switch tabs |
| `1`–`9` | Jump to tab by number |
| `n` | Open project/board finder |
| `c` | Create new issue |
| `e` | Edit issue title and description (in detail panel) |
| `t` | Open transition menu (in detail panel) |
| `r` | Refresh active tab |
| `x` | Close active tab |
| `,` | Open settings |
| `q` | Quit |
| `Esc` | Close panel / cancel |

Mouse clicks work on all interactive elements: tabs, board cards, field selectors, transition button, create/settings links, and logout.
