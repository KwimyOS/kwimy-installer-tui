use anyhow::{Context, Result};
use crossterm::terminal::disable_raw_mode;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use crate::disks::{list_disks, DiskInfo};
use crate::drivers::{detect_gpu_vendors, GpuVendor, NvidiaVariant};
use crate::packages::required_packages;
use crate::selection::{AppSelectionFlags, PackageSelection};
use crate::timezones::{detect_timezone_local, load_timezones};
use installer_core::InstallConfig;

use super::flow::clear_screen;
use super::setup_steps::apps_step;
use super::setup_steps::disk_step;
use super::setup_steps::identity_step;
use super::setup_steps::network_step;
use super::setup_steps::StepOutcome;
use super::steps::SetupStep;

pub(crate) fn run_setup_wizard(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
) -> Result<Option<InstallConfig>> {
    let disks = list_disks().context("list disks")?;
    if disks.is_empty() {
        println!("No disks detected.");
        return Ok(None);
    }
    let base_packages = required_packages();

    let mut selected_disk: Option<DiskInfo> = None;
    let mut keymap = "us".to_string();
    let keymaps = disk_step::load_setup_keymaps();
    let timezones = load_timezones().unwrap_or_else(|_| vec!["UTC".to_string()]);
    let mut timezone = detect_timezone_local(&timezones).unwrap_or_default();
    let mut hostname = "kwimy".to_string();
    let mut network_label: Option<String> = None;
    let mut username = String::new();
    let mut user_password = String::new();
    let mut luks_password = String::new();
    let mut encrypt_disk = true;
    let mut swap_enabled = true;
    let mut app_flags = AppSelectionFlags::new();
    let mut app_selection = PackageSelection::default();
    let gpu_vendors = detect_gpu_vendors().unwrap_or_default();
    let include_drivers = gpu_vendors.contains(&GpuVendor::Nvidia);
    let mut nvidia_variant: Option<NvidiaVariant> = None;
    let kernel_package = "linux".to_string();
    let kernel_headers = "linux-headers".to_string();
    let mut force_network = false;
    let offline_only = std::env::var("KWIMY_OFFLINE_ONLY").ok().as_deref() == Some("1");

    let mut step = SetupStep::Network;
    'setup: loop {
        let outcome = match step {
            SetupStep::Network => network_step::handle_network_step(
                terminal,
                include_drivers,
                &gpu_vendors,
                selected_disk.as_ref(),
                &keymap,
                &timezone,
                &hostname,
                &username,
                &user_password,
                &luks_password,
                encrypt_disk,
                swap_enabled,
                nvidia_variant,
                &mut network_label,
                &mut force_network,
            )?,
            SetupStep::Drivers => network_step::handle_drivers_step(
                terminal,
                include_drivers,
                network_label.as_deref(),
                selected_disk.as_ref(),
                &keymap,
                &timezone,
                &hostname,
                &username,
                &user_password,
                &luks_password,
                encrypt_disk,
                swap_enabled,
                nvidia_variant,
                &mut force_network,
                &mut nvidia_variant,
            )?,
            SetupStep::Disk => {
                let selected_disk_snapshot = selected_disk.clone();
                disk_step::handle_disk_step(
                    terminal,
                    &disks,
                    include_drivers,
                    &gpu_vendors,
                    network_label.as_deref(),
                    selected_disk_snapshot.as_ref(),
                    &keymap,
                    &timezone,
                    &hostname,
                    &username,
                    &user_password,
                    &luks_password,
                    encrypt_disk,
                    swap_enabled,
                    nvidia_variant,
                    &mut selected_disk,
                    &mut force_network,
                )?
            }
            SetupStep::ConfirmDisk => disk_step::handle_confirm_disk_step(
                terminal,
                include_drivers,
                network_label.as_deref(),
                selected_disk.as_ref(),
                &keymap,
                &timezone,
                &hostname,
                &username,
                &user_password,
                &luks_password,
                encrypt_disk,
                swap_enabled,
                nvidia_variant,
            )?,
            SetupStep::Keymap => {
                let keymap_snapshot = keymap.clone();
                disk_step::handle_keymap_step(
                    terminal,
                    &keymaps,
                    include_drivers,
                    network_label.as_deref(),
                    selected_disk.as_ref(),
                    &keymap_snapshot,
                    &timezone,
                    &hostname,
                    &username,
                    &user_password,
                    &luks_password,
                    encrypt_disk,
                    swap_enabled,
                    nvidia_variant,
                    &mut keymap,
                )?
            }
            SetupStep::Timezone => {
                let timezone_snapshot = timezone.clone();
                disk_step::handle_timezone_step(
                    terminal,
                    &timezones,
                    include_drivers,
                    network_label.as_deref(),
                    selected_disk.as_ref(),
                    &keymap,
                    &timezone_snapshot,
                    &hostname,
                    &username,
                    &user_password,
                    &luks_password,
                    encrypt_disk,
                    swap_enabled,
                    nvidia_variant,
                    &mut timezone,
                )?
            }
            SetupStep::Hostname => {
                let hostname_snapshot = hostname.clone();
                identity_step::handle_hostname_step(
                    terminal,
                    include_drivers,
                    network_label.as_deref(),
                    selected_disk.as_ref(),
                    &keymap,
                    &timezone,
                    &hostname_snapshot,
                    &username,
                    &user_password,
                    &luks_password,
                    encrypt_disk,
                    swap_enabled,
                    nvidia_variant,
                    &mut hostname,
                )?
            }
            SetupStep::Username => {
                let username_snapshot = username.clone();
                identity_step::handle_username_step(
                    terminal,
                    include_drivers,
                    network_label.as_deref(),
                    selected_disk.as_ref(),
                    &keymap,
                    &timezone,
                    &hostname,
                    &username_snapshot,
                    &user_password,
                    &luks_password,
                    encrypt_disk,
                    swap_enabled,
                    nvidia_variant,
                    &mut username,
                )?
            }
            SetupStep::UserPassword => {
                let user_password_snapshot = user_password.clone();
                identity_step::handle_user_password_step(
                    terminal,
                    include_drivers,
                    network_label.as_deref(),
                    selected_disk.as_ref(),
                    &keymap,
                    &timezone,
                    &hostname,
                    &username,
                    &user_password_snapshot,
                    &luks_password,
                    encrypt_disk,
                    swap_enabled,
                    nvidia_variant,
                    &mut user_password,
                )?
            }
            SetupStep::EncryptDisk => {
                let luks_password_snapshot = luks_password.clone();
                identity_step::handle_encrypt_disk_step(
                    terminal,
                    include_drivers,
                    network_label.as_deref(),
                    selected_disk.as_ref(),
                    &keymap,
                    &timezone,
                    &hostname,
                    &username,
                    &user_password,
                    &luks_password_snapshot,
                    encrypt_disk,
                    swap_enabled,
                    nvidia_variant,
                    &mut encrypt_disk,
                    &mut luks_password,
                )?
            }
            SetupStep::LuksPassword => {
                let luks_password_snapshot = luks_password.clone();
                identity_step::handle_luks_password_step(
                    terminal,
                    include_drivers,
                    network_label.as_deref(),
                    selected_disk.as_ref(),
                    &keymap,
                    &timezone,
                    &hostname,
                    &username,
                    &user_password,
                    &luks_password_snapshot,
                    swap_enabled,
                    nvidia_variant,
                    &mut luks_password,
                )?
            }
            SetupStep::Swap => identity_step::handle_swap_step(
                terminal,
                include_drivers,
                network_label.as_deref(),
                selected_disk.as_ref(),
                &keymap,
                &timezone,
                &hostname,
                &username,
                &user_password,
                &luks_password,
                encrypt_disk,
                swap_enabled,
                nvidia_variant,
                &mut swap_enabled,
            )?,
            SetupStep::Applications => apps_step::handle_applications_step(
                terminal,
                include_drivers,
                network_label.as_deref(),
                selected_disk.as_ref(),
                &keymap,
                &timezone,
                &hostname,
                &username,
                &user_password,
                &luks_password,
                encrypt_disk,
                swap_enabled,
                nvidia_variant,
                &mut app_flags,
                &mut app_selection,
            )?,
            SetupStep::Review => apps_step::handle_review_step(
                terminal,
                network_label.as_deref(),
                selected_disk.as_ref(),
                encrypt_disk,
                &gpu_vendors,
                nvidia_variant,
                swap_enabled,
                &hostname,
                &username,
                &keymap,
                &timezone,
                &app_flags,
                &app_selection,
            )?,
        };

        match outcome {
            StepOutcome::Next(next) => step = next,
            StepOutcome::Quit => {
                disable_raw_mode().context("disable raw mode")?;
                let _ = clear_screen();
                return Ok(None);
            }
            StepOutcome::Finish => break 'setup,
        }
    }

    let selected_disk = selected_disk.expect("disk selection");
    let config = apps_step::build_install_config(
        &selected_disk,
        keymap,
        timezone,
        hostname,
        username,
        user_password,
        luks_password,
        encrypt_disk,
        swap_enabled,
        &gpu_vendors,
        nvidia_variant,
        kernel_package,
        kernel_headers,
        base_packages,
        &app_flags,
        app_selection,
        offline_only,
    );

    Ok(Some(config))
}
