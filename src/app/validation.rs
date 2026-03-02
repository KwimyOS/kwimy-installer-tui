pub(crate) fn is_utc_variant(value: &str) -> bool {
    matches!(value, "UTC" | "Etc/UTC" | "Etc/GMT" | "GMT")
}

pub(crate) fn valid_username(value: &str) -> bool {
    if value.is_empty() || value == "root" {
        return false;
    }
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !first.is_ascii_lowercase() {
        return false;
    }
    chars.all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_' || ch == '-')
}

pub(crate) fn valid_hostname(value: &str) -> bool {
    if value.is_empty() || value.len() > 63 {
        return false;
    }
    value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '-')
}

pub(crate) fn is_wifi_auth_error(message: &str) -> bool {
    let msg = message.to_lowercase();
    msg.contains("password")
        || msg.contains("secrets")
        || msg.contains("auth")
        || msg.contains("authentication")
        || msg.contains("access denied")
}
