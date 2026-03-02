pub(super) mod apps_step;
pub(super) mod disk_step;
pub(super) mod identity_step;
pub(super) mod network_step;

use super::steps::SetupStep;

pub(super) enum StepOutcome {
    Next(SetupStep),
    Quit,
    Finish,
}
