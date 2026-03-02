use std::collections::VecDeque;
use std::fs::OpenOptions;
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::terminal::disable_raw_mode;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use crate::model::{App, InstallerEvent, Step, StepStatus};
use crate::ui::{draw_ui, SPINNER_LEN};
use installer_core::{run_installer, InstallConfig, STEP_NAMES};

use super::flow::clear_screen;
use super::logging::{append_log_file, handle_event, push_log, LOG_FILE_PATH};

pub(crate) fn run_install_progress(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    config: InstallConfig,
) -> Result<()> {
    let (tx, rx) = crossbeam_channel::unbounded();
    let installer_tx = tx.clone();
    thread::spawn(move || {
        if let Err(err) = run_installer(installer_tx, &config) {
            let _ = tx.send(InstallerEvent::Done(Some(err.to_string())));
        }
    });

    // Set up the UI for the installation progress screen
    clear_screen()?;
    let step_names: Vec<String> = STEP_NAMES.iter().map(|name| (*name).to_string()).collect();

    let logs = VecDeque::from(vec!["Starting kwimy installer...".to_string()]);
    let log_file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(LOG_FILE_PATH)
        .ok();

    let mut app = App {
        steps: step_names
            .iter()
            .map(|name| Step {
                name: name.to_string(),
                status: StepStatus::Pending,
                err: None,
            })
            .collect(),
        progress: 0.0,
        logs,
        spinner_idx: 0,
        done: false,
        err: None,
        log_file,
    };
    if app.log_file.is_some() {
        let line = format!("Logging to {}", LOG_FILE_PATH);
        push_log(&mut app.logs, line.clone());
        append_log_file(&mut app.log_file, &line);
    }

    terminal.clear().context("clear terminal")?;
    terminal.draw(|f| draw_ui(f.size(), f, &app))?;

    // Installation progress screen
    let mut last_tick = Instant::now();
    let mut reboot_requested = false;
    let mut shutdown_requested = false;
    loop {
        terminal.draw(|f| draw_ui(f.size(), f, &app))?;

        let timeout = Duration::from_millis(100);
        if event::poll(timeout).context("poll events")? {
            if let Event::Key(key) = event::read().context("read event")? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Char('Q')
                            if key.modifiers.contains(KeyModifiers::CONTROL) =>
                        {
                            break
                        }
                        KeyCode::Char('r') | KeyCode::Char('R')
                            if app.done && app.err.is_none() =>
                        {
                            reboot_requested = true;
                            break;
                        }
                        KeyCode::Char('s') | KeyCode::Char('S')
                            if app.done && app.err.is_none() =>
                        {
                            shutdown_requested = true;
                            break;
                        }
                        _ => {}
                    }
                }
            }
        }

        while let Ok(evt) = rx.try_recv() {
            handle_event(&mut app, evt);
        }

        // Update the spinner animation
        if last_tick.elapsed() >= Duration::from_millis(120) {
            app.spinner_idx = (app.spinner_idx + 1) % SPINNER_LEN;
            last_tick = Instant::now();
        }
    }

    // Clean up the terminal before exiting
    disable_raw_mode().context("disable raw mode")?;
    let _ = clear_screen();
    if reboot_requested {
        Command::new("systemctl")
            .arg("reboot")
            .status()
            .context("reboot system")?;
    } else if shutdown_requested {
        Command::new("systemctl")
            .arg("poweroff")
            .status()
            .context("power off system")?;
    }
    Ok(())
}
