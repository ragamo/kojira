pub mod auth_modal;
pub mod backlog_view;
pub mod click_regions;
pub mod find_modal;
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
    if app.find_modal_open {
        find_modal::render(frame, app);
    }
    if app.auth_open {
        auth_modal::render(frame, app);
    }
}
