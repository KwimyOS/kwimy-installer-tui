use std::collections::HashSet;

use anyhow::Result;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use crate::disks::DiskInfo;
use crate::drivers::{driver_packages, format_gpu_summary, GpuVendor, NvidiaVariant};
use crate::selection::{
    browser_choices, compositor_choices, compositor_labels, editor_choices, labels_for_flags,
    labels_for_selection, selection_from_app_flags, selection_from_flags_for, terminal_choices,
    AppSelectionFlags, PackageSelection,
};
use crate::ui::{run_application_selector, run_review, ReviewAction, ReviewItem, SelectionAction};
use installer_core::InstallConfig;

use super::super::steps::{build_install_summary, SetupStep};
use super::StepOutcome;

#[allow(clippy::too_many_arguments)]
pub(in crate::app) fn handle_applications_step(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    include_drivers: bool,
    network_label: Option<&str>,
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
    app_flags: &mut AppSelectionFlags,
    app_selection: &mut PackageSelection,
) -> Result<StepOutcome> {
    let summary = build_install_summary(
        SetupStep::Applications,
        include_drivers,
        network_label,
        selected_disk,
        keymap,
        timezone,
        hostname,
        username,
        user_password,
        luks_password,
        encrypt_disk,
        swap_enabled,
        nvidia_variant,
    );
    match run_application_selector(terminal, app_flags, &summary)? {
        SelectionAction::Submit(flags) => {
            *app_flags = flags;
            *app_selection = selection_from_app_flags(app_flags);
            Ok(StepOutcome::Next(SetupStep::Review))
        }
        SelectionAction::Back => Ok(StepOutcome::Next(SetupStep::Swap)),
        SelectionAction::Quit => Ok(StepOutcome::Quit),
    }
}

#[allow(clippy::too_many_arguments)]
pub(in crate::app) fn handle_review_step(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    network_label: Option<&str>,
    selected_disk: Option<&DiskInfo>,
    encrypt_disk: bool,
    gpu_vendors: &HashSet<GpuVendor>,
    nvidia_variant: Option<NvidiaVariant>,
    swap_enabled: bool,
    hostname: &str,
    username: &str,
    keymap: &str,
    timezone: &str,
    app_flags: &AppSelectionFlags,
    app_selection: &PackageSelection,
) -> Result<StepOutcome> {
    let Some(disk) = selected_disk else {
        return Ok(StepOutcome::Next(SetupStep::Disk));
    };

    let compositor_labels = labels_for_flags(&app_flags.compositors, &compositor_labels());
    let browser_labels = labels_for_selection(app_selection, browser_choices());
    let editor_labels = labels_for_selection(app_selection, editor_choices());
    let terminal_labels = labels_for_selection(app_selection, terminal_choices());
    let system_items = vec![
        ReviewItem {
            label: "Network".to_string(),
            value: network_label
                .map(|s| s.to_string())
                .unwrap_or_else(|| "Not connected".to_string()),
        },
        ReviewItem {
            label: "Disk".to_string(),
            value: disk.label(),
        },
        ReviewItem {
            label: "Filesystem".to_string(),
            value: if encrypt_disk {
                "Btrfs (LUKS encrypted)".to_string()
            } else {
                "Btrfs".to_string()
            },
        },
        ReviewItem {
            label: "GPU".to_string(),
            value: format_gpu_summary(gpu_vendors, nvidia_variant)
                .unwrap_or_else(|| "Not detected".to_string()),
        },
        ReviewItem {
            label: "Swap".to_string(),
            value: if swap_enabled {
                "Enabled (zram)".to_string()
            } else {
                "Disabled".to_string()
            },
        },
        ReviewItem {
            label: "Hostname".to_string(),
            value: hostname.to_string(),
        },
        ReviewItem {
            label: "Username".to_string(),
            value: username.to_string(),
        },
        ReviewItem {
            label: "Keyboard".to_string(),
            value: keymap.to_string(),
        },
        ReviewItem {
            label: "Timezone".to_string(),
            value: timezone.to_string(),
        },
    ];
    let package_items = vec![
        ReviewItem {
            label: "Compositor".to_string(),
            value: if compositor_labels.is_empty() {
                "None".to_string()
            } else {
                compositor_labels.join(", ")
            },
        },
        ReviewItem {
            label: "Browsers".to_string(),
            value: if browser_labels.is_empty() {
                "None".to_string()
            } else {
                browser_labels.join(", ")
            },
        },
        ReviewItem {
            label: "Editors".to_string(),
            value: if editor_labels.is_empty() {
                "None".to_string()
            } else {
                editor_labels.join(", ")
            },
        },
        ReviewItem {
            label: "Terminals".to_string(),
            value: if terminal_labels.is_empty() {
                "None".to_string()
            } else {
                terminal_labels.join(", ")
            },
        },
    ];
    let selected_packages = compositor_labels.len()
        + browser_labels.len()
        + editor_labels.len()
        + terminal_labels.len();

    match run_review(terminal, &system_items, &package_items, selected_packages)? {
        ReviewAction::Confirm => Ok(StepOutcome::Finish),
        ReviewAction::Back => Ok(StepOutcome::Next(SetupStep::Applications)),
        ReviewAction::Edit => Ok(StepOutcome::Next(SetupStep::Network)),
        ReviewAction::Quit => Ok(StepOutcome::Quit),
    }
}

