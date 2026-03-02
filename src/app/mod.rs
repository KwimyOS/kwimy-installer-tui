mod flow;
mod logging;
mod progress;
mod setup;
mod setup_steps;
mod steps;
mod validation;

use anyhow::Result;
use crossterm::terminal::enable_raw_mode;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use self::flow::clear_screen;

pub fn run() -> Result<()> {
    dotenvy::dotenv().ok();

    let allow_nonroot = std::env::var("KWIMY_DEV_ALLOW_NONROOT").ok().as_deref() == Some("1");
    if unsafe { libc::geteuid() } != 0 && !allow_nonroot {
        println!("kwimy should be run as root in the live ISO.");
        println!("If you are testing locally, use sudo.");
        return Ok(());
    }

    enable_raw_mode()?;
    clear_screen()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(std::io::stdout()))?;

    let Some(config) = setup::run_setup_wizard(&mut terminal)? else {
        return Ok(());
    };

    progress::run_install_progress(&mut terminal, config)
}
