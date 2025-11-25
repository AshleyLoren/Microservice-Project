use crate::user_models::{User, UploadedFile};
use anyhow::{Context, Result, bail};
use std::fs;
use std::path::Path;
use tokio::sync::RwLock;

const USERS_FILE: &str = "users.json";
const FILES_FILE: &str = "uploaded_files.json";

pub struct UserStorage {
    users: RwLock<Vec<User>>,
    files: RwLock<Vec<UploadedFile>>,
}

impl UserStorage {
    pub fn new() -> Result<Self> {
        let users = if Path::new(USERS_FILE).exists() {
            let data = fs::read_to_string(USERS_FILE)
                .context("Failed to read users file")?;
            serde_json::from_str(&data)
                .context("Failed to parse users file")?
        } else {
            Vec::new()
        };

        let files = if Path::new(FILES_FILE).exists() {
            let data = fs::read_to_string(FILES_FILE)
                .context("Failed to read files file")?;
            serde_json::from_str(&data)
                .context("Failed to parse files file")?
        } else {
            Vec::new()
        };

        Ok(Self {
            users: RwLock::new(users),
            files: RwLock::new(files),
        })
    }

    pub async fn create_user(&self, user: User) -> Result<User> {
        let mut users = self.users.write().await;
        
        if users.iter().any(|u| u.username == user.username) {
            bail!("Username already exists");
        }
        
        users.push(user.clone());
        self.save_users_to_disk(&users)?;
        Ok(user)
    }

    pub async fn get_user_by_username(&self, username: &str) -> Result<Option<User>> {
        let users = self.users.read().await;
        Ok(users.iter().find(|u| u.username == username).cloned())
    }

    pub async fn add_file(&self, file: UploadedFile) -> Result<UploadedFile> {
        let mut files = self.files.write().await;
        files.push(file.clone());
        self.save_files_to_disk(&files)?;
        Ok(file)
    }

    pub async fn get_user_files(&self, user_id: &str) -> Result<Vec<UploadedFile>> {
        let files = self.files.read().await;
        Ok(files.iter().filter(|f| f.user_id == user_id).cloned().collect())
    }

    pub async fn get_file_by_id(&self, file_id: &str, user_id: &str) -> Result<Option<UploadedFile>> {
        let files = self.files.read().await;
        Ok(files.iter().find(|f| f.id == file_id && f.user_id == user_id).cloned())
    }

    pub async fn add_tag_to_file(&self, file_id: &str, user_id: &str, tag: String) -> Result<()> {
        let mut files = self.files.write().await;
        
        if let Some(file) = files.iter_mut().find(|f| f.id == file_id && f.user_id == user_id) {
            if !file.tags.contains(&tag) {
                file.tags.push(tag);
                self.save_files_to_disk(&files)?;
            }
        } else {
            bail!("File not found");
        }
        
        Ok(())
    }

    pub async fn remove_tag_from_file(&self, file_id: &str, user_id: &str, tag: &str) -> Result<()> {
        let mut files = self.files.write().await;
        
        if let Some(file) = files.iter_mut().find(|f| f.id == file_id && f.user_id == user_id) {
            file.tags.retain(|t| t != tag);
            self.save_files_to_disk(&files)?;
        } else {
            bail!("File not found");
        }
        
        Ok(())
    }

    fn save_users_to_disk(&self, users: &[User]) -> Result<()> {
        let json = serde_json::to_string_pretty(users)
            .context("Failed to serialize users")?;
        fs::write(USERS_FILE, json)
            .context("Failed to write to users file")?;
        Ok(())
    }

    fn save_files_to_disk(&self, files: &[UploadedFile]) -> Result<()> {
        let json = serde_json::to_string_pretty(files)
            .context("Failed to serialize files")?;
        fs::write(FILES_FILE, json)
            .context("Failed to write to files file")?;
        Ok(())
    }
}
