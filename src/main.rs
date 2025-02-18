use crate::{
    app::App,
    ui::Ui
};
use ratatui;
use color_eyre::Result;
//use reqwest;
mod app;
mod ui;

fn main() -> Result<()> {
    // Setup terminal
    let mut terminal = ratatui::init();
    
    // Run the app
    let mut app = App::new();
    let res = Ui::new(app).run(terminal);

    // Clean
    ratatui::restore();
    res
}
