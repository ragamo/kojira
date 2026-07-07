use ratatui::prelude::Rect;

#[derive(Default)]
pub struct ClickRegions {
    pub header: HeaderRegion,
    pub backlog: BacklogRegion,
    pub board_cards: BoardCardRegion,
    pub find_modal: FindModalRegion,
    pub assignee_modal: AssigneeModalRegion,
}

impl ClickRegions {
    pub fn clear(&mut self) {
        self.header = HeaderRegion::default();
        self.backlog = BacklogRegion::default();
        self.board_cards = BoardCardRegion::default();
        self.find_modal = FindModalRegion::default();
        self.assignee_modal = AssigneeModalRegion::default();
    }
}

#[derive(Default)]
pub struct AssigneeModalRegion {
    pub bounds: Option<Rect>,
    pub item_areas: Vec<Rect>,
}

#[derive(Default)]
pub struct HeaderRegion {
    pub settings_link: Option<Rect>,
    pub create_link: Option<Rect>,
    pub login_link: Option<Rect>,
    pub logout_link: Option<Rect>,
    pub tab_areas: Vec<(Rect, usize)>,
    pub tab_add: Option<Rect>,
    pub tab_row_y: Option<u16>,
}

#[derive(Default)]
pub struct BacklogRegion {
    pub filter_areas: Vec<Rect>,
    pub row_areas: Vec<Rect>,
}

#[derive(Default)]
pub struct BoardCardRegion {
    pub cards: Vec<(Rect, String, usize)>,
}

#[derive(Default)]
pub struct FindModalRegion {
    pub bounds: Option<Rect>,
    pub result_areas: Vec<Rect>,
    pub panel_item_areas: Vec<Rect>,
}
