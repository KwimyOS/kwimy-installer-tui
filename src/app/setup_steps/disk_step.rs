use std::collections::HashSet;

use anyhow::Result;
use ratatui::backend::CrosstermBackend;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::Terminal;

use crate::disks::DiskInfo;
use crate::drivers::{GpuVendor, NvidiaVariant};
use crate::keymaps::{find_keymap_index, load_keymaps};
use crate::timezones::{detect_timezone_geoip, find_timezone_index};
use crate::ui::{
    render_timezone_loading, run_confirm_selector, run_disk_selector, run_keymap_selector,
    run_timezone_selector, ConfirmAction, SelectionAction,
};

use super::super::steps::{build_install_summary, SetupStep};
use super::super::validation::is_utc_variant;
use super::StepOutcome;

#[allow(clippy::too_many_arguments)]
pub(in crate::app) fn handle_disk_step(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    disks: &[DiskInfo],
    include_drivers: bool,
    gpu_vendors: &HashSet<GpuVendor>,
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
    selected_disk_mut: &mut Option<DiskInfo>,
    force_network: &mut bool,
) -> Result<StepOutcome> {
    let summary = build_install_summary(
        SetupStep::Disk,
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
    match run_disk_selector(terminal, disks, 0, &summary)? {
        SelectionAction::Submit(index) => {
            *selected_disk_mut = disks.get(index).cloned();
            Ok(StepOutcome::Next(SetupStep::ConfirmDisk))
        }
        SelectionAction::Back => {
            if gpu_vendors.contains(&GpuVendor::Nvidia) {
                Ok(StepOutcome::Next(SetupStep::Drivers))
            } else {
                *force_network = true;
                Ok(StepOutcome::Next(SetupStep::Network))
            }
        }
        SelectionAction::Quit => Ok(StepOutcome::Quit),
    }
}

#[allow(clippy::too_many_arguments)]
pub(in crate::app) fn handle_confirm_disk_step(
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
) -> Result<StepOutcome> {
    let Some(disk) = selected_disk else {
        return Ok(StepOutcome::Next(SetupStep::Disk));
    };

    let summary = build_install_summary(
        SetupStep::ConfirmDisk,
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
    let warning_lines = vec![
        Line::from(Span::styled(
            "This will ERASE the selected disk:",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled(" ", Style::default().fg(Color::White)),
            Span::styled(" 󰋊  ", Style::default().fg(Color::LightBlue)),
            Span::styled(disk.label(), Style::default().add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
    ];
    let info_lines = vec![
        Line::from(Span::styled(
            "All data on this disk will be lost. This action cannot be undone.",
            Style::default().fg(Color::Magenta),
        )),
        Line::from(Span::styled(
            "Choose Yes to continue or No to go back",
            Style::default().fg(Color::White),
        )),
    ];

    match run_confirm_selector(
        terminal,
        "Confirm disk erase",
        &warning_lines,
        &info_lines,
        &summary,
    )? {
        ConfirmAction::Yes => Ok(StepOutcome::Next(SetupStep::Keymap)),
        ConfirmAction::No | ConfirmAction::Back => Ok(StepOutcome::Next(SetupStep::Disk)),
        ConfirmAction::Quit => Ok(StepOutcome::Quit),
    }
}

#[allow(clippy::too_many_arguments)]
pub(in crate::app) fn handle_keymap_step(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    keymaps: &[String],
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
    keymap_mut: &mut String,
) -> Result<StepOutcome> {
    let initial = find_keymap_index(keymaps, keymap).unwrap_or(0);
    let summary = build_install_summary(
        SetupStep::Keymap,
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
    match run_keymap_selector(terminal, keymaps, initial, &summary)? {
        SelectionAction::Submit(index) => {
            if let Some(value) = keymaps.get(index) {
                *keymap_mut = value.to_string();
            }
            Ok(StepOutcome::Next(SetupStep::Timezone))
        }
        SelectionAction::Back => Ok(StepOutcome::Next(SetupStep::ConfirmDisk)),
        SelectionAction::Quit => Ok(StepOutcome::Quit),
    }
}

#[allow(clippy::too_many_arguments)]
pub(in crate::app) fn handle_timezone_step(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    timezones: &[String],
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
    timezone_mut: &mut String,
) -> Result<StepOutcome> {
    if timezone.is_empty() || is_utc_variant(timezone) {
        if std::env::var("KWIMY_SKIP_NETWORK").ok().as_deref() != Some("1")
            && std::env::var("KWIMY_OFFLINE_ONLY").ok().as_deref() != Some("1")
        {
            render_timezone_loading(
                terminal,
                &build_install_summary(
                    SetupStep::Timezone,
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
                ),
            )?;
        }

        let _ = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("/run/kwimy/timezone-detect.log")
            .and_then(|mut file| {
                use std::io::Write;
                writeln!(file, "detect_timezone: retry at timezone step")
            });

        if let Some(value) = detect_timezone_geoip(timezones) {
            *timezone_mut = value;
        }
    }

    let initial = find_timezone_index(timezones, timezone_mut).unwrap_or(0);
    let summary = build_install_summary(
        SetupStep::Timezone,
        include_drivers,
        network_label,
        selected_disk,
        keymap,
        timezone_mut,
        hostname,
        username,
        user_password,
        luks_password,
        encrypt_disk,
        swap_enabled,
        nvidia_variant,
    );
    match run_timezone_selector(terminal, timezones, initial, &summary)? {
        SelectionAction::Submit(index) => {
            if let Some(value) = timezones.get(index) {
                *timezone_mut = value.to_string();
            }
            Ok(StepOutcome::Next(SetupStep::Hostname))
        }
        SelectionAction::Back => Ok(StepOutcome::Next(SetupStep::Keymap)),
        SelectionAction::Quit => Ok(StepOutcome::Quit),
    }
}

pub(in crate::app) fn load_setup_keymaps() -> Vec<String> {
    load_keymaps().unwrap_or_else(|_| vec!["us".to_string()])
}
