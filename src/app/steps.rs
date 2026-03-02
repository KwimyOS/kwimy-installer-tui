use crate::disks::DiskInfo;
use crate::drivers::{nvidia_variant_label, NvidiaVariant};
use crate::ui::{InstallSummary, SUMMARY_STEP_COUNT};

#[derive(Clone, Copy, Debug)]
pub(crate) enum SetupStep {
    Network,
    Disk,
    ConfirmDisk,
    Keymap,
    Timezone,
    Hostname,
    Username,
    UserPassword,
    EncryptDisk,
    LuksPassword,
    Drivers,
    Swap,
    Applications,
    Review,
}

pub(crate) fn summary_current_index(step: SetupStep, include_drivers: bool) -> usize {
    let step_count = SUMMARY_STEP_COUNT + if include_drivers { 1 } else { 0 };
    match step {
        SetupStep::Network => 0,
        SetupStep::Drivers => 1,
        SetupStep::Disk | SetupStep::ConfirmDisk => {
            if include_drivers {
                2
            } else {
                1
            }
        }
        SetupStep::Keymap => {
            if include_drivers {
                3
            } else {
                2
            }
        }
        SetupStep::Timezone => {
            if include_drivers {
                4
            } else {
                3
            }
        }
        SetupStep::Hostname => {
            if include_drivers {
                5
            } else {
                4
            }
        }
        SetupStep::Username | SetupStep::UserPassword => {
            if include_drivers {
                6
            } else {
                5
            }
        }
        SetupStep::EncryptDisk | SetupStep::LuksPassword => {
            if include_drivers {
                7
            } else {
                6
            }
        }
        SetupStep::Swap => {
            if include_drivers {
                8
            } else {
                7
            }
        }
        SetupStep::Applications | SetupStep::Review => step_count,
    }
}

pub(crate) fn build_install_summary(
    step: SetupStep,
    include_drivers: bool,
    network: Option<&str>,
    selected_disk: Option<&DiskInfo>,
    keymap: &str,
    timezone: &str,
    hostname: &str,
    username: &str,
    user_password: &str,
    luks_password: &str,
    encrypt_disk: bool,
    swap_enabled: bool,
    nvidia_variant: Option<NvidiaVariant>,
) -> InstallSummary {
    let drivers = if include_drivers {
        Some(
            nvidia_variant
                .map(nvidia_variant_label)
                .unwrap_or("Skipped")
                .to_string(),
        )
    } else {
        None
    };
    InstallSummary {
        current_index: summary_current_index(step, include_drivers),
        network: network.map(|value| value.to_string()),
        drivers,
        disk: selected_disk.map(|disk| disk.label()),
        keymap: Some(keymap.to_string()),
        timezone: Some(timezone.to_string()),
        hostname: Some(hostname.to_string()),
        username: if user_password.is_empty() || username.is_empty() {
            None
        } else {
            Some(username.to_string())
        },
        encryption: if !encrypt_disk {
            Some("no".to_string())
        } else if luks_password.is_empty() {
            None
        } else {
            Some("Btrfs (LUKS encrypted)".to_string())
        },
        zram_swap: Some(if swap_enabled { "yes" } else { "no" }.to_string()),
        include_drivers,
    }
}
