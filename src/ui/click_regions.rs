use ratatui::prelude::Rect;

#[derive(Default)]
pub struct ClickRegions {
    pub header: HeaderRegion,
}

impl ClickRegions {
    pub fn clear(&mut self) {
        self.header = HeaderRegion::default();
    }
}

#[derive(Default)]
pub struct HeaderRegion {
    pub project_selector: Option<Rect>,
    pub find_link: Option<Rect>,
    pub settings_link: Option<Rect>,
    pub login_link: Option<Rect>,
    pub logout_link: Option<Rect>,
    pub tab_backlog: Option<Rect>,
    pub tab_board: Option<Rect>,
}
