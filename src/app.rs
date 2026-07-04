use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::prelude::Rect;
use tokio::sync::mpsc;

use crate::config;
use crate::config::types::{AppConfig, FavoriteProject, OpenBoard};
use crate::event::{AppEvent, AppMessage};
use crate::provider::jira::JiraProvider;
use crate::provider::types::{JiraBoard, JiraIssue};
use crate::table_nav::TableNav;
use crate::theme::{self, Theme};
use crate::ui::click_regions::ClickRegions;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tab {
    Backlog,
    Board(u64),
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
    BoardPicker,
    ProjectDropdown,
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
    pub is_favorite: bool,
}

pub struct App {
    pub running: bool,
    pub theme: &'static Theme,
    pub header_bg_soft: bool,
    pub active_tab: Tab,
    pub projects: Vec<FavoriteProject>,
    pub selected_project: usize,
    pub project_selector_open: bool,
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

    // Backlog
    pub backlog_issues: Vec<JiraIssue>,
    pub backlog_loading: bool,
    pub backlog_error: Option<String>,
    pub backlog_nav: TableNav,
    pub backlog_filter: Option<String>,
    pub backlog_statuses: Vec<String>,
    pub column_order: Vec<String>,

    // Detail panel
    pub detail_open: bool,
    pub detail_issue: Option<JiraIssue>,
    pub detail_description: Option<String>,
    pub detail_height: u16,
    pub detail_scroll: u16,
    pub detail_close_area: Option<Rect>,
    pub detail_resize_area: Option<Rect>,
    pub detail_url_area: Option<Rect>,
    pub detail_dragging: bool,

    // Board tabs
    pub board_tabs: Vec<BoardTab>,
    pub board_picker_open: bool,
    pub board_picker_boards: Vec<JiraBoard>,
    pub board_picker_selected: usize,
    pub board_picker_loading: bool,

    // Find modal
    pub find_modal_open: bool,
    pub find_input: String,
    pub find_results: Vec<FindProject>,
    pub find_selected: usize,
    pub find_loading: bool,

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

        let projects = config.jira.favorites.clone();
        let logged_in = config.auth.token.is_some();
        let user_email = config.auth.email.clone();

        let current_project_key = projects.first().map(|p| p.key.clone()).unwrap_or_default();
        let board_tabs: Vec<BoardTab> = config
            .jira
            .open_boards
            .iter()
            .filter(|b| b.project_key == current_project_key)
            .map(|b| BoardTab {
                board_id: b.board_id,
                board_name: b.board_name.clone(),
                columns: Vec::new(),
                col_scroll: Vec::new(),
                loading: true,
                error: None,
            })
            .collect();
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
            active_tab: Tab::Backlog,
            projects,
            selected_project: 0,
            project_selector_open: false,
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

            backlog_issues: Vec::new(),
            backlog_loading: false,
            backlog_error: None,
            backlog_nav: TableNav::default(),
            backlog_filter: None,
            backlog_statuses: Vec::new(),
            column_order: Vec::new(),

            detail_open: false,
            detail_issue: None,
            detail_description: None,
            detail_height: 0,
            detail_scroll: 0,
            detail_close_area: None,
            detail_resize_area: None,
            detail_url_area: None,
            detail_dragging: false,

            board_tabs,
            board_picker_open: false,
            board_picker_boards: Vec::new(),
            board_picker_selected: 0,
            board_picker_loading: false,

