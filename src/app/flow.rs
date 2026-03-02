use std::io;

use anyhow::{Context, Result};
use crossterm::terminal::ClearType;
use crossterm::{cursor, execute, terminal::Clear};

pub(crate) fn clear_screen() -> Result<()> {
    execute!(io::stdout(), Clear(ClearType::All), cursor::MoveTo(0, 0)).context("clear screen")?;
    Ok(())
}
