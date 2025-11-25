use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reminder {
    pub id: String,
    pub message: String,
    pub due_time: DateTime<Utc>,
    pub username: Option<String>,
    pub sent: bool,
    pub created_at: DateTime<Utc>,
    pub recurrence: Option<String>,
}

impl Reminder {
    pub fn new(message: String, due_time: DateTime<Utc>, username: Option<String>, recurrence: Option<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            message,
            due_time,
            username,
            sent: false,
            created_at: Utc::now(),
            recurrence,
        }
    }

    pub fn calculate_next_occurrence(&self) -> Option<DateTime<Utc>> {
        let recurrence = self.recurrence.as_ref()?;
        let now = Utc::now();
        
        // Calculate the interval duration
        let interval = match recurrence.to_lowercase().as_str() {
            "daily" => Duration::days(1),
            "weekly" => Duration::weeks(1),
            custom if custom.ends_with("minutes") || custom.parse::<i64>().is_ok() => {
                let minutes: i64 = if custom.ends_with("minutes") {
                    custom.trim_end_matches("minutes").trim().parse().ok()?
                } else {
                    custom.parse().ok()?
                };
                
                // Safeguard: reject non-positive intervals
                if minutes <= 0 {
                    eprintln!("Warning: Invalid recurrence interval {} minutes, skipping reschedule", minutes);
                    return None;
                }
                
                Duration::minutes(minutes)
            }
            _ => return None,
        };

        // Keep adding the interval until we get a future time
        let mut next_time = self.due_time + interval;
        while next_time <= now {
            next_time = next_time + interval;
        }
        
        Some(next_time)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateReminderRequest {
    pub message: String,
    pub due_time: String,
    pub username: Option<String>,
    pub recurrence: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateReminderResponse {
    pub id: String,
    pub message: String,
    pub due_time: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReminderListResponse {
    pub reminders: Vec<Reminder>,
}