            find_modal_open: false,
            find_input: String::new(),
            find_results: Vec::new(),
            find_selected: 0,
            find_loading: false,

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
                    self.backlog_loading = true;
                    self.load_backlog();
                }
            }
            AppMessage::TokenValidated(Err(e)) => {
                self.is_validating = false;
                self.auth_error = Some(e.to_string());
            }
            AppMessage::BacklogLoaded(Ok(issues)) => {
                let mut statuses: Vec<String> = Vec::new();
                for issue in &issues {
                    let name = &issue.fields.status.name;
                    if !statuses.contains(name) {
                        statuses.push(name.clone());
                    }
                }

                if !self.column_order.is_empty() {
                    statuses.sort_by_key(|s| {
                        self.column_order
                            .iter()
                            .position(|c| c.eq_ignore_ascii_case(s))
                            .unwrap_or(usize::MAX)
                    });
                }

                self.backlog_statuses = statuses;
                self.backlog_issues = issues;
                self.backlog_loading = false;
                self.backlog_nav.reset();
            }
            AppMessage::ColumnOrderLoaded(Ok(columns)) => {
                let project_key = self
                    .projects
                    .get(self.selected_project)
                    .map(|p| p.key.as_str())
                    .unwrap_or("");
                config::save_column_order_cache(project_key, &columns);
                self.column_order = columns;
                if !self.backlog_statuses.is_empty() && !self.column_order.is_empty() {
                    self.backlog_statuses.sort_by_key(|s| {
                        self.column_order
                            .iter()
                            .position(|c| c.eq_ignore_ascii_case(s))
                            .unwrap_or(usize::MAX)
                    });
                }
            }
            AppMessage::ColumnOrderLoaded(Err(_)) => {}
            AppMessage::BacklogLoaded(Err(e)) => {
                self.backlog_loading = false;
                self.backlog_error = Some(e.to_string());
            }
            AppMessage::BoardsLoaded(Ok(boards)) => {
                self.board_picker_boards = boards;
                self.board_picker_loading = false;
            }
            AppMessage::BoardsLoaded(Err(_)) => {
                self.board_picker_loading = false;
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
                let favorites: Vec<&str> = self.projects.iter().map(|f| f.key.as_str()).collect();
                self.find_results = projects
                    .into_iter()
                    .map(|p| FindProject {
                        is_favorite: favorites.contains(&p.key.as_str()),
                        key: p.key,
                        name: p.name,
                    })
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

        match self.focus {
            FocusLayer::Auth => self.handle_auth_key(key),
            FocusLayer::Settings => self.handle_settings_key(key),
            FocusLayer::Find => self.handle_find_key(key),
            FocusLayer::BoardPicker => self.handle_board_picker_key(key),
            FocusLayer::ProjectDropdown => self.handle_dropdown_key(key),
            FocusLayer::Main => self.handle_main_key(key),
        }
    }

    fn handle_main_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') => self.running = false,
            KeyCode::Tab => self.next_tab(),
            KeyCode::BackTab => self.prev_tab(),
            KeyCode::Char('1') => self.active_tab = Tab::Backlog,
            KeyCode::Char(c @ '2'..='9') => {
                let idx = (c as usize) - ('2' as usize);
                if idx < self.board_tabs.len() {
                    self.active_tab = Tab::Board(self.board_tabs[idx].board_id);
                }
            }
            KeyCode::Char('p') => {
                self.project_selector_open = true;
                self.focus = FocusLayer::ProjectDropdown;
            }
            KeyCode::Char('f') => self.open_find(),
            KeyCode::Char('r') => self.refresh_active_tab(),
            KeyCode::Char(',') => self.open_settings(),
            KeyCode::Char('x') => self.close_active_board_tab(),
            KeyCode::Enter if self.active_tab == Tab::Backlog && !self.detail_open => {
                self.open_detail_from_backlog();
            }
            KeyCode::Esc if self.detail_open => {
                self.detail_open = false;
                self.detail_issue = None;
                self.detail_description = None;
                self.detail_scroll = 0;
            }
            KeyCode::Down | KeyCode::Char('j') if self.detail_open => {
                self.detail_scroll = self.detail_scroll.saturating_add(1);
            }
            KeyCode::Up | KeyCode::Char('k') if self.detail_open => {
                self.detail_scroll = self.detail_scroll.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') if self.active_tab == Tab::Backlog => {
                let count = self.filtered_backlog_count();
                self.backlog_nav.move_down(count);
            }
            KeyCode::Up | KeyCode::Char('k') if self.active_tab == Tab::Backlog => {
                let count = self.filtered_backlog_count();
                self.backlog_nav.move_up(count);
            }
            _ => {}
        }
    }

    fn handle_dropdown_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.selected_project > 0 {
                    self.selected_project -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.selected_project + 1 < self.projects.len() {
                    self.selected_project += 1;
                }
            }
            KeyCode::Enter => {
                self.project_selector_open = false;
                self.focus = FocusLayer::Main;
                self.on_project_changed();
            }
            KeyCode::Esc => {
                self.project_selector_open = false;
                self.focus = FocusLayer::Main;
            }
            KeyCode::Char('s') => {
                if let Some(project) = self.projects.get(self.selected_project).cloned() {
                    self.remove_favorite(&project.key);
                    if self.projects.is_empty() {
                        self.project_selector_open = false;
                        self.focus = FocusLayer::Main;
                    }
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
                    self.add_favorite(&project);
                    self.selected_project = self
                        .projects
                        .iter()
                        .position(|p| p.key == project.key)
                        .unwrap_or(0);
                    self.find_modal_open = false;
                    self.focus = FocusLayer::Main;
                    self.on_project_changed();
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
            KeyCode::Char('s') if !self.find_results.is_empty() => {
                if let Some(project) = self.find_results.get(self.find_selected).cloned() {
                    if project.is_favorite {
                        self.remove_favorite(&project.key);
                    } else {
                        self.add_favorite(&project);
                    }
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

    pub fn load_backlog(&self) {
        let project = match self.projects.get(self.selected_project) {
            Some(p) => p.clone(),
            None => return,
        };
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
        let project_key = project.key;

        tokio::spawn(async move {
            let provider = JiraProvider::new(client, base_url, email, token);
            let result = provider.get_backlog(&project_key).await;
            let _ = tx.send(AppMessage::BacklogLoaded(result));
        });
    }

    fn add_favorite(&mut self, project: &FindProject) {
        let already = self.config.jira.favorites.iter().any(|f| f.key == project.key);
        if already {
            return;
        }
        self.config.jira.favorites.push(FavoriteProject {
            key: project.key.clone(),
            name: project.name.clone(),
        });
        let _ = config::save_config(&self.config);

        if !self.projects.iter().any(|p| p.key == project.key) {
            self.projects.push(FavoriteProject {
                key: project.key.clone(),
                name: project.name.clone(),
            });
        }

        for p in &mut self.find_results {
            if p.key == project.key {
                p.is_favorite = true;
            }
        }
    }

    fn remove_favorite(&mut self, key: &str) {
        self.config.jira.favorites.retain(|f| f.key != key);
        let _ = config::save_config(&self.config);

        self.projects.retain(|p| p.key != key);
        if self.selected_project >= self.projects.len() && !self.projects.is_empty() {
            self.selected_project = self.projects.len() - 1;
        }

        for p in &mut self.find_results {
            if p.key == key {
                p.is_favorite = false;
            }
        }
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
                self.detail_scroll = self.detail_scroll.saturating_add(2);
                return;
            }
            MouseEventKind::ScrollUp if self.focus == FocusLayer::Main && self.detail_open => {
                self.detail_scroll = self.detail_scroll.saturating_sub(2);
                return;
            }
            MouseEventKind::ScrollDown if self.focus == FocusLayer::Main && self.active_tab == Tab::Backlog => {
                let count = self.filtered_backlog_count();
                self.backlog_nav.scroll_down(count);
                return;
            }
            MouseEventKind::ScrollUp if self.focus == FocusLayer::Main && self.active_tab == Tab::Backlog => {
                self.backlog_nav.scroll_up();
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

            if self.board_picker_open {
                return;
            }

            if self.project_selector_open {
                self.handle_dropdown_mouse(pos);
                return;
            }

            if self.detail_open {
                if hit(pos, self.detail_resize_area) {
                    self.detail_dragging = true;
                    return;
                }
                if hit(pos, self.detail_close_area) {
                    self.detail_open = false;
                    self.detail_issue = None;
                    self.detail_description = None;
                    self.detail_scroll = 0;
                    return;
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

            let mut tab_clicked = false;
            for (area, idx) in &self.click_regions.header.tab_areas {
                if hit(pos, Some(*area)) {
                    if *idx == 0 {
                        self.active_tab = Tab::Backlog;
                    } else if let Some(bt) = self.board_tabs.get(*idx - 1) {
                        self.active_tab = Tab::Board(bt.board_id);
                    }
                    tab_clicked = true;
                    break;
                }
            }
            if tab_clicked {
                // handled
            } else if hit(pos, self.click_regions.header.tab_add) {
                self.open_board_picker();
            } else if hit(pos, self.click_regions.header.project_selector) {
                self.project_selector_open = !self.project_selector_open;
                self.focus = if self.project_selector_open {
                    FocusLayer::ProjectDropdown
                } else {
                    FocusLayer::Main
                };
            } else if hit(pos, self.click_regions.header.find_link) {
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
            if self.active_tab == Tab::Backlog {
                for (i, area) in self.click_regions.backlog.row_areas.iter().enumerate() {
                    if hit(pos, Some(*area)) {
                        self.backlog_nav.selected = Some(self.backlog_nav.offset + i);
                        self.open_detail_from_backlog();
                        return;
                    }
                }
            }

            // Backlog filter clicks
            if self.active_tab == Tab::Backlog {
                for (i, area) in self.click_regions.backlog.filter_areas.iter().enumerate() {
                    if hit(pos, Some(*area)) {
                        if i == 0 {
                            self.backlog_filter = None;
                        } else if let Some(status) = self.backlog_statuses.get(i - 1) {
                            self.backlog_filter = Some(status.clone());
                        }
                        self.backlog_nav.reset();
                        break;
                    }
                }
            }
        }
    }

    fn handle_dropdown_mouse(&mut self, pos: (u16, u16)) {
        for (i, area) in self.click_regions.project_dropdown.items.iter().enumerate() {
            if hit(pos, Some(*area)) {
                self.selected_project = i;
                self.project_selector_open = false;
                self.focus = FocusLayer::Main;
                self.on_project_changed();
                return;
            }
        }
        if !hit(pos, self.click_regions.project_dropdown.bounds) {
            self.project_selector_open = false;
            self.focus = FocusLayer::Main;
        }
    }

    fn handle_find_mouse(&mut self, pos: (u16, u16)) {
        for (i, area) in self.click_regions.find_modal.star_areas.iter().enumerate() {
            if hit(pos, Some(*area)) {
                if let Some(project) = self.find_results.get(i).cloned() {
                    if project.is_favorite {
                        self.remove_favorite(&project.key);
                    } else {
                        self.add_favorite(&project);
                    }
                }
                return;
            }
        }

        for (i, area) in self.click_regions.find_modal.result_areas.iter().enumerate() {
            if hit(pos, Some(*area)) {
                self.find_selected = i;
                if let Some(project) = self.find_results.get(i).cloned() {
                    self.add_favorite(&project);
                    self.selected_project = self
                        .projects
                        .iter()
                        .position(|p| p.key == project.key)
                        .unwrap_or(0);
                    self.find_modal_open = false;
                    self.focus = FocusLayer::Main;
                    self.on_project_changed();
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
        let filtered: Vec<&JiraIssue> = self
            .backlog_issues
            .iter()
            .filter(|issue| match &self.backlog_filter {
                None => true,
                Some(f) => issue.fields.status.name == *f,
            })
            .collect();
        if let Some(issue) = self.backlog_nav.selected.and_then(|i| filtered.get(i)) {
            self.detail_issue = Some((*issue).clone());
            self.detail_description = None;
            self.detail_open = true;
            self.detail_height = 0;
            self.detail_scroll = 0;
        }
    }

    pub fn open_detail_for_issue(&mut self, issue: &JiraIssue) {
        self.detail_issue = Some(issue.clone());
        self.detail_description = None;
        self.detail_open = true;
        self.detail_height = 0;
        self.detail_scroll = 0;
    }

    fn on_project_changed(&mut self) {
        self.column_order.clear();
        self.backlog_filter = None;
        self.backlog_loading = true;
        self.backlog_nav.reset();
        self.load_column_order();
        self.load_backlog();
    }

    fn filtered_backlog_count(&self) -> usize {
        match &self.backlog_filter {
            None => self.backlog_issues.len(),
            Some(f) => self.backlog_issues.iter().filter(|i| i.fields.status.name == *f).count(),
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
        let mut tabs = vec![Tab::Backlog];
        for bt in &self.board_tabs {
            tabs.push(Tab::Board(bt.board_id));
        }
        tabs
    }

    fn refresh_active_tab(&mut self) {
        match &self.active_tab {
            Tab::Backlog => {
                self.backlog_loading = true;
                self.backlog_error = None;
                self.load_backlog();
            }
            Tab::Board(id) => {
                let board_id = *id;
                if let Some(tab) = self.board_tabs.iter_mut().find(|t| t.board_id == board_id) {
                    tab.loading = true;
                    tab.error = None;
                }
                self.load_board_data(board_id);
            }
        }
    }

    fn close_active_board_tab(&mut self) {
        if let Tab::Board(id) = &self.active_tab {
            let board_id = *id;
            self.board_tabs.retain(|t| t.board_id != board_id);
            self.active_tab = Tab::Backlog;
            self.save_open_boards();
        }
    }

    fn open_board_picker(&mut self) {
        self.board_picker_open = true;
        self.board_picker_selected = 0;
        self.board_picker_boards.clear();
        self.board_picker_loading = true;
        self.focus = FocusLayer::BoardPicker;
        self.load_boards_list();
    }

    fn handle_board_picker_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.board_picker_open = false;
                self.focus = FocusLayer::Main;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.board_picker_selected > 0 {
                    self.board_picker_selected -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.board_picker_selected < self.board_picker_boards.len().saturating_sub(1) {
                    self.board_picker_selected += 1;
                }
            }
            KeyCode::Enter => {
                if let Some(board) = self.board_picker_boards.get(self.board_picker_selected).cloned() {
                    self.add_board_tab(board);
                    self.board_picker_open = false;
                    self.focus = FocusLayer::Main;
                }
            }
            _ => {}
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
        self.save_open_boards();
        self.load_board_data(board.id);
    }

    fn save_open_boards(&self) {
        let project_key = self
            .projects
            .get(self.selected_project)
            .map(|p| p.key.clone())
            .unwrap_or_default();
        let mut config = self.config.clone();
        config.jira.open_boards.retain(|b| b.project_key != project_key);
        for bt in &self.board_tabs {
            config.jira.open_boards.push(OpenBoard {
                project_key: project_key.clone(),
                board_id: bt.board_id,
                board_name: bt.board_name.clone(),
            });
        }
        let _ = config::save_config(&config);
    }

    fn load_boards_list(&self) {
        let project = match self.projects.get(self.selected_project) {
            Some(p) => p.clone(),
            None => return,
        };
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
            let result = provider.get_boards(&project.key).await;
            let _ = tx.send(AppMessage::BoardsLoaded(result));
        });
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
