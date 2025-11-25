use crate::models::Reminder;
use anyhow::{Context, Result};
use chrono::Utc;
use std::fs;
use std::path::Path;
use tokio::sync::RwLock;

const STORAGE_FILE: &str = "reminders.json";

pub struct ReminderStorage {
    reminders: RwLock<Vec<Reminder>>,
}

impl ReminderStorage {
    pub fn new() -> Result<Self> {
        let reminders = if Path::new(STORAGE_FILE).exists() {
            let data = fs::read_to_string(STORAGE_FILE)
                .context("Failed to read storage file")?;
            serde_json::from_str(&data)
                .context("Failed to parse storage file")?
        } else {
            Vec::new()
        };

        Ok(Self {
            reminders: RwLock::new(reminders),
        })
    }

    pub async fn add_reminder(&self, reminder: Reminder) -> Result<Reminder> {
        let mut reminders = self.reminders.write().await;
        reminders.push(reminder.clone());
        self.save_to_disk(&reminders)?;
        Ok(reminder)
    }

    pub async fn get_upcoming_reminders(&self) -> Result<Vec<Reminder>> {
        let reminders = self.reminders.read().await;
        let now = Utc::now();
        
        let mut upcoming: Vec<Reminder> = reminders
            .iter()
            .filter(|r| !r.sent && r.due_time > now)
            .cloned()
            .collect();
        
        upcoming.sort_by(|a, b| a.due_time.cmp(&b.due_time));
        Ok(upcoming)
    }

    pub async fn get_due_reminders(&self) -> Result<Vec<Reminder>> {
        let reminders = self.reminders.read().await;
        let now = Utc::now();
        
        Ok(reminders
            .iter()
            .filter(|r| !r.sent && r.due_time <= now)
            .cloned()
            .collect())
    }

    pub async fn mark_as_sent(&self, id: &str) -> Result<()> {
        let mut reminders = self.reminders.write().await;
        
        if let Some(reminder) = reminders.iter_mut().find(|r| r.id == id) {
            reminder.sent = true;
            self.save_to_disk(&reminders)?;
        }
        
        Ok(())
    }

    pub async fn reschedule_reminder(&self, id: &str, next_due_time: chrono::DateTime<Utc>) -> Result<()> {
        let mut reminders = self.reminders.write().await;
        
        if let Some(reminder) = reminders.iter_mut().find(|r| r.id == id) {
            reminder.due_time = next_due_time;
            reminder.sent = false;
            self.save_to_disk(&reminders)?;
        }
        
        Ok(())
    }

    pub async fn get_all_reminders(&self) -> Result<Vec<Reminder>> {
        let reminders = self.reminders.read().await;
        Ok(reminders.clone())
    }

    fn save_to_disk(&self, reminders: &[Reminder]) -> Result<()> {
        let json = serde_json::to_string_pretty(reminders)
            .context("Failed to serialize reminders")?;
        fs::write(STORAGE_FILE, json)
            .context("Failed to write to storage file")?;
        Ok(())
    }
}
