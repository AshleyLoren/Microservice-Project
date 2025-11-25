use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
}

impl User {
    pub fn new(username: String, password_hash: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            username,
            password_hash,
            created_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadedFile {
    pub id: String,
    pub user_id: String,
    pub filename: String,
    pub content: String,
    pub uploaded_at: DateTime<Utc>,
    #[serde(default)]
    pub tags: Vec<String>,
}

impl UploadedFile {
    pub fn new(user_id: String, filename: String, content: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            user_id,
            filename,
            content,
            uploaded_at: Utc::now(),
            tags: Vec::new(),
        }
    }
}
