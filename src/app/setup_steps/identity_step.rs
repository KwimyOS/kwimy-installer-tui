use anyhow::Result;
use ratatui::backend::CrosstermBackend;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::Terminal;

use crate::disks::DiskInfo;
use crate::drivers::NvidiaVariant;
use crate::ui::{run_confirm_selector, run_text_input, ConfirmAction, InputAction};

use super::super::steps::{build_install_summary, SetupStep};
use super::super::validation::{valid_hostname, valid_username};
use super::StepOutcome;

#[allow(clippy::too_many_arguments)]
pub(in crate::app) fn handle_hostname_step(
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
    hostname_mut: &mut String,
) -> Result<StepOutcome> {
    let controls = vec![
        Line::from(vec![
            Span::styled("Ctrl+U", Style::default().fg(Color::Cyan)),
            Span::raw(" or "),
            Span::styled("Backspace", Style::default().fg(Color::Cyan)),
            Span::raw(" clears the input "),
            Span::styled("Esc", Style::default().fg(Color::Cyan)),
            Span::raw(" to go back"),
        ]),
        Line::from("Type to enter a hostname"),
    ];
    let info = vec![
        Line::from("Enter hostname (letters, numbers, and hyphens)"),
        Line::from("Example: my-hostname"),
    ];
    let summary = build_install_summary(
        SetupStep::Hostname,
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
    match run_text_input(
        terminal,
        "Hostname",
        &controls,
        &info,
        "Hostname",
        Some(hostname),
        false,
        &summary,
    )? {
        InputAction::Submit(value) => {
            let value = value.trim();
            if value.is_empty() {
                *hostname_mut = "kwimy".to_string();
                Ok(StepOutcome::Next(SetupStep::Username))
            } else if valid_hostname(value) {
                *hostname_mut = value.to_string();
                Ok(StepOutcome::Next(SetupStep::Username))
            } else {
                Ok(StepOutcome::Next(SetupStep::Hostname))
            }
        }
        InputAction::Back => Ok(StepOutcome::Next(SetupStep::Timezone)),
        InputAction::Quit => Ok(StepOutcome::Quit),
    }
}

#[allow(clippy::too_many_arguments)]
pub(in crate::app) fn handle_username_step(
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
    username_mut: &mut String,
) -> Result<StepOutcome> {
    let controls = vec![
        Line::from(vec![
            Span::styled("Ctrl+U", Style::default().fg(Color::Cyan)),
            Span::raw(" or "),
            Span::styled("Backspace", Style::default().fg(Color::Cyan)),
            Span::raw(" clears the input "),
            Span::styled("Esc", Style::default().fg(Color::Cyan)),
            Span::raw(" to go back"),
        ]),
        Line::from("Type to enter your username"),
    ];
    let info = vec![
        Line::from("Use lowercase letters, numbers, and hyphens only"),
        Line::from("Example: kevin"),
    ];
    let summary = build_install_summary(
        SetupStep::Username,
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
    match run_text_input(
        terminal,
        "User account",
        &controls,
        &info,
        "Username",
        Some(username),
        false,
        &summary,
    )? {
        InputAction::Submit(value) => {
            let value = value.trim();
            if valid_username(value) {
                *username_mut = value.to_string();
                Ok(StepOutcome::Next(SetupStep::UserPassword))
            } else {
                Ok(StepOutcome::Next(SetupStep::Username))
            }
        }
        InputAction::Back => Ok(StepOutcome::Next(SetupStep::Hostname)),
        InputAction::Quit => Ok(StepOutcome::Quit),
    }
}

#[allow(clippy::too_many_arguments)]
pub(in crate::app) fn handle_user_password_step(
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
    user_password_mut: &mut String,
) -> Result<StepOutcome> {
    let controls = vec![
        Line::from(vec![
            Span::styled("Ctrl+U", Style::default().fg(Color::Cyan)),
            Span::raw(" or "),
            Span::styled("Backspace", Style::default().fg(Color::Cyan)),
            Span::raw(" clears the input "),
            Span::styled("Esc", Style::default().fg(Color::Cyan)),
            Span::raw(" to go back"),
        ]),
        Line::from("Type to enter your password"),
    ];
    let info = vec![
        Line::from("Set a password for the sudo user"),
        Line::from("Press Enter to submit"),
    ];
    let summary = build_install_summary(
        SetupStep::UserPassword,
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
    match run_text_input(
        terminal,
        "User password",
        &controls,
        &info,
        "Password",
        None,
        true,
        &summary,
    )? {
        InputAction::Submit(value) => {
            if value.is_empty() {
                return Ok(StepOutcome::Next(SetupStep::UserPassword));
            }
            let confirm_controls = vec![
                Line::from(vec![
                    Span::styled("Ctrl+U", Style::default().fg(Color::Cyan)),
                    Span::raw(" or "),
                    Span::styled("Backspace", Style::default().fg(Color::Cyan)),
                    Span::raw(" clears the input "),
                    Span::styled("Esc", Style::default().fg(Color::Cyan)),
                    Span::raw(" to go back"),
                ]),
                Line::from("Type to confirm your password"),
            ];
            let confirm_info = vec![Line::from("Re-enter the password to confirm")];
            let summary = build_install_summary(
                SetupStep::UserPassword,
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
            match run_text_input(
                terminal,
                "Confirm password",
                &confirm_controls,
                &confirm_info,
                "Re-enter password",
                None,
                true,
                &summary,
            )? {
                InputAction::Submit(confirm) => {
                    if confirm == value {
                        *user_password_mut = value;
                        Ok(StepOutcome::Next(SetupStep::EncryptDisk))
                    } else {
                        Ok(StepOutcome::Next(SetupStep::UserPassword))
                    }
                }
                InputAction::Back => Ok(StepOutcome::Next(SetupStep::UserPassword)),
                InputAction::Quit => Ok(StepOutcome::Quit),
            }
        }
        InputAction::Back => Ok(StepOutcome::Next(SetupStep::Username)),
        InputAction::Quit => Ok(StepOutcome::Quit),
    }
}

#[allow(clippy::too_many_arguments)]
pub(in crate::app) fn handle_encrypt_disk_step(
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
    encrypt_disk_mut: &mut bool,
    luks_password_mut: &mut String,
) -> Result<StepOutcome> {
    let info_lines = vec![
        Line::from("Encrypt the disk with a LUKS passphrase"),
        Line::from("Highly recommended to protect your data at rest"),
        Line::from("Choose Yes to set a passphrase or No to skip"),
    ];
    let warning_lines: Vec<Line> = Vec::new();
    let summary = build_install_summary(
        SetupStep::EncryptDisk,
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
    match run_confirm_selector(
        terminal,
        "Disk encryption",
        &warning_lines,
        &info_lines,
        &summary,
    )? {
        ConfirmAction::Yes => {
            *encrypt_disk_mut = true;
            Ok(StepOutcome::Next(SetupStep::LuksPassword))
        }
        ConfirmAction::No => {
            *encrypt_disk_mut = false;
            luks_password_mut.clear();
            Ok(StepOutcome::Next(SetupStep::Swap))
        }
        ConfirmAction::Back => Ok(StepOutcome::Next(SetupStep::UserPassword)),
        ConfirmAction::Quit => Ok(StepOutcome::Quit),
    }
}

#[allow(clippy::too_many_arguments)]
pub(in crate::app) fn handle_luks_password_step(
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
    swap_enabled: bool,
    nvidia_variant: Option<NvidiaVariant>,
    luks_password_mut: &mut String,
) -> Result<StepOutcome> {
    let controls = vec![
        Line::from(vec![
            Span::styled("Ctrl+U", Style::default().fg(Color::Cyan)),
            Span::raw(" or "),
            Span::styled("Backspace", Style::default().fg(Color::Cyan)),
            Span::raw(" clears the input "),
            Span::styled("Esc", Style::default().fg(Color::Cyan)),
            Span::raw(" to go back"),
        ]),
        Line::from("Type to enter the disk passphrase"),
    ];
    let info = vec![
        Line::from("Set a disk encryption passphrase"),
        Line::from("This unlocks your system at boot"),
    ];
    let summary = build_install_summary(
        SetupStep::LuksPassword,
        include_drivers,
        network_label,
        selected_disk,
        keymap,
        timezone,
        hostname,
        username,
        user_password,
        luks_password,
        true,
        swap_enabled,
        nvidia_variant,
    );
    match run_text_input(
        terminal,
        "Disk encryption passphrase",
        &controls,
        &info,
        "Encryption passphras",
        None,
        true,
        &summary,
    )? {
        InputAction::Submit(value) => {
            if value.is_empty() {
                return Ok(StepOutcome::Next(SetupStep::LuksPassword));
            }
            let confirm_controls = vec![
                Line::from(vec![
                    Span::styled("Ctrl+U", Style::default().fg(Color::Cyan)),
                    Span::raw(" or "),
                    Span::styled("Backspace", Style::default().fg(Color::Cyan)),
                    Span::raw(" clears the input "),
                    Span::styled("Esc", Style::default().fg(Color::Cyan)),
                    Span::raw(" to go back"),
                ]),
                Line::from("Type to confirm the passphrase"),
            ];
            let confirm_info = vec![Line::from("Re-enter the passphrase to confirm")];
            let summary = build_install_summary(
                SetupStep::LuksPassword,
                include_drivers,
                network_label,
                selected_disk,
                keymap,
                timezone,
                hostname,
                username,
                user_password,
                luks_password,
                true,
                swap_enabled,
                nvidia_variant,
            );
            match run_text_input(
                terminal,
                "Confirm passphrase",
                &confirm_controls,
                &confirm_info,
                "Re-enter encryption passphras",
                None,
                true,
                &summary,
            )? {
                InputAction::Submit(confirm) => {
                    if confirm == value {
                        *luks_password_mut = value;
                        Ok(StepOutcome::Next(SetupStep::Swap))
                    } else {
                        Ok(StepOutcome::Next(SetupStep::LuksPassword))
                    }
                }
                InputAction::Back => Ok(StepOutcome::Next(SetupStep::LuksPassword)),
                InputAction::Quit => Ok(StepOutcome::Quit),
            }
        }
        InputAction::Back => Ok(StepOutcome::Next(SetupStep::EncryptDisk)),
        InputAction::Quit => Ok(StepOutcome::Quit),
    }
}

#[allow(clippy::too_many_arguments)]
pub(in crate::app) fn handle_swap_step(
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
    swap_enabled_mut: &mut bool,
) -> Result<StepOutcome> {
    let info_lines = vec![
        Line::from("Enable zram-based swap (in-memory compressed)"),
        Line::from("Recommended to improve responsiveness under memory pressure"),
    ];
    let warning_lines: Vec<Line> = Vec::new();
    let summary = build_install_summary(
        SetupStep::Swap,
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
    match run_confirm_selector(
        terminal,
        "Enable swap",
        &warning_lines,
        &info_lines,
        &summary,
    )? {
        ConfirmAction::Yes => {
            *swap_enabled_mut = true;
            Ok(StepOutcome::Next(SetupStep::Applications))
        }
        ConfirmAction::No => {
            *swap_enabled_mut = false;
            Ok(StepOutcome::Next(SetupStep::Applications))
        }
        ConfirmAction::Back => {
            if encrypt_disk {
                Ok(StepOutcome::Next(SetupStep::LuksPassword))
            } else {
                Ok(StepOutcome::Next(SetupStep::EncryptDisk))
            }
        }
        ConfirmAction::Quit => Ok(StepOutcome::Quit),
    }
}
