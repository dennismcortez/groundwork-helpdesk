use chrono::{NaiveDateTime, Utc};

pub struct SlaStatus {
    pub label: String,
    pub color: String,
    pub bg: String,
}

pub fn get_sla_status(priority: &str, status: &str, opened_at: &str) -> SlaStatus {
    if status == "Resolved" {
        return SlaStatus {
            label: "Resolved".to_string(),
            color: "#4A6626".to_string(),
            bg: "#DCE7C8".to_string(),
        };
    }

    let target_hours = match priority {
        "High" => 4.0,
        "Medium" => 24.0,
        "Low" => 72.0,
        _ => 24.0,
    };

    let opened = NaiveDateTime::parse_from_str(opened_at, "%Y-%m-%d %H:%M:%S")
        .unwrap_or_else(|_| Utc::now().naive_utc());
    let elapsed = Utc::now()
        .naive_utc()
        .signed_duration_since(opened)
        .num_seconds() as f64 / 3600.0;
    let percent_used = elapsed / target_hours;

    if percent_used < 0.5 {
        SlaStatus {
            label: "On track".to_string(),
            color: "#8A4A12".to_string(),
            bg: "#F7DCC0".to_string(),
        }
    } else if percent_used < 0.9 {
        SlaStatus {
            label: "Due soon".to_string(),
            color: "#8A5A0A".to_string(),
            bg: "#F6D9B0".to_string(),
        }
    } else {
        SlaStatus {
            label: "Overdue".to_string(),
            color: "#8A2E1E".to_string(),
            bg: "#F2C6BC".to_string(),
        }
    }
}
