mod app;
mod ui;
use crate::{app::init::App, ui::init::Ui};
use color_eyre::Result;
use ratatui;

fn main() -> Result<()> {
    // Setup terminal
    let terminal = ratatui::init();

    // Run the app
    let app = App::new();
    let res = Ui::new(app).run(terminal);

    // Clean
    ratatui::restore();
    res
}
