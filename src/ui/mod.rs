pub mod click_regions;
pub mod main_view;
pub mod settings_modal;

use ratatui::Frame;

use crate::app::App;

pub fn render(frame: &mut Frame, app: &mut App) {
    app.click_regions.clear();
    main_view::render(frame, app);

    if app.settings_open {
        settings_modal::render(frame, app);
    }
}
