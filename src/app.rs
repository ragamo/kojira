use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::prelude::Rect;

use crate::config;
use crate::config::types::{AppConfig, FavoriteProject};
use crate::event::AppEvent;
use crate::theme::{self, Theme};
use crate::ui::click_regions::ClickRegions;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Backlog,
    Board,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusLayer {
    Main,
    Settings,
    ProjectDropdown,
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
    pub fn new(config: AppConfig) -> Self {
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
            AppEvent::Tick => {}
            AppEvent::Resize(_, _) => {}
            AppEvent::Message(_) => {}
        }
        Ok(())
    }

    fn handle_key(&mut self, key: KeyEvent) {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            self.running = false;
            return;
        }

        match self.focus {
            FocusLayer::Settings => self.handle_settings_key(key),
            FocusLayer::ProjectDropdown => self.handle_dropdown_key(key),
            FocusLayer::Main => self.handle_main_key(key),
        }
    }

    fn handle_main_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') => self.running = false,
            KeyCode::Tab => self.toggle_tab(),
            KeyCode::Char('1') => self.active_tab = Tab::Backlog,
            KeyCode::Char('2') => self.active_tab = Tab::Board,
            KeyCode::Char('p') => {
                self.project_selector_open = true;
                self.focus = FocusLayer::ProjectDropdown;
            }
            KeyCode::Char(',') => self.open_settings(),
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
            }
            KeyCode::Esc => {
                self.project_selector_open = false;
                self.focus = FocusLayer::Main;
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
        if let MouseEventKind::Down(MouseButton::Left) = mouse.kind {
            let pos = (mouse.column, mouse.row);

            if self.settings_open {
                self.handle_settings_mouse(pos);
                return;
            }

            if hit(pos, self.click_regions.header.tab_backlog) {
                self.active_tab = Tab::Backlog;
            } else if hit(pos, self.click_regions.header.tab_board) {
                self.active_tab = Tab::Board;
            } else if hit(pos, self.click_regions.header.project_selector) {
                self.project_selector_open = !self.project_selector_open;
                self.focus = if self.project_selector_open {
                    FocusLayer::ProjectDropdown
                } else {
                    FocusLayer::Main
                };
            } else if hit(pos, self.click_regions.header.settings_link) {
                self.open_settings();
            }
        }
    }

    fn handle_settings_mouse(&mut self, pos: (u16, u16)) {
        // Tab clicks
        for (i, area) in self.settings_tab_areas.iter().enumerate() {
            if hit(pos, Some(*area)) {
                self.settings_selected = i;
                return;
            }
        }

        // Theme list clicks
        if self.settings_selected == 0 {
            for (i, area) in self.settings_theme_areas.iter().enumerate() {
                if hit(pos, Some(*area)) {
                    self.theme_selected = i;
                    self.theme = theme::ALL_THEMES[i];
                    return;
                }
            }
        }

        // Config tab clicks
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

        // Apply / Close buttons
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

    fn toggle_tab(&mut self) {
        self.active_tab = match self.active_tab {
            Tab::Backlog => Tab::Board,
            Tab::Board => Tab::Backlog,
        };
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
