use chrono::{DateTime, Utc};

/// Format a DateTime<Utc> to a human-readable string like "Today 14:32" or "Yesterday 09:00".
pub fn format_time(dt: &DateTime<Utc>) -> String {
    let now = Utc::now();
    let today = now.date_naive();
    let dt_date = dt.date_naive();

    let time_str = dt.format("%H:%M").to_string();

    if dt_date == today {
        format!("Today {}", time_str)
    } else if (today - dt_date).num_days() == 1 {
        format!("Yesterday {}", time_str)
    } else {
        format!("{} {}", dt_date.format("%d %b"), time_str)
    }
}

/// Format a relative time string like "2h ago", "5m ago", "Just now".
pub fn format_relative(dt: &DateTime<Utc>) -> String {
    let now = Utc::now();
    let diff = now.signed_duration_since(*dt);
    let minutes = diff.num_minutes();

    if minutes < 1 {
        "Just now".to_string()
    } else if minutes < 60 {
        format!("{}m ago", minutes)
    } else if minutes < 1440 {
        format!("{}h ago", minutes / 60)
    } else {
        format!("{}d ago", minutes / 1440)
    }
}

/// Format a NaiveDate to display string.
pub fn format_date(date: &chrono::NaiveDate) -> String {
    date.format("%d %b %Y").to_string()
}
