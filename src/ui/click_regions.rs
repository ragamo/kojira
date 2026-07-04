use ratatui::prelude::Rect;

#[derive(Default)]
pub struct ClickRegions {
    pub header: HeaderRegion,
    pub backlog: BacklogRegion,
    pub find_modal: FindModalRegion,
    pub project_dropdown: ProjectDropdownRegion,
}

impl ClickRegions {
    pub fn clear(&mut self) {
        self.header = HeaderRegion::default();
        self.backlog = BacklogRegion::default();
        self.find_modal = FindModalRegion::default();
        self.project_dropdown = ProjectDropdownRegion::default();
    }
}

#[derive(Default)]
pub struct HeaderRegion {
    pub project_selector: Option<Rect>,
    pub find_link: Option<Rect>,
    pub settings_link: Option<Rect>,
    pub login_link: Option<Rect>,
    pub logout_link: Option<Rect>,
    pub tab_areas: Vec<(Rect, usize)>,
    pub tab_add: Option<Rect>,
}

#[derive(Default)]
pub struct BacklogRegion {
    pub filter_areas: Vec<Rect>,
}

#[derive(Default)]
pub struct FindModalRegion {
    pub bounds: Option<Rect>,
    pub result_areas: Vec<Rect>,
    pub star_areas: Vec<Rect>,
}

#[derive(Default)]
pub struct ProjectDropdownRegion {
    pub bounds: Option<Rect>,
    pub items: Vec<Rect>,
}
