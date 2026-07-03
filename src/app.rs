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
    pub settings_apply_area: Option<Rect>,
    pub settings_close_area: Option<Rect>,
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
            settings_apply_area: None,
            settings_close_area: None,
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
            AppEvent::Resize(_, _) => {}
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
                self.backlog_issues = issues;
                self.backlog_loading = false;
                self.backlog_nav.clamp(self.backlog_issues.len());
            }
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
                            // Fallback: match by column name ~= status name
                            let status_name = &issue.fields.status.name;
                            let placed_by_name = tab.columns.iter_mut().any(|col| {
                                if col.name.eq_ignore_ascii_case(status_name) {
                                    col.issues.push(issue.clone());
                                    true
                                } else {
                                    false
                                }
                            });
                            if !placed_by_name {
                                if let Some(col) = tab.columns.first_mut() {
                                    col.issues.push(issue.clone());
                                }
                            }
                        }
                    }

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
            AppMessage::Tick => {}
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
            KeyCode::Down | KeyCode::Char('j') if self.active_tab == Tab::Backlog => {
                self.backlog_nav.move_down(self.backlog_issues.len());
            }
            KeyCode::Up | KeyCode::Char('k') if self.active_tab == Tab::Backlog => {
                self.backlog_nav.move_up(self.backlog_issues.len());
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
                self.backlog_loading = true;
                self.backlog_nav.reset();
                self.load_backlog();
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
        const NUM_TABS: usize = 3;
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
                    self.backlog_loading = true;
                    self.backlog_nav.reset();
                    self.load_backlog();
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
        let _ = config::save_config(&self.config);
        self.settings_open = false;
        self.focus = FocusLayer::Main;
    }

    fn handle_mouse(&mut self, mouse: MouseEvent) {
        match mouse.kind {
            MouseEventKind::ScrollDown if self.focus == FocusLayer::Main && self.active_tab == Tab::Backlog => {
                self.backlog_nav.scroll_down(self.backlog_issues.len());
                return;
            }
            MouseEventKind::ScrollUp if self.focus == FocusLayer::Main && self.active_tab == Tab::Backlog => {
                self.backlog_nav.scroll_up();
                return;
            }
            _ => {}
        }

        if let MouseEventKind::Down(MouseButton::Left) = mouse.kind {
            let pos = (mouse.column, mouse.row);

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
        }
    }

    fn handle_dropdown_mouse(&mut self, pos: (u16, u16)) {
        for (i, area) in self.click_regions.project_dropdown.items.iter().enumerate() {
            if hit(pos, Some(*area)) {
                self.selected_project = i;
                self.project_selector_open = false;
                self.focus = FocusLayer::Main;
                self.backlog_loading = true;
                self.backlog_nav.reset();
                self.load_backlog();
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
                    self.backlog_loading = true;
                    self.backlog_nav.reset();
                    self.load_backlog();
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
