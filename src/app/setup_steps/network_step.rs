use std::collections::HashSet;
use std::thread;
use std::time::{Duration, Instant};

use anyhow::Result;
use ratatui::backend::CrosstermBackend;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::Terminal;

use crate::disks::DiskInfo;
use crate::drivers::{GpuVendor, NvidiaVariant};
use crate::network::{
    active_connection_label, connect_wifi_profile, disconnect_wifi_device, forget_wifi_connection,
    has_wifi_device, is_network_ready, is_wifi_connected, list_wifi_networks, wifi_device_name,
    wifi_device_state,
};
use crate::ui::{
    render_text_input, render_wifi_connecting, render_wifi_searching, run_network_required,
    run_nvidia_selector, run_text_input, run_wifi_selector, InputAction, NetworkAction,
    NvidiaAction, WifiAction, SPINNER, SPINNER_LEN,
};

use super::super::steps::{build_install_summary, SetupStep};
use super::super::validation::is_wifi_auth_error;
use super::StepOutcome;

#[allow(clippy::too_many_arguments)]
pub(in crate::app) fn handle_network_step(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    include_drivers: bool,
    gpu_vendors: &HashSet<GpuVendor>,
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
    network_label: &mut Option<String>,
    force_network: &mut bool,
) -> Result<StepOutcome> {
    if std::env::var("KWIMY_SKIP_NETWORK").ok().as_deref() == Some("1") {
        *network_label = Some("Skipped (dev)".to_string());
        if gpu_vendors.contains(&GpuVendor::Nvidia) {
            return Ok(StepOutcome::Next(SetupStep::Drivers));
        }
        return Ok(StepOutcome::Next(SetupStep::Disk));
    }

    let mut editing_network = *force_network;
    *force_network = false;
    if editing_network && !has_wifi_device().unwrap_or(false) {
        editing_network = false;
    }

    if !editing_network && is_network_ready().unwrap_or(false) {
        if network_label.is_none() {
            *network_label = active_connection_label().ok().flatten();
            if network_label.is_none() {
                *network_label = Some("Connected".to_string());
            }
        }
        if gpu_vendors.contains(&GpuVendor::Nvidia) {
            return Ok(StepOutcome::Next(SetupStep::Drivers));
        }
        return Ok(StepOutcome::Next(SetupStep::Disk));
    }

    let summary = build_install_summary(
        SetupStep::Network,
        include_drivers,
        network_label.as_deref(),
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

    let wifi_supported = has_wifi_device().unwrap_or(false);
    if !wifi_supported {
        return Ok(match run_network_required(terminal, &summary)? {
            NetworkAction::Retry => StepOutcome::Next(SetupStep::Network),
            NetworkAction::Quit => StepOutcome::Quit,
        });
    }

    let mut status_message: Option<String> = None;
    let mut wifi_connected = false;
    let mut last_connect_at: Option<Instant> = None;

    loop {
        let mut internet_ready = is_network_ready().unwrap_or(false);
        if internet_ready && network_label.is_none() {
            *network_label = active_connection_label().ok().flatten();
            if network_label.is_none() {
                *network_label = Some("Connected".to_string());
            }
        }

        let summary = build_install_summary(
            SetupStep::Network,
            include_drivers,
            network_label.as_deref(),
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
        render_wifi_searching(
            terminal,
            status_message.as_deref(),
            wifi_connected,
            internet_ready,
            &summary,
        )?;

        let networks = match list_wifi_networks() {
            Ok(list) => list,
            Err(err) => {
                status_message = Some(err.to_string());
                Vec::new()
            }
        };

        wifi_connected = networks.iter().any(|network| network.in_use);
        if wifi_connected {
            last_connect_at = None;
        } else if let Some(connected_at) = last_connect_at {
            if connected_at.elapsed() < Duration::from_secs(5) {
                wifi_connected = true;
            } else {
                last_connect_at = None;
            }
        }

        let summary = build_install_summary(
            SetupStep::Network,
            include_drivers,
            network_label.as_deref(),
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

        match run_wifi_selector(
            terminal,
            &networks,
            status_message.as_deref(),
            wifi_connected,
            internet_ready,
            &summary,
        )? {
            WifiAction::Submit(index) => {
                let Some(network) = networks.get(index) else {
                    continue;
                };
                let needs_password = !network.is_open();
                let mut password: Option<String> = None;

                if needs_password {
                    let mut password_error: Option<String> = None;
                    let controls = vec![
                        Line::from(vec![
                            Span::styled("Ctrl+U", Style::default().fg(Color::Cyan)),
                            Span::raw(" or "),
                            Span::styled("Backspace", Style::default().fg(Color::Cyan)),
                            Span::raw(" clears the input"),
                        ]),
                        Line::from(format!("Enter password for \"{}\".", network.ssid)),
                    ];

                    loop {
                        let info = if let Some(error_message) = &password_error {
                            vec![Line::from(Span::styled(
                                error_message,
                                Style::default().fg(Color::Red),
                            ))]
                        } else {
                            vec![Line::from("Press Enter to connect.")]
                        };

                        let summary = build_install_summary(
                            SetupStep::Network,
                            include_drivers,
                            network_label.as_deref(),
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
                            "Wi-Fi password",
                            &controls,
                            &info,
                            "Wi-Fi password",
                            None,
                            true,
                            &summary,
                        )? {
                            InputAction::Submit(value) => {
                                if value.is_empty() {
                                    continue;
                                }
                                let start = Instant::now();
                                let spinner = SPINNER[0];
                                let connecting_info = vec![Line::from(Span::styled(
                                    format!("Connecting... {} (starting)", spinner),
                                    Style::default().fg(Color::Green),
                                ))];
                                render_text_input(
                                    terminal,
                                    "Wi-Fi password",
                                    &controls,
                                    &connecting_info,
                                    "Wi-Fi password",
                                    &value,
                                    true,
                                    &summary,
                                )?;

                                let _ = disconnect_wifi_device();
                                let _ = forget_wifi_connection(&network.ssid);
                                let device = wifi_device_name().ok().flatten();
                                let connection_name = format!("kwimy-{}", network.ssid);

                                match connect_wifi_profile(
                                    &network.ssid,
                                    Some(&value),
                                    device.as_deref(),
                                    Some(&connection_name),
                                ) {
                                    Ok(()) => {
                                        while start.elapsed() < Duration::from_secs(8) {
                                            let spinner_idx = (start.elapsed().as_millis() / 200)
                                                % SPINNER_LEN as u128;
                                            let spinner = SPINNER[spinner_idx as usize];
                                            let state = wifi_device_state()
                                                .ok()
                                                .flatten()
                                                .unwrap_or_else(|| "unknown".to_string());
                                            let connecting_info = vec![Line::from(Span::styled(
                                                format!("Connecting... {} ({})", spinner, state),
                                                Style::default().fg(Color::Green),
                                            ))];
                                            render_text_input(
                                                terminal,
                                                "Wi-Fi password",
                                                &controls,
                                                &connecting_info,
                                                "Wi-Fi password",
                                                &value,
                                                true,
                                                &summary,
                                            )?;
                                            if is_wifi_connected().unwrap_or(false) {
                                                password = Some(value);
                                                wifi_connected = true;
                                                last_connect_at = Some(Instant::now());
                                                break;
                                            }
                                            thread::sleep(Duration::from_millis(200));
                                        }
                                        if password.is_some() {
                                            break;
                                        }
                                        let state = wifi_device_state()
                                            .ok()
                                            .flatten()
                                            .unwrap_or_else(|| "unknown".to_string());
                                        password_error = Some(format!(
                                            "Connection failed (state: {}). Please try again.",
                                            state
                                        ));
                                        continue;
                                    }
                                    Err(err) => {
                                        let err_msg = err.to_string();
                                        if is_wifi_auth_error(&err_msg) {
                                            password_error =
                                                Some("Incorrect password.".to_string());
                                            let _ = forget_wifi_connection(&network.ssid);
                                            continue;
                                        }
                                        status_message = Some(err_msg);
                                        break;
                                    }
                                }
                            }
                            InputAction::Back => break,
                            InputAction::Quit => return Ok(StepOutcome::Quit),
                        }
                    }
                }

                if needs_password && password.is_none() {
                    continue;
                }

                if network.is_open() {
                    let _ = disconnect_wifi_device();
                    let _ = forget_wifi_connection(&network.ssid);
                    let device = wifi_device_name().ok().flatten();
                    let connection_name = format!("kwimy-{}", network.ssid);
                    if let Err(err) = connect_wifi_profile(
                        &network.ssid,
                        None,
                        device.as_deref(),
                        Some(&connection_name),
                    ) {
                        status_message = Some(err.to_string());
                        continue;
                    }

                    let start = Instant::now();
                    while start.elapsed() < Duration::from_secs(8) {
                        let spinner_idx = (start.elapsed().as_millis() / 200) % SPINNER_LEN as u128;
                        let spinner = SPINNER[spinner_idx as usize];
                        let summary = build_install_summary(
                            SetupStep::Network,
                            include_drivers,
                            network_label.as_deref(),
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
                        render_wifi_connecting(
                            terminal,
                            index,
                            &networks,
                            status_message.as_deref(),
                            wifi_connected,
                            internet_ready,
                            &summary,
                            spinner,
                        )?;
                        if is_wifi_connected().unwrap_or(false) {
                            wifi_connected = true;
                            last_connect_at = Some(Instant::now());
                            break;
                        }
                        thread::sleep(Duration::from_millis(200));
                    }
                    if !wifi_connected {
                        status_message = Some("Connection failed. Please try again.".to_string());
                        continue;
                    }
                }

                internet_ready = is_network_ready().unwrap_or(false);
                if internet_ready {
                    *network_label = active_connection_label().ok().flatten();
                    if network_label.is_none() {
                        *network_label = Some(network.ssid.clone());
                    }
                    status_message = None;
                } else {
                    status_message = Some("Connected to Wi-Fi but no internet access.".to_string());
                }
                continue;
            }
            WifiAction::Rescan => status_message = None,
            WifiAction::Refresh => {}
            WifiAction::Continue => {
                if internet_ready {
                    if gpu_vendors.contains(&GpuVendor::Nvidia) {
                        return Ok(StepOutcome::Next(SetupStep::Drivers));
                    }
                    return Ok(StepOutcome::Next(SetupStep::Disk));
                }
            }
            WifiAction::Quit => return Ok(StepOutcome::Quit),
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(in crate::app) fn handle_drivers_step(
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
    force_network: &mut bool,
    nvidia_variant_mut: &mut Option<NvidiaVariant>,
) -> Result<StepOutcome> {
    let summary = build_install_summary(
        SetupStep::Drivers,
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
    match run_nvidia_selector(terminal, &summary)? {
        NvidiaAction::Select(variant) => {
            *nvidia_variant_mut = Some(variant);
            Ok(StepOutcome::Next(SetupStep::Disk))
        }
        NvidiaAction::Skip => {
            *nvidia_variant_mut = None;
            Ok(StepOutcome::Next(SetupStep::Disk))
        }
        NvidiaAction::Back => {
            *force_network = has_wifi_device().unwrap_or(false);
            Ok(StepOutcome::Next(SetupStep::Network))
        }
        NvidiaAction::Quit => Ok(StepOutcome::Quit),
    }
}
