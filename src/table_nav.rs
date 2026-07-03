#[derive(Debug, Default)]
pub struct TableNav {
    pub offset: usize,
    pub selected: Option<usize>,
    pub visible_rows: usize,
}

impl TableNav {
    pub fn reset(&mut self) {
        self.offset = 0;
        self.selected = None;
    }

    pub fn scroll_down(&mut self, item_count: usize) {
        let max_offset = item_count.saturating_sub(self.visible_rows);
        if self.offset < max_offset {
            self.offset = (self.offset + 3).min(max_offset);
        }
    }

    pub fn scroll_up(&mut self) {
        self.offset = self.offset.saturating_sub(3);
    }

    pub fn move_down(&mut self, item_count: usize) -> bool {
        if item_count == 0 {
            return false;
        }
        let new_index = match self.selected {
            Some(i) if i >= item_count - 1 => return false,
            Some(i) => i + 1,
            None => 0,
        };
        self.selected = Some(new_index);
        if new_index >= self.offset + self.visible_rows {
            self.offset = new_index + 1 - self.visible_rows;
        }
        true
    }

    pub fn move_up(&mut self, item_count: usize) -> bool {
        if item_count == 0 {
            return false;
        }
        let new_index = match self.selected {
            Some(0) => return false,
            Some(i) => i - 1,
            None => 0,
        };
        self.selected = Some(new_index);
        if new_index < self.offset {
            self.offset = new_index;
        }
        true
    }

    pub fn clamp(&mut self, item_count: usize) {
        let max_offset = item_count.saturating_sub(self.visible_rows);
        if self.offset > max_offset {
            self.offset = max_offset;
        }
        if let Some(i) = self.selected {
            if i >= item_count {
                self.selected = if item_count > 0 { Some(item_count - 1) } else { None };
            }
        }
    }
}
