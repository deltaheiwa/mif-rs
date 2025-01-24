use chrono::TimeDelta;

/// Return a discord-compatible timestamp string
///
/// - <t:timestamp:R> for relative time
///
/// '12 minutes ago', '2 hours ago', '1 day ago', etc.
pub fn get_relative_timestamp(timestamp: &i64) -> String {
    format!("<t:{}:R>", timestamp)
}

pub fn get_long_date(timestamp: &i64) -> String {
    format!("<t:{}:D>", timestamp)
}

pub fn pretty_time_delta(delta: &TimeDelta) -> String {
    let total_seconds = delta.num_seconds().abs();
    let mut remaining_seconds = total_seconds;
    let mut parts = Vec::new();

    let days = remaining_seconds / 86400; // 86400 seconds in a day
    if days > 0 {
        parts.push(format!("{} day{}", days, if days > 1 { "s" } else { "" }));
        remaining_seconds %= 86400;
    }

    let hours = remaining_seconds / 3600; // 3600 seconds in an hour
    if hours > 0 {
        parts.push(format!("{} hour{}", hours, if hours > 1 { "s" } else { "" }));
        remaining_seconds %= 3600;
    }

    let minutes = remaining_seconds / 60; // 60 seconds in a minute
    if minutes > 0 {
        parts.push(format!("{} minute{}", minutes, if minutes > 1 { "s" } else { "" }));
        remaining_seconds %= 60;
    }

    if remaining_seconds > 0 || parts.is_empty() {
        parts.push(format!(
            "{} second{}",
            remaining_seconds,
            if remaining_seconds != 1 { "s" } else { "" }
        ));
    }

    let sign = if delta.num_seconds() < 0 { "-" } else { "" };
    format!("{}{}", sign, parts.join(", "))
}