#[allow(clippy::too_many_arguments)]
pub(in crate::app) fn build_install_config(
    selected_disk: &DiskInfo,
    keymap: String,
    timezone: String,
    hostname: String,
    username: String,
    user_password: String,
    luks_password: String,
    encrypt_disk: bool,
    swap_enabled: bool,
    gpu_vendors: &HashSet<GpuVendor>,
    nvidia_variant: Option<NvidiaVariant>,
    kernel_package: String,
    kernel_headers: String,
    mut base_packages: Vec<String>,
    app_flags: &AppSelectionFlags,
    app_selection: PackageSelection,
    offline_only: bool,
) -> InstallConfig {
    let mut compositor_flags = vec![false; compositor_choices().len()];
    if let Some((idx, _)) = app_flags
        .compositors
        .iter()
        .enumerate()
        .find(|(_, flag)| **flag)
    {
        if let Some(flag) = compositor_flags.get_mut(idx) {
            *flag = true;
        }
    }

    let compositor_selection = selection_from_flags_for(&compositor_flags, compositor_choices());
    base_packages.extend(compositor_selection.pacman);
    let selected_browsers = labels_for_selection(&app_selection, browser_choices());
    let selected_editors = labels_for_selection(&app_selection, editor_choices());
    let mut extra_aur_packages = app_selection.yay.clone();
    extra_aur_packages.extend(compositor_selection.yay);
    let compositor_label = app_flags
        .compositors
        .iter()
        .enumerate()
        .find(|(_, flag)| **flag)
        .and_then(|(idx, _)| compositor_choices().get(idx))
        .map(|choice| choice.label.clone())
        .or_else(|| {
            compositor_choices()
                .first()
                .map(|choice| choice.label.clone())
        })
        .unwrap_or_else(|| "Hyprland (Caelestia)".to_string());

    InstallConfig {
        disk: selected_disk.clone().into(),
        keymap,
        timezone,
        hostname,
        username,
        user_password,
        luks_password,
        encrypt_disk,
        swap_enabled,
        driver_packages: driver_packages(gpu_vendors, nvidia_variant),
        kernel_package,
        kernel_headers,
        base_packages,
        selected_browsers,
        selected_editors,
        extra_pacman_packages: app_selection.pacman,
        extra_aur_packages,
        compositor_label,
        offline_only,
        hyprland_selected: app_flags.compositors.iter().any(|flag| *flag),
    }
}
