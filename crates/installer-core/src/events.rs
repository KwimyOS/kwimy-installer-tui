#[derive(Clone, Copy, PartialEq, Eq)]
pub enum StepStatus {
    Pending,
    Running,
    Done,
    Skipped,
    Failed,
}

pub enum InstallerEvent {
    Log(String),
    Progress(f64),
    Step {
        index: usize,
        status: StepStatus,
        err: Option<String>,
    },
    Done(Option<String>),
}
