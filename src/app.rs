use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::prelude::Rect;
use tokio::sync::mpsc;

use crate::config;
use crate::config::types::{AppConfig, FavoriteProject, OpenTab};
use crate::event::{AppEvent, AppMessage};
use crate::provider::jira::JiraProvider;
use crate::provider::types::{IssueMetadata, JiraBoard, JiraChangelogEntry, JiraComment, JiraIssue, JiraTransition};
use crate::table_nav::TableNav;
use crate::theme::{self, Theme};
use crate::ui::click_regions::ClickRegions;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tab {
    List(u64),
    Board(u64),
}

#[derive(Debug, Clone)]
pub struct ListTab {
    pub id: u64,
    pub project_key: String,
    pub project_name: String,
    pub issues: Vec<JiraIssue>,
    pub loading: bool,
    pub error: Option<String>,
    pub nav: TableNav,
    pub filter: Option<String>,
    pub statuses: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct BoardTab {
    pub board_id: u64,
    pub board_name: String,
    pub columns: Vec<BoardColumn>,
    pub col_scroll: Vec<usize>,
    pub loading: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct BoardColumn {
    pub name: String,
    pub status_ids: Vec<String>,
    pub issues: Vec<JiraIssue>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusLayer {
    Main,
    Settings,
    Auth,
    Find,
    FindBoardPanel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthField {
    Subdomain,
    Email,
    Token,
}

#[derive(Debug, Clone)]
pub struct FindProject {
    pub key: String,
    pub name: String,
}

#[derive(Default)]
pub struct SimpleEditor {
    pub lines: Vec<String>,
    pub cursor_row: usize,
    pub cursor_col: usize,
    pub scroll: u16,
}

impl SimpleEditor {
    pub fn load(&mut self, text: &str) {
        self.lines = text.lines().map(|l| l.to_string()).collect();
        if self.lines.is_empty() {
            self.lines.push(String::new());
        }
        self.cursor_row = 0;
        self.cursor_col = 0;
        self.scroll = 0;
    }

    pub fn to_text(&self) -> String {
        self.lines.join("\n")
    }

    pub fn input(&mut self, key: KeyEvent) {
        let row = self.cursor_row.min(self.lines.len().saturating_sub(1));
        let col = self.cursor_col.min(self.lines.get(row).map(|l| l.chars().count()).unwrap_or(0));

        match key.code {
            KeyCode::Char(c) => {
                if let Some(line) = self.lines.get_mut(row) {
                    let mut chars: Vec<char> = line.chars().collect();
                    chars.insert(col, c);
                    *line = chars.into_iter().collect();
                    self.cursor_col = col + 1;
                }
            }
            KeyCode::Backspace => {
                if col > 0 {
                    if let Some(line) = self.lines.get_mut(row) {
                        let mut chars: Vec<char> = line.chars().collect();
                        chars.remove(col - 1);
                        *line = chars.into_iter().collect();
                        self.cursor_col = col - 1;
                    }
                } else if row > 0 {
                    let cur = self.lines.remove(row);
                    let prev_len = self.lines.get(row - 1).map(|l| l.chars().count()).unwrap_or(0);
                    if let Some(prev) = self.lines.get_mut(row - 1) {
                        prev.push_str(&cur);
                    }
                    self.cursor_row = row - 1;
                    self.cursor_col = prev_len;
                }
            }
            KeyCode::Delete => {
                let line_len = self.lines.get(row).map(|l| l.chars().count()).unwrap_or(0);
                if col < line_len {
                    if let Some(line) = self.lines.get_mut(row) {
                        let mut chars: Vec<char> = line.chars().collect();
                        chars.remove(col);
                        *line = chars.into_iter().collect();
                    }
                } else if row + 1 < self.lines.len() {
                    let next = self.lines.remove(row + 1);
                    if let Some(cur) = self.lines.get_mut(row) {
                        cur.push_str(&next);
                    }
                }
            }
            KeyCode::Enter => {
                if let Some(line) = self.lines.get_mut(row) {
                    let rest: String = line.chars().skip(col).collect();
                    let new_len = line.chars().count() - rest.chars().count();
                    *line = line.chars().take(new_len).collect();
                    self.lines.insert(row + 1, rest);
                }
                self.cursor_row = row + 1;
                self.cursor_col = 0;
            }
            KeyCode::Left => {
                if col > 0 {
                    self.cursor_col = col - 1;
                } else if row > 0 {
                    self.cursor_row = row - 1;
                    self.cursor_col = self.lines.get(row - 1).map(|l| l.chars().count()).unwrap_or(0);
                }
            }
            KeyCode::Right => {
                let line_len = self.lines.get(row).map(|l| l.chars().count()).unwrap_or(0);
                if col < line_len {
                    self.cursor_col = col + 1;
                } else if row + 1 < self.lines.len() {
                    self.cursor_row = row + 1;
                    self.cursor_col = 0;
                }
            }
            KeyCode::Up => {
                if row > 0 {
                    self.cursor_row = row - 1;
                    let new_line_len = self.lines.get(row - 1).map(|l| l.chars().count()).unwrap_or(0);
                    self.cursor_col = col.min(new_line_len);
                }
            }
            KeyCode::Down => {
                if row + 1 < self.lines.len() {
                    self.cursor_row = row + 1;
                    let new_line_len = self.lines.get(row + 1).map(|l| l.chars().count()).unwrap_or(0);
                    self.cursor_col = col.min(new_line_len);
                }
            }
            KeyCode::Home => { self.cursor_col = 0; }
            KeyCode::End => {
                self.cursor_col = self.lines.get(row).map(|l| l.chars().count()).unwrap_or(0);
            }
            _ => {}
        }
    }
}

pub struct App {
    pub running: bool,
    pub theme: &'static Theme,
    pub header_bg_soft: bool,
    pub active_tab: Tab,
    pub projects: Vec<FavoriteProject>,
    pub selected_project: usize,
    pub click_regions: ClickRegions,
    pub config: AppConfig,
    pub focus: FocusLayer,

    // Auth
    pub logged_in: bool,
    pub user_display_name: Option<String>,
    pub user_email: Option<String>,
    pub auth_open: bool,
    pub auth_field: AuthField,
    pub subdomain_input: String,
    pub email_input: String,
    pub token_input: String,
    pub is_validating: bool,
    pub auth_error: Option<String>,

    // List tabs (each is an independent backlog view)
    pub list_tabs: Vec<ListTab>,
    pub next_list_id: u64,

    // column_order is still shared (same project → same board config)
    pub column_order: Vec<String>,

    // Detail panel
    pub detail_open: bool,
    pub detail_tab: usize,
    pub detail_tab_areas: Vec<Rect>,
    pub detail_issue: Option<JiraIssue>,
    pub detail_description: Option<String>,
    pub detail_comments: Vec<JiraComment>,
    pub detail_changelog: Vec<JiraChangelogEntry>,
    pub detail_metadata: Option<IssueMetadata>,
    pub detail_height: u16,
    pub detail_scroll: u16,
    pub detail_max_scroll: u16,
    pub detail_close_area: Option<Rect>,
    pub detail_resize_area: Option<Rect>,
    pub detail_url_area: Option<Rect>,
    pub detail_dragging: bool,
    pub detail_editing: bool,
    pub detail_editor: SimpleEditor,
    pub detail_transitions: Vec<JiraTransition>,
    pub detail_transition_open: bool,
    pub detail_transition_selected: usize,
    pub detail_transition_btn_area: Option<Rect>,

    // Board tabs
    pub board_tabs: Vec<BoardTab>,

    // Find modal
    pub find_modal_open: bool,
    pub find_input: String,
    pub find_results: Vec<FindProject>,
    pub find_selected: usize,
    pub find_loading: bool,

    // Find modal – board panel (right side)
    pub find_board_panel_open: bool,
    pub find_panel_project: Option<FindProject>,
    pub find_panel_boards: Vec<JiraBoard>,
    pub find_panel_selected: Vec<bool>,
    pub find_panel_cursor: usize,
    pub find_panel_loading: bool,

    // Async messaging
    pub message_tx: mpsc::UnboundedSender<AppMessage>,
    pub http_client: reqwest::Client,

    // Settings modal
    pub settings_open: bool,
    pub settings_selected: usize,
    pub settings_tab_areas: Vec<Rect>,
    pub settings_theme_areas: Vec<Rect>,
    pub settings_header_soft_area: Option<Rect>,
    pub settings_header_hard_area: Option<Rect>,
    pub settings_board_on_area: Option<Rect>,
    pub settings_board_off_area: Option<Rect>,
    pub settings_board_backlog_on_area: Option<Rect>,
    pub settings_board_backlog_off_area: Option<Rect>,
    pub settings_apply_area: Option<Rect>,
    pub settings_close_area: Option<Rect>,
    pub board_hide_subtasks: bool,
    pub board_hide_backlog_col: bool,
    pub settings_board_field: usize,
    pub mouse_pos: (u16, u16),
    pub board_content_area: Option<Rect>,
    pub theme_selected: usize,
    pub theme_confirmed: usize,
    pub header_bg_confirmed: bool,
}

impl App {
    pub fn new(config: AppConfig, message_tx: mpsc::UnboundedSender<AppMessage>) -> Self {
        let theme = config
            .ui
            .theme
            .as_deref()
            .map(theme::find_theme)
            .unwrap_or(&theme::ONE_DARK);

        let header_bg_soft = config.ui.header_bg.as_deref() != Some("hard");

        let theme_selected = theme::ALL_THEMES
            .iter()
            .position(|t| t.name == theme.name)
            .unwrap_or(0);

        // Derive the project list from persisted open_tabs (deduplicated)
        let mut projects: Vec<FavoriteProject> = Vec::new();
        for tab in &config.jira.open_tabs {
            let (key, name) = match tab {
                OpenTab::List { project_key, project_name, .. } => (project_key.clone(), project_name.clone()),
                OpenTab::Board { project_key, board_name, .. } => (project_key.clone(), board_name.clone()),
            };
            if !projects.iter().any(|p| p.key == key) {
                projects.push(FavoriteProject { key, name });
            }
        }

        let logged_in = config.auth.token.is_some();
        let user_email = config.auth.email.clone();

        let current_project_key = projects.first().map(|p| p.key.clone()).unwrap_or_default();
        let current_project_name = projects.first().map(|p| p.name.clone()).unwrap_or_default();

        let mut next_list_id: u64 = 1;
        let mut list_tabs: Vec<ListTab> = Vec::new();
        let mut board_tabs: Vec<BoardTab> = Vec::new();

        for open_tab in config.jira.open_tabs.iter().filter(|t| match t {
            OpenTab::List { project_key, .. } => project_key == &current_project_key,
            OpenTab::Board { project_key, .. } => project_key == &current_project_key,
        }) {
            match open_tab {
                OpenTab::List { id, project_key, project_name } => {
                    list_tabs.push(ListTab {
                        id: *id,
                        project_key: project_key.clone(),
                        project_name: project_name.clone(),
                        issues: Vec::new(),
                        loading: true,
                        error: None,
                        nav: TableNav::default(),
                        filter: None,
                        statuses: Vec::new(),
                    });
                    if *id >= next_list_id {
                        next_list_id = id + 1;
                    }
                }
                OpenTab::Board { board_id, board_name, .. } => {
                    board_tabs.push(BoardTab {
                        board_id: *board_id,
                        board_name: board_name.clone(),
                        columns: Vec::new(),
                        col_scroll: Vec::new(),
                        loading: true,
                        error: None,
                    });
                }
            }
        }

        // If no list tabs persisted, create the default one
        if list_tabs.is_empty() && !current_project_key.is_empty() {
            list_tabs.push(ListTab {
                id: next_list_id,
                project_key: current_project_key.clone(),
                project_name: current_project_name.clone(),
                issues: Vec::new(),
                loading: false,
                error: None,
                nav: TableNav::default(),
                filter: None,
                statuses: Vec::new(),
            });
            next_list_id += 1;
        }

        let first_list_id = list_tabs.first().map(|t| t.id).unwrap_or(0);
        let subdomain = config
            .jira
            .base_url
            .as_deref()
            .and_then(|u| u.strip_prefix("https://"))
            .and_then(|u| u.strip_suffix(".atlassian.net"))
            .unwrap_or("")
            .to_string();

        Self {
            running: true,
            theme,
            header_bg_soft,
            active_tab: Tab::List(first_list_id),
            projects,
            selected_project: 0,
            click_regions: ClickRegions::default(),
            focus: FocusLayer::Main,

            logged_in,
            user_display_name: None,
            user_email: user_email.clone(),
            auth_open: false,
            auth_field: AuthField::Subdomain,
            subdomain_input: subdomain,
            email_input: user_email.unwrap_or_default(),
            token_input: String::new(),
            is_validating: false,
            auth_error: None,

            list_tabs,
            next_list_id,
            column_order: Vec::new(),

            detail_open: false,
            detail_tab: 0,
            detail_tab_areas: Vec::new(),
            detail_issue: None,
            detail_description: None,
            detail_comments: Vec::new(),
            detail_changelog: Vec::new(),
            detail_metadata: None,
            detail_height: 0,
            detail_scroll: 0,
            detail_max_scroll: 0,
            detail_close_area: None,
            detail_resize_area: None,
            detail_url_area: None,
            detail_dragging: false,
            detail_editing: false,
            detail_editor: SimpleEditor::default(),
            detail_transitions: Vec::new(),
            detail_transition_open: false,
            detail_transition_selected: 0,
            detail_transition_btn_area: None,

            board_tabs,

            find_modal_open: false,
            find_input: String::new(),
            find_results: Vec::new(),
            find_selected: 0,
            find_loading: false,

            find_board_panel_open: false,
            find_panel_project: None,
            find_panel_boards: Vec::new(),
            find_panel_selected: Vec::new(),
            find_panel_cursor: 0,
            find_panel_loading: false,

            message_tx,
            http_client: reqwest::Client::new(),

            settings_open: false,
            settings_selected: 0,
            settings_tab_areas: Vec::new(),
            settings_theme_areas: Vec::new(),
            settings_header_soft_area: None,
            settings_header_hard_area: None,
            settings_board_on_area: None,
            settings_board_off_area: None,
            settings_board_backlog_on_area: None,
            settings_board_backlog_off_area: None,
            settings_apply_area: None,
            settings_close_area: None,
            board_hide_subtasks: config.ui.board_hide_subtasks.unwrap_or(false),
            board_hide_backlog_col: config.ui.board_hide_backlog_col.unwrap_or(false),
            settings_board_field: 0,
            mouse_pos: (0, 0),
            board_content_area: None,
            theme_selected,
            theme_confirmed: theme_selected,
            header_bg_confirmed: header_bg_soft,

            config,
        }
    }

    pub fn handle_event(&mut self, event: AppEvent) -> Result<()> {
        match event {
            AppEvent::Key(key) => self.handle_key(key),
            AppEvent::Mouse(mouse) => self.handle_mouse(mouse),
            AppEvent::Message(msg) => self.handle_message(msg),
            AppEvent::Tick => {}
            AppEvent::Resize => {}
        }
        Ok(())
    }

    fn handle_message(&mut self, msg: AppMessage) {
        match msg {
            AppMessage::TokenValidated(Ok(user)) => {
                self.is_validating = false;
                self.logged_in = true;
                self.user_display_name = Some(user.display_name);
                self.user_email = user.email.clone();
                self.config.auth.token = Some(self.token_input.clone());
                self.config.auth.email = Some(self.email_input.clone());
                let _ = config::save_config(&self.config);
                self.auth_open = false;
                self.auth_error = None;
                self.focus = FocusLayer::Main;
                if !self.projects.is_empty() {
                    self.load_all_list_tabs();
                }
            }
            AppMessage::TokenValidated(Err(e)) => {
                self.is_validating = false;
                self.auth_error = Some(e.to_string());
            }
            AppMessage::BacklogLoaded(tab_id, Ok(issues)) => {
                let col_order = self.column_order.clone();
                if let Some(tab) = self.list_tabs.iter_mut().find(|t| t.id == tab_id) {
                    let mut statuses: Vec<String> = Vec::new();
                    for issue in &issues {
                        let name = &issue.fields.status.name;
                        if !statuses.contains(name) {
                            statuses.push(name.clone());
                        }
                    }
                    if !col_order.is_empty() {
                        statuses.sort_by_key(|s| {
                            col_order
                                .iter()
                                .position(|c| c.eq_ignore_ascii_case(s))
                                .unwrap_or(usize::MAX)
                        });
                    }
                    tab.statuses = statuses;
                    tab.issues = issues;
                    tab.loading = false;
                    tab.nav.reset();
                }
            }
            AppMessage::ColumnOrderLoaded(Ok(columns)) => {
                let project_key = self
                    .projects
                    .get(self.selected_project)
                    .map(|p| p.key.as_str())
                    .unwrap_or("");
                config::save_column_order_cache(project_key, &columns);
                self.column_order = columns.clone();
                for tab in &mut self.list_tabs {
                    if !tab.statuses.is_empty() {
                        tab.statuses.sort_by_key(|s| {
                            columns
                                .iter()
                                .position(|c| c.eq_ignore_ascii_case(s))
                                .unwrap_or(usize::MAX)
                        });
                    }
                }
            }
            AppMessage::IssueDetailLoaded(key, Ok((desc, metadata))) => {
                if self.detail_issue.as_ref().map(|i| &i.key) == Some(&key) {
                    self.detail_description = Some(desc);
                    self.detail_metadata = Some(metadata);
                }
            }
            AppMessage::IssueDetailLoaded(_, Err(_)) => {}
            AppMessage::TransitionsLoaded(key, Ok(transitions)) => {
                if self.detail_issue.as_ref().map(|i| &i.key) == Some(&key) {
                    self.detail_transitions = transitions;
                }
            }
            AppMessage::TransitionsLoaded(_, Err(_)) => {}
            AppMessage::CommentsLoaded(key, Ok(comments)) => {
                if self.detail_issue.as_ref().map(|i| &i.key) == Some(&key) {
                    self.detail_comments = comments;
                }
            }
            AppMessage::CommentsLoaded(_, Err(_)) => {}
            AppMessage::ChangelogLoaded(key, Ok(entries)) => {
                if self.detail_issue.as_ref().map(|i| &i.key) == Some(&key) {
                    self.detail_changelog = entries;
                }
            }
            AppMessage::ChangelogLoaded(_, Err(_)) => {}
            AppMessage::DescriptionUpdated(key, Ok(())) => {
                if self.detail_issue.as_ref().map(|i| &i.key) == Some(&key) {
                    self.detail_editing = false;
                }
            }
            AppMessage::DescriptionUpdated(_, Err(_)) => {
                self.detail_editing = false;
            }
            AppMessage::TransitionDone(key, Ok(())) => {
                if self.detail_issue.as_ref().map(|i| &i.key) == Some(&key) {
                    self.detail_transition_open = false;
                    self.load_issue_detail(&key.clone());
                    self.reload_all_list_tabs();
                }
            }
            AppMessage::TransitionDone(_, Err(_)) => {}
            AppMessage::ColumnOrderLoaded(Err(_)) => {}
            AppMessage::BacklogLoaded(tab_id, Err(e)) => {
                if let Some(tab) = self.list_tabs.iter_mut().find(|t| t.id == tab_id) {
                    tab.loading = false;
                    tab.error = Some(e.to_string());
                }
            }
            AppMessage::BoardsForFindLoaded(project_key, Ok(boards)) => {
                if self.find_panel_project.as_ref().map(|p| &p.key) == Some(&project_key) {
                    let total = boards.len() + 1; // +1 for list tab
                    self.find_panel_boards = boards;
                    self.find_panel_selected = vec![false; total];
                    self.find_panel_loading = false;
                }
            }
            AppMessage::BoardsForFindLoaded(_, Err(_)) => {
                self.find_panel_loading = false;
            }
            AppMessage::BoardDataLoaded(board_id, Ok((cfg, issues))) => {
                if let Some(tab) = self.board_tabs.iter_mut().find(|t| t.board_id == board_id) {
                    tab.columns = cfg
                        .column_config
                        .columns
                        .into_iter()
                        .map(|col| {
                            let status_ids: Vec<String> =
                                col.statuses.into_iter().map(|s| s.id).collect();
                            BoardColumn {
                                name: col.name,
                                status_ids,
                                issues: Vec::new(),
                            }
                        })
                        .collect();

                    for issue in &issues {
                        let issue_status_id = issue.fields.status.id.clone().unwrap_or_default();
                        let placed = tab.columns.iter_mut().any(|col| {
                            if col.status_ids.contains(&issue_status_id) {
                                col.issues.push(issue.clone());
                                true
                            } else {
                                false
                            }
                        });
                        if !placed {
                            let status_name = &issue.fields.status.name;
                            let _ = tab.columns.iter_mut().any(|col| {
                                if col.name.eq_ignore_ascii_case(status_name) {
                                    col.issues.push(issue.clone());
                                    true
                                } else {
                                    false
                                }
                            });
                        }
                    }


                    tab.col_scroll = vec![0; tab.columns.len()];
                    tab.loading = false;
                    tab.error = None;
                }
            }
            AppMessage::BoardDataLoaded(board_id, Err(e)) => {
                if let Some(tab) = self.board_tabs.iter_mut().find(|t| t.board_id == board_id) {
                    tab.loading = false;
                    tab.error = Some(e.to_string());
                }
            }
            AppMessage::SearchResults(Ok(projects)) => {
                self.find_results = projects
                    .into_iter()
                    .map(|p| FindProject { key: p.key, name: p.name })
                    .collect();
                self.find_loading = false;
                self.find_selected = 0;
            }
            AppMessage::SearchResults(Err(_)) => {
                self.find_loading = false;
            }
        }
    }

    fn handle_key(&mut self, key: KeyEvent) {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            self.running = false;
            return;
        }

        if self.detail_editing {
            self.handle_editor_key(key);
            return;
        }

        match self.focus {
            FocusLayer::Auth => self.handle_auth_key(key),
            FocusLayer::Settings => self.handle_settings_key(key),
            FocusLayer::Find => self.handle_find_key(key),
            FocusLayer::FindBoardPanel => self.handle_find_board_panel_key(key),


            FocusLayer::Main => self.handle_main_key(key),
        }
    }

    fn handle_main_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') => self.running = false,
            KeyCode::Tab => self.next_tab(),
            KeyCode::BackTab => self.prev_tab(),
            KeyCode::Char(c @ '1'..='9') => {
                let idx = (c as usize) - ('1' as usize);
                let all = self.all_tab_ids();
                if let Some(tab) = all.get(idx) {
                    self.active_tab = tab.clone();
                }
            }
            KeyCode::Char('f') => self.open_find(),
            KeyCode::Char('r') => self.refresh_active_tab(),
            KeyCode::Char(',') => self.open_settings(),
            KeyCode::Char('x') => self.close_active_tab(),
            KeyCode::Enter if matches!(self.active_tab, Tab::List(_)) && !self.detail_open => {
                self.open_detail_from_backlog();
            }
            KeyCode::Esc if self.detail_transition_open => {
                self.detail_transition_open = false;
            }
            KeyCode::Esc if self.detail_open => {
                self.detail_open = false;
                self.detail_issue = None;
                self.detail_description = None;
                self.detail_scroll = 0;
            }
            KeyCode::Char('t') if self.detail_open && !self.detail_transitions.is_empty() => {
                self.detail_transition_open = !self.detail_transition_open;
                self.detail_transition_selected = 0;
            }
            KeyCode::Char('e') if self.detail_open && self.detail_tab == 0 => {
                let desc = self.detail_description.clone().unwrap_or_default();
                self.detail_editor.load(&desc);
                self.detail_editing = true;
            }
            KeyCode::Right | KeyCode::Char('l') if self.detail_open && !self.detail_transition_open => {
                self.detail_tab = (self.detail_tab + 1) % 3;
                self.detail_scroll = 0;
            }
            KeyCode::Left | KeyCode::Char('h') if self.detail_open && !self.detail_transition_open => {
                self.detail_tab = (self.detail_tab + 2) % 3;
                self.detail_scroll = 0;
            }
            KeyCode::Down | KeyCode::Char('j') if self.detail_transition_open => {
                if self.detail_transition_selected < self.detail_transitions.len().saturating_sub(1) {
                    self.detail_transition_selected += 1;
                }
            }
            KeyCode::Up | KeyCode::Char('k') if self.detail_transition_open => {
                if self.detail_transition_selected > 0 {
                    self.detail_transition_selected -= 1;
                }
            }
            KeyCode::Enter if self.detail_transition_open => {
                if let Some(transition) = self.detail_transitions.get(self.detail_transition_selected).cloned() {
                    if let Some(ref issue) = self.detail_issue.clone() {
                        self.do_transition(&issue.key, &transition.id);
                    }
                }
            }
            KeyCode::Down | KeyCode::Char('j') if self.detail_open => {
                if self.detail_scroll < self.detail_max_scroll {
                    self.detail_scroll = self.detail_scroll.saturating_add(1).min(self.detail_max_scroll);
                }
            }
            KeyCode::Up | KeyCode::Char('k') if self.detail_open => {
                self.detail_scroll = self.detail_scroll.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') if matches!(self.active_tab, Tab::List(_)) => {
                let count = self.filtered_backlog_count();
                if let Some(tab) = self.active_list_tab_mut() {
                    tab.nav.move_down(count);
                }
            }
            KeyCode::Up | KeyCode::Char('k') if matches!(self.active_tab, Tab::List(_)) => {
                let count = self.filtered_backlog_count();
                if let Some(tab) = self.active_list_tab_mut() {
                    tab.nav.move_up(count);
                }
            }
            _ => {}
        }
    }

    fn handle_settings_key(&mut self, key: KeyEvent) {
        const NUM_TABS: usize = 4;
        match key.code {
            KeyCode::Esc => {
                self.theme = theme::ALL_THEMES[self.theme_confirmed];
                self.theme_selected = self.theme_confirmed;
                self.header_bg_soft = self.header_bg_confirmed;
                self.settings_open = false;
                self.focus = FocusLayer::Main;
            }
            KeyCode::Tab | KeyCode::Right | KeyCode::Char('l') => {
                self.settings_selected = (self.settings_selected + 1) % NUM_TABS;
            }
            KeyCode::BackTab | KeyCode::Left | KeyCode::Char('h') => {
                self.settings_selected = (self.settings_selected + NUM_TABS - 1) % NUM_TABS;
            }
            KeyCode::Up | KeyCode::Char('k') if self.settings_selected == 0 => {
                if self.theme_selected > 0 {
                    self.theme_selected -= 1;
                    self.theme = theme::ALL_THEMES[self.theme_selected];
                }
            }
            KeyCode::Down | KeyCode::Char('j') if self.settings_selected == 0 => {
                if self.theme_selected < theme::ALL_THEMES.len().saturating_sub(1) {
                    self.theme_selected += 1;
                    self.theme = theme::ALL_THEMES[self.theme_selected];
                }
            }
            KeyCode::Char(' ') if self.settings_selected == 1 => {
                self.header_bg_soft = !self.header_bg_soft;
            }
            KeyCode::Up | KeyCode::Char('k') if self.settings_selected == 2 => {
                if self.settings_board_field > 0 {
                    self.settings_board_field -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') if self.settings_selected == 2 => {
                if self.settings_board_field < 1 {
                    self.settings_board_field += 1;
                }
            }
            KeyCode::Char(' ') if self.settings_selected == 2 => {
                match self.settings_board_field {
                    0 => self.board_hide_subtasks = !self.board_hide_subtasks,
                    _ => self.board_hide_backlog_col = !self.board_hide_backlog_col,
                }
            }
            KeyCode::Enter => {
                self.apply_settings();
            }
            _ => {}
        }
    }

    fn open_settings(&mut self) {
        self.settings_open = true;
        self.focus = FocusLayer::Settings;
    }

    fn open_find(&mut self) {
        self.find_modal_open = true;
        self.find_input.clear();
        self.find_results.clear();
        self.find_selected = 0;
        self.focus = FocusLayer::Find;
    }

    fn handle_find_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.find_modal_open = false;
                self.focus = FocusLayer::Main;
            }
            KeyCode::Enter => {
                if self.find_results.is_empty() {
                    self.find_loading = true;
                    self.search_projects();
                } else if let Some(project) = self.find_results.get(self.find_selected).cloned() {
                    self.open_find_board_panel(project);
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.find_selected > 0 {
                    self.find_selected -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.find_selected < self.find_results.len().saturating_sub(1) {
                    self.find_selected += 1;
                }
            }
            KeyCode::Backspace => {
                self.find_input.pop();
                self.find_results.clear();
            }
            KeyCode::Char(c) => {
                self.find_input.push(c);
                self.find_results.clear();
            }
            _ => {}
        }
    }

    fn open_find_board_panel(&mut self, project: FindProject) {
        self.find_board_panel_open = true;
        self.find_panel_project = Some(project.clone());
        self.find_panel_boards.clear();
        self.find_panel_selected.clear();
        self.find_panel_cursor = 0;
        self.find_panel_loading = true;
        self.focus = FocusLayer::FindBoardPanel;
        self.load_boards_for_find_project(project.key);
    }

    fn handle_find_board_panel_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.find_board_panel_open = false;
                self.find_panel_project = None;
                self.focus = FocusLayer::Find;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.find_panel_cursor > 0 {
                    self.find_panel_cursor -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let total = self.find_panel_boards.len() + 1; // +1 for the List tab item
                if self.find_panel_cursor < total.saturating_sub(1) {
                    self.find_panel_cursor += 1;
                }
            }
            KeyCode::Char(' ') => {
                self.toggle_find_panel_item(self.find_panel_cursor);
            }
            KeyCode::Enter => {
                // Toggle current item then confirm
                self.toggle_find_panel_item(self.find_panel_cursor);
                self.confirm_find_panel_selection();
            }
            KeyCode::Char('a') => {
                self.confirm_find_panel_selection();
            }
            _ => {}
        }
    }

    fn toggle_find_panel_item(&mut self, idx: usize) {
        let total = self.find_panel_boards.len() + 1;
        if self.find_panel_selected.len() < total {
            self.find_panel_selected.resize(total, false);
        }
        if idx < total {
            self.find_panel_selected[idx] = !self.find_panel_selected[idx];
        }
    }

    fn confirm_find_panel_selection(&mut self) {
        let (project_key, project_name) = match self.find_panel_project.clone() {
            Some(p) => (p.key, p.name),
            None => return,
        };

        // Ensure selected vec is sized correctly (1 list item + boards)
        let total = self.find_panel_boards.len() + 1;
        if self.find_panel_selected.len() < total {
            self.find_panel_selected.resize(total, false);
        }

        // If nothing selected, default to just the list tab (switch project only)
        let any_selected = self.find_panel_selected.iter().any(|&s| s);

        // Collect boards to open (indices start at 1, idx 0 = list tab)
        let boards_to_add: Vec<JiraBoard> = self
            .find_panel_boards
            .iter()
            .enumerate()
            .filter(|(i, _)| self.find_panel_selected.get(i + 1).copied().unwrap_or(false))
            .map(|(_, b)| b.clone())
            .collect();

        // Register project if not known yet (for column_order / load_column_order)
        if !self.projects.iter().any(|p| p.key == project_key) {
            self.projects.push(FavoriteProject { key: project_key.clone(), name: project_name.clone() });
        }

        let want_list = !any_selected || self.find_panel_selected.get(0).copied().unwrap_or(false);

        // Always just add tabs — never reset existing ones
        if want_list {
            self.add_list_tab(project_key.clone(), project_name);
        }
        for board in boards_to_add {
            self.add_board_tab(board);
        }

        // Close modals
        self.find_board_panel_open = false;
        self.find_modal_open = false;
        self.find_panel_project = None;
        self.focus = FocusLayer::Main;
    }

    fn load_boards_for_find_project(&self, project_key: String) {
        let tx = self.message_tx.clone();
        let client = self.http_client.clone();
        let email = self.config.auth.email.clone().unwrap_or_default();
        let token = self.config.auth.token.clone().unwrap_or_default();
        let base_url = self
            .config
            .jira
            .base_url
            .clone()
            .unwrap_or_else(|| "https://jira.atlassian.net".into());
        let key_clone = project_key.clone();

        tokio::spawn(async move {
            let provider = JiraProvider::new(client, base_url, email, token);
            let result = provider.get_boards(&key_clone).await;
            let _ = tx.send(AppMessage::BoardsForFindLoaded(project_key, result));
        });
    }

    fn search_projects(&self) {
        if self.find_input.is_empty() {
            return;
        }
        let tx = self.message_tx.clone();
        let client = self.http_client.clone();
        let email = self.config.auth.email.clone().unwrap_or_default();
        let token = self.config.auth.token.clone().unwrap_or_default();
        let base_url = self
            .config
            .jira
            .base_url
            .clone()
            .unwrap_or_else(|| "https://jira.atlassian.net".into());
        let query = self.find_input.clone();

        tokio::spawn(async move {
            let provider = JiraProvider::new(client, base_url, email, token);
            let result = provider.search_projects(&query).await;
            let _ = tx.send(AppMessage::SearchResults(result));
        });
    }

    pub fn load_column_order(&mut self) {
        if !self.column_order.is_empty() {
            return;
        }
        let project = match self.projects.get(self.selected_project) {
            Some(p) => p.clone(),
            None => return,
        };
        // Try disk cache first
        if let Some(cached) = config::load_column_order_cache(&project.key) {
            self.column_order = cached;
            return;
        }
        let tx = self.message_tx.clone();
        let client = self.http_client.clone();
        let email = self.config.auth.email.clone().unwrap_or_default();
        let token = self.config.auth.token.clone().unwrap_or_default();
        let base_url = self
            .config
            .jira
            .base_url
            .clone()
            .unwrap_or_else(|| "https://jira.atlassian.net".into());

        tokio::spawn(async move {
            let provider = JiraProvider::new(client, base_url, email, token);
            let boards = match provider.get_boards(&project.key).await {
                Ok(b) => b,
                Err(e) => {
                    let _ = tx.send(AppMessage::ColumnOrderLoaded(Err(e)));
                    return;
                }
            };
            if let Some(board) = boards.first() {
                match provider.get_board_config(board.id).await {
                    Ok(cfg) => {
                        let columns: Vec<String> =
                            cfg.column_config.columns.iter().map(|c| c.name.clone()).collect();
                        let _ = tx.send(AppMessage::ColumnOrderLoaded(Ok(columns)));
                    }
                    Err(e) => {
                        let _ = tx.send(AppMessage::ColumnOrderLoaded(Err(e)));
                    }
                }
            }
        });
    }

    fn load_backlog_for_tab(&self, tab_id: u64, project_key: String) {
        let tx = self.message_tx.clone();
        let client = self.http_client.clone();
        let email = self.config.auth.email.clone().unwrap_or_default();
        let token = self.config.auth.token.clone().unwrap_or_default();
        let base_url = self
            .config
            .jira
            .base_url
            .clone()
            .unwrap_or_else(|| "https://jira.atlassian.net".into());

        tokio::spawn(async move {
            let provider = JiraProvider::new(client, base_url, email, token);
            let result = provider.get_backlog(&project_key).await;
            let _ = tx.send(AppMessage::BacklogLoaded(tab_id, result));
        });
    }

    pub fn load_all_list_tabs(&mut self) {
        let tabs: Vec<(u64, String)> = self
            .list_tabs
            .iter_mut()
            .map(|t| {
                t.loading = true;
                (t.id, t.project_key.clone())
            })
            .collect();
        for (id, key) in tabs {
            self.load_backlog_for_tab(id, key);
        }
    }

    fn reload_all_list_tabs(&mut self) {
        let tabs: Vec<(u64, String)> = self
            .list_tabs
            .iter_mut()
            .map(|t| {
                t.loading = true;
                t.error = None;
                (t.id, t.project_key.clone())
            })
            .collect();
        for (id, key) in tabs {
            self.load_backlog_for_tab(id, key);
        }
    }

    fn add_list_tab(&mut self, project_key: String, project_name: String) {
        let id = self.next_list_id;
        self.next_list_id += 1;
        self.list_tabs.push(ListTab {
            id,
            project_key: project_key.clone(),
            project_name,
            issues: Vec::new(),
            loading: true,
            error: None,
            nav: TableNav::default(),
            filter: None,
            statuses: Vec::new(),
        });
        self.active_tab = Tab::List(id);
        self.save_open_tabs();
        self.load_backlog_for_tab(id, project_key);
    }

    fn open_auth(&mut self) {
        self.auth_open = true;
        self.token_input.clear();
        self.auth_error = None;
        self.auth_field = AuthField::Subdomain;
        self.focus = FocusLayer::Auth;
    }

    fn logout(&mut self) {
        self.logged_in = false;
        self.user_display_name = None;
        self.user_email = None;
        self.config.auth.token = None;
        self.config.auth.email = None;
        let _ = config::save_config(&self.config);
    }

    fn handle_auth_key(&mut self, key: KeyEvent) {
        if self.is_validating {
            return;
        }
        match key.code {
            KeyCode::Esc => {
                self.auth_open = false;
                self.auth_error = None;
                self.focus = FocusLayer::Main;
            }
            KeyCode::Tab => {
                self.auth_field = match self.auth_field {
                    AuthField::Subdomain => AuthField::Email,
                    AuthField::Email => AuthField::Token,
                    AuthField::Token => AuthField::Subdomain,
                };
            }
            KeyCode::BackTab => {
                self.auth_field = match self.auth_field {
                    AuthField::Subdomain => AuthField::Token,
                    AuthField::Email => AuthField::Subdomain,
                    AuthField::Token => AuthField::Email,
                };
            }
            KeyCode::Enter => self.submit_token(),
            KeyCode::Backspace => match self.auth_field {
                AuthField::Subdomain => { self.subdomain_input.pop(); }
                AuthField::Email => { self.email_input.pop(); }
                AuthField::Token => { self.token_input.pop(); }
            },
            KeyCode::Char(c) => match self.auth_field {
                AuthField::Subdomain => self.subdomain_input.push(c),
                AuthField::Email => self.email_input.push(c),
                AuthField::Token => self.token_input.push(c),
            },
            _ => {}
        }
    }

    fn submit_token(&mut self) {
        if self.subdomain_input.is_empty() {
            self.auth_error = Some("Subdomain is required".into());
            return;
        }
        if self.email_input.is_empty() || self.token_input.is_empty() {
            self.auth_error = Some("Email and token are required".into());
            return;
        }

        self.is_validating = true;
        self.auth_error = None;

        let base_url = format!("https://{}.atlassian.net", self.subdomain_input);

        let tx = self.message_tx.clone();
        let client = self.http_client.clone();
        let email = self.email_input.clone();
        let token = self.token_input.clone();
        let url = base_url.clone();

        self.config.jira.base_url = Some(base_url);

        tokio::spawn(async move {
            let provider = JiraProvider::new(client, url, email, token);
            let result = provider.get_current_user().await;
            let _ = tx.send(AppMessage::TokenValidated(result));
        });
    }

    fn apply_settings(&mut self) {
        self.theme_confirmed = self.theme_selected;
        self.config.ui.theme = Some(self.theme.name.to_string());
        self.header_bg_confirmed = self.header_bg_soft;
        self.config.ui.header_bg =
            Some(if self.header_bg_soft { "soft" } else { "hard" }.to_string());
        self.config.ui.board_hide_subtasks = Some(self.board_hide_subtasks);
        self.config.ui.board_hide_backlog_col = Some(self.board_hide_backlog_col);
        let _ = config::save_config(&self.config);
        self.settings_open = false;
        self.focus = FocusLayer::Main;
    }

    fn handle_mouse(&mut self, mouse: MouseEvent) {
        let pos = (mouse.column, mouse.row);

        if let MouseEventKind::Moved = mouse.kind {
            self.mouse_pos = pos;
            return;
        }

        if mouse.kind == MouseEventKind::Drag(MouseButton::Left) {
            if self.detail_dragging {
                if let Some(resize_area) = self.detail_resize_area {
                    let panel_bottom = resize_area.y + self.detail_height;
                    let new_height = panel_bottom.saturating_sub(pos.1);
                    self.detail_height = new_height.max(6).min(panel_bottom.saturating_sub(6));
                }
            }
            return;
        }

        if mouse.kind == MouseEventKind::Up(MouseButton::Left) {
            self.detail_dragging = false;
            return;
        }

        match mouse.kind {
            MouseEventKind::ScrollDown if self.focus == FocusLayer::Main && self.detail_open => {
                let in_detail = self.detail_resize_area
                    .map(|r| pos.1 >= r.y)
                    .unwrap_or(false);
                if in_detail {
                    if self.detail_scroll < self.detail_max_scroll {
                        self.detail_scroll = self.detail_scroll.saturating_add(2).min(self.detail_max_scroll);
                    }
                } else if matches!(self.active_tab, Tab::List(_)) {
                    let count = self.filtered_backlog_count();
                    if let Some(tab) = self.active_list_tab_mut() { tab.nav.scroll_down(count); }
                } else if let Tab::Board(id) = self.active_tab {
                    self.scroll_board_column(id, mouse.column, 3);
                }
                return;
            }
            MouseEventKind::ScrollUp if self.focus == FocusLayer::Main && self.detail_open => {
                let in_detail = self.detail_resize_area
                    .map(|r| pos.1 >= r.y)
                    .unwrap_or(false);
                if in_detail {
                    self.detail_scroll = self.detail_scroll.saturating_sub(2);
                } else if matches!(self.active_tab, Tab::List(_)) {
                    if let Some(tab) = self.active_list_tab_mut() { tab.nav.scroll_up(); }
                } else if let Tab::Board(id) = self.active_tab {
                    self.scroll_board_column(id, mouse.column, -3);
                }
                return;
            }
            MouseEventKind::ScrollDown if self.focus == FocusLayer::Main && matches!(self.active_tab, Tab::List(_)) => {
                let count = self.filtered_backlog_count();
                if let Some(tab) = self.active_list_tab_mut() { tab.nav.scroll_down(count); }
                return;
            }
            MouseEventKind::ScrollUp if self.focus == FocusLayer::Main && matches!(self.active_tab, Tab::List(_)) => {
                if let Some(tab) = self.active_list_tab_mut() { tab.nav.scroll_up(); }
                return;
            }
            MouseEventKind::ScrollDown if self.focus == FocusLayer::Main => {
                if let Tab::Board(id) = self.active_tab {
                    self.scroll_board_column(id, mouse.column, 3);
                }
                return;
            }
            MouseEventKind::ScrollUp if self.focus == FocusLayer::Main => {
                if let Tab::Board(id) = self.active_tab {
                    self.scroll_board_column(id, mouse.column, -3);
                }
                return;
            }
            _ => {}
        }

        if let MouseEventKind::Down(MouseButton::Left) = mouse.kind {
            if self.settings_open {
                self.handle_settings_mouse(pos);
                return;
            }

            if self.find_modal_open {
                self.handle_find_mouse(pos);
                return;
            }

            if self.detail_open {
                if self.detail_transition_open {
                    // Click on button toggles off
                    if hit(pos, self.detail_transition_btn_area) {
                        self.detail_transition_open = false;
                        return;
                    }
                    // Click anywhere else closes dropdown (click outside)
                    self.detail_transition_open = false;
                    return;
                }
                if hit(pos, self.detail_close_area) {
                    self.detail_open = false;
                    self.detail_issue = None;
                    self.detail_description = None;
                    self.detail_scroll = 0;
                    return;
                }
                if hit(pos, self.detail_transition_btn_area) {
                    if !self.detail_transitions.is_empty() {
                        self.detail_transition_open = true;
                        self.detail_transition_selected = 0;
                    }
                    return;
                }
                if hit(pos, self.detail_resize_area) {
                    self.detail_dragging = true;
                    return;
                }
                for (i, tab_area) in self.detail_tab_areas.iter().enumerate() {
                    if hit(pos, Some(*tab_area)) {
                        self.detail_tab = i;
                        self.detail_scroll = 0;
                        return;
                    }
                }
                if hit(pos, self.detail_url_area) {
                    if let Some(ref issue) = self.detail_issue {
                        let base_url = self
                            .config
                            .jira
                            .base_url
                            .as_deref()
                            .unwrap_or("https://jira.atlassian.net");
                        let url = format!("{}/browse/{}", base_url, issue.key);
                        let _ = open_url(&url);
                    }
                    return;
                }
            }

            let all_tabs = self.all_tab_ids();
            let mut tab_clicked = false;
            for (area, idx) in &self.click_regions.header.tab_areas {
                if hit(pos, Some(*area)) {
                    if let Some(tab) = all_tabs.get(*idx) {
                        self.active_tab = tab.clone();
                    }
                    tab_clicked = true;
                    break;
                }
            }
            if tab_clicked {
                // handled
            } else if hit(pos, self.click_regions.header.tab_add) {
                self.open_find();
            } else if hit(pos, self.click_regions.header.settings_link) {
                self.open_settings();
            } else if hit(pos, self.click_regions.header.login_link) {
                self.open_auth();
            } else if hit(pos, self.click_regions.header.logout_link) {
                self.logout();
            }

            // Board card clicks
            if let Tab::Board(_) = &self.active_tab {
                for (area, key) in &self.click_regions.board_cards.cards {
                    if hit(pos, Some(*area)) {
                        let issue = self
                            .board_tabs
                            .iter()
                            .flat_map(|t| t.columns.iter())
                            .flat_map(|c| c.issues.iter())
                            .find(|i| i.key == *key)
                            .cloned();
                        if let Some(issue) = issue {
                            self.open_detail_for_issue(&issue);
                        }
                        return;
                    }
                }
            }

            // Backlog row clicks
            if matches!(self.active_tab, Tab::List(_)) {
                for (i, area) in self.click_regions.backlog.row_areas.iter().enumerate() {
                    if hit(pos, Some(*area)) {
                        if let Some(tab) = self.active_list_tab_mut() {
                            tab.nav.selected = Some(tab.nav.offset + i);
                        }
                        self.open_detail_from_backlog();
                        return;
                    }
                }
            }

            // Backlog filter clicks
            if matches!(self.active_tab, Tab::List(_)) {
                let statuses: Vec<String> = self
                    .active_list_tab()
                    .map(|t| t.statuses.clone())
                    .unwrap_or_default();
                for (i, area) in self.click_regions.backlog.filter_areas.iter().enumerate() {
                    if hit(pos, Some(*area)) {
                        if let Some(tab) = self.active_list_tab_mut() {
                            if i == 0 {
                                tab.filter = None;
                            } else if let Some(status) = statuses.get(i - 1) {
                                tab.filter = Some(status.clone());
                            }
                            tab.nav.reset();
                        }
                        break;
                    }
                }
            }
        }
    }

    fn handle_find_mouse(&mut self, pos: (u16, u16)) {
        // Panel clicks take priority when panel is open
        if self.find_board_panel_open {
            let panel_areas = self.click_regions.find_modal.panel_item_areas.clone();
            for (i, area) in panel_areas.iter().enumerate() {
                if hit(pos, Some(*area)) {
                    self.find_panel_cursor = i;
                    self.toggle_find_panel_item(i);
                    return;
                }
            }
        }

        for (i, area) in self.click_regions.find_modal.result_areas.iter().enumerate() {
            if hit(pos, Some(*area)) {
                self.find_selected = i;
                if let Some(project) = self.find_results.get(i).cloned() {
                    self.open_find_board_panel(project);
                }
                return;
            }
        }
    }

    fn handle_settings_mouse(&mut self, pos: (u16, u16)) {
        for (i, area) in self.settings_tab_areas.iter().enumerate() {
            if hit(pos, Some(*area)) {
                self.settings_selected = i;
                return;
            }
        }

        if self.settings_selected == 0 {
            for (i, area) in self.settings_theme_areas.iter().enumerate() {
                if hit(pos, Some(*area)) {
                    self.theme_selected = i;
                    self.theme = theme::ALL_THEMES[i];
                    return;
                }
            }
        }

        if self.settings_selected == 1 {
            if hit(pos, self.settings_header_soft_area) {
                self.header_bg_soft = true;
                return;
            }
            if hit(pos, self.settings_header_hard_area) {
                self.header_bg_soft = false;
                return;
            }
        }

        if self.settings_selected == 2 {
            if hit(pos, self.settings_board_on_area) {
                self.board_hide_subtasks = true;
                return;
            }
            if hit(pos, self.settings_board_off_area) {
                self.board_hide_subtasks = false;
                return;
            }
            if hit(pos, self.settings_board_backlog_on_area) {
                self.board_hide_backlog_col = true;
                return;
            }
            if hit(pos, self.settings_board_backlog_off_area) {
                self.board_hide_backlog_col = false;
                return;
            }
        }

        if hit(pos, self.settings_apply_area) {
            self.apply_settings();
            return;
        }
        if hit(pos, self.settings_close_area) {
            self.theme = theme::ALL_THEMES[self.theme_confirmed];
            self.theme_selected = self.theme_confirmed;
            self.header_bg_soft = self.header_bg_confirmed;
            self.settings_open = false;
            self.focus = FocusLayer::Main;
        }
    }

    fn open_detail_from_backlog(&mut self) {
        let tab = match self.active_list_tab() {
            Some(t) => t,
            None => return,
        };
        let filtered: Vec<&JiraIssue> = tab
            .issues
            .iter()
            .filter(|issue| match &tab.filter {
                None => true,
                Some(f) => issue.fields.status.name == *f,
            })
            .collect();
        let selected = tab.nav.selected;
        if let Some(issue) = selected.and_then(|i| filtered.get(i)) {
            let issue = (*issue).clone();
            self.open_detail_for_issue(&issue);
        }
    }

    pub fn open_detail_for_issue(&mut self, issue: &JiraIssue) {
        let key = issue.key.clone();
        self.detail_issue = Some(issue.clone());
        self.detail_description = None;
        self.detail_comments.clear();
        self.detail_changelog.clear();
        self.detail_metadata = None;
        self.detail_open = true;
        self.detail_tab = 0;
        self.detail_height = 0;
        self.detail_scroll = 0;
        self.detail_transitions.clear();
        self.detail_transition_open = false;
        self.detail_transition_selected = 0;
        self.load_issue_detail(&key);
    }

    pub fn save_description(&self, issue_key: &str, text: String) {
        let tx = self.message_tx.clone();
        let client = self.http_client.clone();
        let email = self.config.auth.email.clone().unwrap_or_default();
        let token = self.config.auth.token.clone().unwrap_or_default();
        let base_url = self
            .config
            .jira
            .base_url
            .clone()
            .unwrap_or_else(|| "https://jira.atlassian.net".into());
        let key = issue_key.to_string();

        tokio::spawn(async move {
            let provider = JiraProvider::new(client, base_url, email, token);
            let result = provider.update_description(&key, &text).await;
            let _ = tx.send(AppMessage::DescriptionUpdated(key, result));
        });
    }

    fn handle_editor_key(&mut self, key: KeyEvent) {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('s') {
            let text = self.detail_editor.to_text();
            if let Some(ref issue) = self.detail_issue.clone() {
                self.detail_description = Some(text.clone());
                self.save_description(&issue.key, text);
            }
            self.detail_editing = false;
            return;
        }
        if key.code == KeyCode::Esc {
            self.detail_editing = false;
            return;
        }
        self.detail_editor.input(key);
    }

    fn load_issue_detail(&self, issue_key: &str) {
        let tx = self.message_tx.clone();
        let client = self.http_client.clone();
        let email = self.config.auth.email.clone().unwrap_or_default();
        let token = self.config.auth.token.clone().unwrap_or_default();
        let base_url = self
            .config
            .jira
            .base_url
            .clone()
            .unwrap_or_else(|| "https://jira.atlassian.net".into());
        let key = issue_key.to_string();

        tokio::spawn(async move {
            let provider = JiraProvider::new(client, base_url, email, token);
            let detail = provider.get_issue_detail(&key).await;
            let _ = tx.send(AppMessage::IssueDetailLoaded(key.clone(), detail));
            let transitions = provider.get_transitions(&key).await;
            let _ = tx.send(AppMessage::TransitionsLoaded(key.clone(), transitions));
            let comments = provider.get_comments(&key).await;
            let _ = tx.send(AppMessage::CommentsLoaded(key.clone(), comments));
            let changelog = provider.get_status_changelog(&key).await;
            let _ = tx.send(AppMessage::ChangelogLoaded(key, changelog));
        });
    }

    fn do_transition(&self, issue_key: &str, transition_id: &str) {
        let tx = self.message_tx.clone();
        let client = self.http_client.clone();
        let email = self.config.auth.email.clone().unwrap_or_default();
        let token = self.config.auth.token.clone().unwrap_or_default();
        let base_url = self
            .config
            .jira
            .base_url
            .clone()
            .unwrap_or_else(|| "https://jira.atlassian.net".into());
        let key = issue_key.to_string();
        let tid = transition_id.to_string();

        tokio::spawn(async move {
            let provider = JiraProvider::new(client, base_url, email, token);
            let result = provider.do_transition(&key, &tid).await;
            let _ = tx.send(AppMessage::TransitionDone(key, result));
        });
    }

    fn active_list_tab(&self) -> Option<&ListTab> {
        if let Tab::List(id) = self.active_tab {
            self.list_tabs.iter().find(|t| t.id == id)
        } else {
            None
        }
    }

    fn active_list_tab_mut(&mut self) -> Option<&mut ListTab> {
        if let Tab::List(id) = self.active_tab {
            self.list_tabs.iter_mut().find(|t| t.id == id)
        } else {
            None
        }
    }

    fn filtered_backlog_count(&self) -> usize {
        match self.active_list_tab() {
            Some(tab) => match &tab.filter {
                None => tab.issues.len(),
                Some(f) => tab.issues.iter().filter(|i| i.fields.status.name == *f).count(),
            },
            None => 0,
        }
    }

    fn scroll_board_column(&mut self, board_id: u64, mouse_x: u16, delta: i32) {
        let tab = match self.board_tabs.iter_mut().find(|t| t.board_id == board_id) {
            Some(t) => t,
            None => return,
        };
        if tab.columns.is_empty() || tab.col_scroll.is_empty() {
            return;
        }

        let visible_count = if self.board_hide_backlog_col {
            tab.columns.iter().filter(|c| !c.name.eq_ignore_ascii_case("backlog")).count()
        } else {
            tab.columns.len()
        };
        if visible_count == 0 {
            return;
        }

        let (area_x, area_width) = self
            .board_content_area
            .map(|a| (a.x, a.width))
            .unwrap_or((0, 80));
        let relative_x = mouse_x.saturating_sub(area_x) as usize;
        let col_width = (area_width as usize) / visible_count.max(1);
        let col_idx = if col_width > 0 { relative_x / col_width } else { 0 };
        let col_idx = col_idx.min(visible_count - 1);

        // Map visible index back to actual index
        let actual_idx = if self.board_hide_backlog_col {
            tab.columns
                .iter()
                .enumerate()
                .filter(|(_, c)| !c.name.eq_ignore_ascii_case("backlog"))
                .nth(col_idx)
                .map(|(i, _)| i)
        } else {
            Some(col_idx)
        };

        if let Some(idx) = actual_idx {
            if idx < tab.col_scroll.len() {
                if delta > 0 {
                    tab.col_scroll[idx] = tab.col_scroll[idx].saturating_add(delta as usize);
                } else {
                    tab.col_scroll[idx] = tab.col_scroll[idx].saturating_sub((-delta) as usize);
                }
            }
        }
    }

    fn next_tab(&mut self) {
        let tabs = self.all_tab_ids();
        let current_idx = tabs.iter().position(|t| *t == self.active_tab).unwrap_or(0);
        let next_idx = (current_idx + 1) % tabs.len();
        self.active_tab = tabs[next_idx].clone();
    }

    fn prev_tab(&mut self) {
        let tabs = self.all_tab_ids();
        let current_idx = tabs.iter().position(|t| *t == self.active_tab).unwrap_or(0);
        let prev_idx = if current_idx == 0 { tabs.len() - 1 } else { current_idx - 1 };
        self.active_tab = tabs[prev_idx].clone();
    }

    fn all_tab_ids(&self) -> Vec<Tab> {
        let mut tabs: Vec<Tab> = self.list_tabs.iter().map(|t| Tab::List(t.id)).collect();
        for bt in &self.board_tabs {
            tabs.push(Tab::Board(bt.board_id));
        }
        tabs
    }

    fn refresh_active_tab(&mut self) {
        match self.active_tab {
            Tab::List(id) => {
                let project_key = match self.list_tabs.iter_mut().find(|t| t.id == id) {
                    Some(tab) => {
                        tab.loading = true;
                        tab.error = None;
                        tab.project_key.clone()
                    }
                    None => return,
                };
                self.load_backlog_for_tab(id, project_key);
            }
            Tab::Board(id) => {
                let board_id = id;
                if let Some(tab) = self.board_tabs.iter_mut().find(|t| t.board_id == board_id) {
                    tab.loading = true;
                    tab.error = None;
                }
                self.load_board_data(board_id);
            }
        }
    }

    fn close_active_tab(&mut self) {
        match self.active_tab {
            Tab::List(id) => {
                // Don't close if it's the only list tab
                if self.list_tabs.len() <= 1 {
                    return;
                }
                let pos = self.list_tabs.iter().position(|t| t.id == id).unwrap_or(0);
                self.list_tabs.retain(|t| t.id != id);
                let new_tab = if pos > 0 {
                    self.list_tabs.get(pos - 1).map(|t| Tab::List(t.id))
                } else {
                    self.list_tabs.first().map(|t| Tab::List(t.id))
                };
                self.active_tab = new_tab.unwrap_or_else(|| Tab::Board(0));
                self.save_open_tabs();
            }
            Tab::Board(id) => {
                let board_id = id;
                self.board_tabs.retain(|t| t.board_id != board_id);
                let fallback = self.list_tabs.first().map(|t| Tab::List(t.id));
                self.active_tab = fallback.unwrap_or(Tab::Board(0));
                self.save_open_tabs();
            }
        }
    }

    fn add_board_tab(&mut self, board: JiraBoard) {
        if self.board_tabs.iter().any(|t| t.board_id == board.id) {
            self.active_tab = Tab::Board(board.id);
            return;
        }
        let tab = BoardTab {
            board_id: board.id,
            board_name: board.name.clone(),
            columns: Vec::new(),
            col_scroll: Vec::new(),
            loading: true,
            error: None,
        };
        self.board_tabs.push(tab);
        self.active_tab = Tab::Board(board.id);
        self.save_open_tabs();
        self.load_board_data(board.id);
    }

    fn save_open_tabs(&self) {
        let project_key = self
            .projects
            .get(self.selected_project)
            .map(|p| p.key.clone())
            .unwrap_or_default();
        let mut config = self.config.clone();
        config.jira.open_tabs.retain(|t| match t {
            OpenTab::List { project_key: pk, .. } => pk != &project_key,
            OpenTab::Board { project_key: pk, .. } => pk != &project_key,
        });
        for lt in &self.list_tabs {
            config.jira.open_tabs.push(OpenTab::List {
                project_key: project_key.clone(),
                project_name: lt.project_name.clone(),
                id: lt.id,
            });
        }
        for bt in &self.board_tabs {
            config.jira.open_tabs.push(OpenTab::Board {
                project_key: project_key.clone(),
                board_id: bt.board_id,
                board_name: bt.board_name.clone(),
            });
        }
        let _ = config::save_config(&config);
    }

    pub fn load_board_data(&self, board_id: u64) {
        let tx = self.message_tx.clone();
        let client = self.http_client.clone();
        let email = self.config.auth.email.clone().unwrap_or_default();
        let token = self.config.auth.token.clone().unwrap_or_default();
        let base_url = self
            .config
            .jira
            .base_url
            .clone()
            .unwrap_or_else(|| "https://jira.atlassian.net".into());

        tokio::spawn(async move {
            let provider = JiraProvider::new(client.clone(), base_url.clone(), email.clone(), token.clone());
            let config_result = provider.get_board_config(board_id).await;
            let issues_result = provider.get_board_issues(board_id).await;
            let result = match (config_result, issues_result) {
                (Ok(cfg), Ok(issues)) => Ok((cfg, issues)),
                (Err(e), _) | (_, Err(e)) => Err(e),
            };
            let _ = tx.send(AppMessage::BoardDataLoaded(board_id, result));
        });
    }
}

fn hit(pos: (u16, u16), area: Option<Rect>) -> bool {
    match area {
        Some(rect) => {
            pos.0 >= rect.x
                && pos.0 < rect.x + rect.width
                && pos.1 >= rect.y
                && pos.1 < rect.y + rect.height
        }
        None => false,
    }
}

fn open_url(url: &str) -> std::io::Result<()> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg(url).spawn()?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open").arg(url).spawn()?;
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd").args(["/C", "start", url]).spawn()?;
    }
    Ok(())
}
