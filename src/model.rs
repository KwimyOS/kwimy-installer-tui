use std::collections::VecDeque;
use std::fs::File;

pub use installer_core::events::{InstallerEvent, StepStatus};

// Single installation step
pub struct Step {
    pub name: String,        // The name of the step
    pub status: StepStatus,  // The current status of the step
    pub err: Option<String>, // An error message if the step failed
}

// The main application state
pub struct App {
    // The list of all installation steps
    pub steps: Vec<Step>,
    // The overall progress of the installation
    pub progress: f64,
    // A queue of log messages to be displayed
    pub logs: VecDeque<String>,
    // The current frame of the loading spinner animation
    pub spinner_idx: usize,
    // A flag indicating whether the installation is finished
    pub done: bool,
    // A final error message if the installation failed
    pub err: Option<String>,
    // An optional handle to the log file for writing logs to disk
    pub log_file: Option<File>,
}
