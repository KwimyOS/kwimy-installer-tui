use std::collections::VecDeque;
use std::io::Write;
use std::path::Path;

use crate::model::{App, InstallerEvent, StepStatus};

pub(crate) const LOG_CAPACITY: usize = 200;
pub(crate) const LOG_FILE_PATH: &str = "/tmp/kwimy-installer.log";

pub(crate) fn handle_event(app: &mut App, evt: InstallerEvent) {
    match evt {
        InstallerEvent::Log(line) => {
            push_log(&mut app.logs, line.clone());
            append_log_file(&mut app.log_file, &line);
        }
        InstallerEvent::Progress(value) => app.progress = value,
        InstallerEvent::Step { index, status, err } => {
            if let Some(step) = app.steps.get_mut(index) {
                step.status = status;
                step.err = err.clone();
                let status_label = match step.status {
                    StepStatus::Pending => "PENDING",
                    StepStatus::Running => "RUNNING",
                    StepStatus::Done => "OK",
                    StepStatus::Skipped => "SKIP",
                    StepStatus::Failed => "FAIL",
                };
                let line = format!("STEP {}: {}", step.name, status_label);
                append_log_file(&mut app.log_file, &line);
                if let Some(err) = err {
                    append_log_file(&mut app.log_file, &format!("ERROR: {}", err));
                }
            }
        }
        InstallerEvent::Done(err) => {
            app.done = true;
            app.err = err.clone();
            if let Some(err) = err {
                append_log_file(&mut app.log_file, &format!("DONE: {}", err));
            } else {
                append_log_file(&mut app.log_file, "DONE: ok");
                if Path::new("/mnt/var/log/kwimy-failed-packages.txt").exists() {
                    let line = "Optional packages failed. See /var/log/kwimy-failed-packages.txt on the installed system.";
                    push_log(&mut app.logs, line.to_string());
                    append_log_file(&mut app.log_file, line);
                }
            }
        }
    }
}

pub(crate) fn push_log(logs: &mut VecDeque<String>, line: String) {
    if logs.len() >= LOG_CAPACITY {
        logs.pop_front();
    }
    logs.push_back(line);
}

pub(crate) fn append_log_file(log_file: &mut Option<std::fs::File>, line: &str) {
    if let Some(file) = log_file.as_mut() {
        let _ = writeln!(file, "{}", line);
        let _ = file.flush();
    }
}
