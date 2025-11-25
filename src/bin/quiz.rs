use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

mod user_models {
    include!("../user_models.rs");
}

mod user_storage {
    include!("../user_storage.rs");
}

use user_models::{User, UploadedFile};
use user_storage::UserStorage;

const SESSION_FILE: &str = ".session";

#[derive(Parser)]
#[command(name = "quiz")]
#[command(about = "A CLI tool for managing study quizzes", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Create a new user account")]
    Signup {
        #[arg(short, long, help = "Username")]
        username: String,

        #[arg(short, long, help = "Password")]
        password: String,
    },

    #[command(about = "Log in to your account")]
    Login {
        #[arg(short, long, help = "Username")]
        username: String,

        #[arg(short, long, help = "Password")]
        password: String,
    },

    #[command(about = "Log out of your account")]
    Logout,

    #[command(about = "Upload a text file for quiz generation")]
    Upload {
        #[arg(short, long, help = "Path to the text file")]
        file: String,
    },

    #[command(about = "List your uploaded files")]
    List,

    #[command(about = "Show current user")]
    Whoami,

    #[command(about = "Add tag(s) to a file")]
    Tag {
        #[arg(short, long, help = "File ID")]
        file_id: String,

        #[arg(short, long, help = "Tags (comma-separated)")]
        tags: String,
    },

    #[command(about = "Remove tag from a file")]
    Untag {
        #[arg(short, long, help = "File ID")]
        file_id: String,

        #[arg(short, long, help = "Tag to remove")]
        tag: String,
    },

    #[command(about = "Filter files by tag")]
    FilterByTag {
        #[arg(short, long, help = "Tag to filter by")]
        tag: String,
    },

    #[command(about = "Apply tags to multiple files")]
    BulkTag {
        #[arg(short, long, help = "File IDs (comma-separated)")]
        file_ids: String,

        #[arg(short, long, help = "Tags to add (comma-separated)")]
        tags: String,
    },

    #[command(about = "Remove tag from multiple files")]
    BulkUntag {
        #[arg(short, long, help = "File IDs (comma-separated)")]
        file_ids: String,

        #[arg(short, long, help = "Tag to remove")]
        tag: String,
    },

    #[command(about = "Create a study notification reminder")]
    Notify {
        #[arg(short = 'n', long, help = "Title of the notification")]
        title: String,

        #[arg(short, long, help = "Memo/description (optional)")]
        memo: Option<String>,

        #[arg(short = 't', long, help = "Date and time (ISO 8601, e.g., 2025-11-05T10:00:00Z)")]
        time: String,

        #[arg(short, long, help = "Recurrence pattern: daily, weekly, or custom interval in minutes")]
        recurrence: Option<String>,
    },

    #[command(about = "List your study notifications")]
    ListNotifications,
}

#[derive(Debug, Serialize, Deserialize)]
struct Session {
    user_id: String,
    username: String,
}

impl Session {
    fn save(&self) -> Result<()> {
        let json = serde_json::to_string(self)?;
        fs::write(SESSION_FILE, json)?;
        Ok(())
    }

    fn load() -> Option<Self> {
        if Path::new(SESSION_FILE).exists() {
            let data = fs::read_to_string(SESSION_FILE).ok()?;
            serde_json::from_str(&data).ok()
        } else {
            None
        }
    }

    fn clear() -> Result<()> {
        if Path::new(SESSION_FILE).exists() {
            fs::remove_file(SESSION_FILE)?;
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    if let Err(e) = run_command(cli.command).await {
        eprintln!("❌ Error: {}", e);
        std::process::exit(1);
    }
}

async fn run_command(command: Commands) -> Result<()> {
    let storage = UserStorage::new()?;

    match command {
        Commands::Signup { username, password } => {
            signup(&storage, username, password).await?;
        }
        Commands::Login { username, password } => {
            login(&storage, username, password).await?;
        }
        Commands::Logout => {
            logout()?;
        }
        Commands::Upload { file } => {
            let session = require_login()?;
            upload_file(&storage, &session, file).await?;
        }
        Commands::List => {
            let session = require_login()?;
            list_files(&storage, &session).await?;
        }
        Commands::Whoami => {
            whoami()?;
        }
        Commands::Tag { file_id, tags } => {
            let session = require_login()?;
            tag_file(&storage, &session, file_id, tags).await?;
        }
        Commands::Untag { file_id, tag } => {
            let session = require_login()?;
            untag_file(&storage, &session, file_id, tag).await?;
        }
        Commands::FilterByTag { tag } => {
            let session = require_login()?;
            filter_files_by_tag(&storage, &session, tag).await?;
        }
        Commands::BulkTag { file_ids, tags } => {
            let session = require_login()?;
            bulk_tag_files(&storage, &session, file_ids, tags).await?;
        }
        Commands::BulkUntag { file_ids, tag } => {
            let session = require_login()?;
            bulk_untag_files(&storage, &session, file_ids, tag).await?;
        }
        Commands::Notify { title, memo, time, recurrence } => {
            let session = require_login()?;
            create_notification(&session, title, memo, time, recurrence).await?;
        }
        Commands::ListNotifications => {
            let session = require_login()?;
            list_notifications(&session).await?;
        }
    }

    Ok(())
}

async fn signup(storage: &UserStorage, username: String, password: String) -> Result<()> {
    if username.is_empty() {
        bail!("Username cannot be empty");
    }

    if password.len() < 6 {
        bail!("Password must be at least 6 characters long");
    }

    let password_hash = bcrypt::hash(&password, bcrypt::DEFAULT_COST)
        .context("Failed to hash password")?;

    let user = User::new(username.clone(), password_hash);
    storage.create_user(user.clone()).await?;

    println!("✅ Account created successfully!");
    println!("👤 Username: {}", username);
    println!("🆔 User ID: {}", user.id);
    println!("\n💡 You can now log in using: quiz login -u {} -p <password>", username);

    Ok(())
}

async fn login(storage: &UserStorage, username: String, password: String) -> Result<()> {
    let user = storage.get_user_by_username(&username).await?
        .ok_or_else(|| anyhow::anyhow!("Invalid username or password"))?;

    let valid = bcrypt::verify(&password, &user.password_hash)
        .context("Failed to verify password")?;

    if !valid {
        bail!("Invalid username or password");
    }

    let session = Session {
        user_id: user.id.clone(),
        username: user.username.clone(),
    };

    session.save()?;

    println!("✅ Login successful!");
    println!("👤 Welcome back, {}!", user.username);

    Ok(())
}

fn logout() -> Result<()> {
    Session::clear()?;
    println!("✅ Logged out successfully!");
    Ok(())
}

async fn upload_file(storage: &UserStorage, session: &Session, file_path: String) -> Result<()> {
    // Check if file exists
    if !Path::new(&file_path).exists() {
        bail!("File not found: {}", file_path);
    }

    let path = Path::new(&file_path);
    
    // Validate file type - only .txt files allowed
    let extension = path.extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    
    if extension != "txt" {
        bail!("❌ Unsupported file type: .{}\n💡 Only .txt files are allowed. Please upload a text file.", extension);
    }

    // Read file content
    let content = fs::read_to_string(&file_path)
        .context("Failed to read file. The file may be corrupted or unreadable.")?;

    // Check for empty files
    if content.trim().is_empty() {
        bail!("❌ Empty file detected\n💡 The file '{}' contains no content. Please upload a file with text content.", file_path);
    }

    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let file_size_bytes = content.len();
    let file_size_kb = file_size_bytes as f64 / 1024.0;
    
    // Show file preview and metadata
    println!("\n📄 File Preview");
    println!("═══════════════════════════════════════");
    println!("📝 Filename: {}", filename);
    println!("📊 Size: {} characters ({:.2} KB)", file_size_bytes, file_size_kb);
    println!("═══════════════════════════════════════");
    println!("\n📖 Content Preview (first 200 characters):");
    println!("{}", 
        if content.len() > 200 {
            format!("{}...", &content[..200])
        } else {
            content.clone()
        }
    );
    println!("═══════════════════════════════════════\n");

    // Ask for confirmation
    println!("❓ Confirm upload of this file? (yes/no): ");
    use std::io::{self, Write};
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    let confirmed = input.trim().to_lowercase();
    if confirmed != "yes" && confirmed != "y" {
        println!("❌ Upload cancelled");
        return Ok(());
    }

    // Create and store the file
    let uploaded_file = UploadedFile::new(
        session.user_id.clone(),
        filename.clone(),
        content.clone(),
    );

    storage.add_file(uploaded_file.clone()).await?;

    println!("\n✅ File uploaded successfully!");
    println!("📄 Filename: {}", filename);
    println!("🆔 File ID: {}", uploaded_file.id);
    println!("📊 Size: {} characters", content.len());
    println!("\n💡 Use 'quiz list' to see all your uploaded files");

    Ok(())
}

async fn list_files(storage: &UserStorage, session: &Session) -> Result<()> {
    let files = storage.get_user_files(&session.user_id).await?;

    if files.is_empty() {
        println!("📭 No files uploaded yet.");
        println!("💡 Use 'quiz upload -f <file>' to upload a text file");
        return Ok(());
    }

    println!("📚 Your uploaded files:\n");
    for (i, file) in files.iter().enumerate() {
        println!("{}. 📄 {}", i + 1, file.filename);
        println!("   🆔 ID: {}", file.id);
        println!("   📊 Size: {} characters", file.content.len());
        println!("   ⏰ Uploaded: {}", file.uploaded_at.format("%Y-%m-%d %H:%M:%S UTC"));
        if !file.tags.is_empty() {
            println!("   🏷️  Tags: {}", file.tags.join(", "));
        }
        println!();
    }

    Ok(())
}

fn require_login() -> Result<Session> {
    Session::load()
        .ok_or_else(|| anyhow::anyhow!("You must be logged in. Use: quiz login -u <username> -p <password>"))
}

fn whoami() -> Result<()> {
    if let Some(session) = Session::load() {
        println!("👤 Logged in as: {}", session.username);
        println!("🆔 User ID: {}", session.user_id);
    } else {
        println!("❌ Not logged in");
        println!("💡 Use 'quiz login -u <username> -p <password>' to log in");
    }
    Ok(())
}

async fn tag_file(storage: &UserStorage, session: &Session, file_id: String, tags: String) -> Result<()> {
    let file = storage.get_file_by_id(&file_id, &session.user_id).await?
        .ok_or_else(|| anyhow::anyhow!("File not found with ID: {}", file_id))?;

    let tag_list: Vec<String> = tags.split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if tag_list.is_empty() {
        bail!("No valid tags provided");
    }

    for tag in &tag_list {
        storage.add_tag_to_file(&file_id, &session.user_id, tag.clone()).await?;
    }

    println!("✅ Tags added to file '{}'!", file.filename);
    println!("🏷️  Tags: {}", tag_list.join(", "));
    println!("\n💡 Use 'quiz list' to see all your files and tags");

    Ok(())
}

async fn untag_file(storage: &UserStorage, session: &Session, file_id: String, tag: String) -> Result<()> {
    let file = storage.get_file_by_id(&file_id, &session.user_id).await?
        .ok_or_else(|| anyhow::anyhow!("File not found with ID: {}", file_id))?;

    storage.remove_tag_from_file(&file_id, &session.user_id, &tag).await?;

    println!("✅ Tag '{}' removed from file '{}'!", tag, file.filename);

    Ok(())
}

#[derive(Debug, Serialize)]
struct CreateReminderRequest {
    message: String,
    due_time: String,
    username: Option<String>,
    recurrence: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CreateReminderResponse {
    id: String,
    message: String,
    due_time: String,
}

#[derive(Debug, Deserialize)]
struct Reminder {
    id: String,
    message: String,
    due_time: String,
    sent: bool,
    username: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RemindersResponse {
    reminders: Vec<Reminder>,
}

async fn create_notification(
    session: &Session,
    title: String,
    memo: Option<String>,
    time: String,
    recurrence: Option<String>,
) -> Result<()> {
    let message = if let Some(m) = &memo {
        format!("📚 {}\n📝 {}", title, m)
    } else {
        format!("📚 {}", title)
    };
    
    let request = CreateReminderRequest {
        message,
        due_time: time.clone(),
        username: Some(session.username.clone()),
        recurrence: recurrence.clone(),
    };

    let client = reqwest::Client::new();
    let response = client
        .post("http://localhost:3000/reminders")
        .json(&request)
        .send()
        .await
        .context("Failed to connect to reminder service. Is the server running?")?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        bail!("Failed to create notification: {}", error_text);
    }

    let created: CreateReminderResponse = response.json().await
        .context("Failed to parse response")?;

    println!("✅ Study notification created successfully!");
    println!("📚 Title: {}", title);
    if let Some(m) = memo {
        println!("📝 Memo: {}", m);
    }
    println!("⏰ Scheduled for: {}", time);
    if let Some(rec) = recurrence {
        println!("🔄 Recurrence: {}", rec);
    }
    println!("🆔 Notification ID: {}", created.id);
    println!("\n💡 The reminder service will notify you at the scheduled time!");

    Ok(())
}

async fn list_notifications(session: &Session) -> Result<()> {
    let client = reqwest::Client::new();
    let response = client
        .get("http://localhost:3000/reminders")
        .send()
        .await
        .context("Failed to connect to reminder service. Is the server running?")?;

    if !response.status().is_success() {
        bail!("Failed to fetch notifications");
    }

    let response_data: RemindersResponse = response.json().await
        .context("Failed to parse response")?;

    // Filter by current user
    let user_reminders: Vec<&Reminder> = response_data.reminders
        .iter()
        .filter(|r| r.username.as_ref() == Some(&session.username))
        .collect();

    if user_reminders.is_empty() {
        println!("📭 No study notifications found");
        println!("💡 Use 'quiz notify' to create a study reminder");
        return Ok(());
    }

    println!("📚 Your Study Notifications:\n");
    for (i, reminder) in user_reminders.iter().enumerate() {
        println!("{}. 📌 Notification", i + 1);
        println!("   🆔 ID: {}", reminder.id);
        println!("   📝 Message: {}", reminder.message);
        println!("   ⏰ Scheduled: {}", reminder.due_time);
        println!("   📊 Status: {}", if reminder.sent { "✅ Sent" } else { "⏳ Pending" });
        println!();
    }

    Ok(())
}

async fn filter_files_by_tag(storage: &UserStorage, session: &Session, tag: String) -> Result<()> {
    let all_files = storage.get_user_files(&session.user_id).await?;
    
    let filtered_files: Vec<&UploadedFile> = all_files
        .iter()
        .filter(|f| f.tags.contains(&tag))
        .collect();

    if filtered_files.is_empty() {
        println!("📭 No files found with tag '{}'", tag);
        println!("💡 Use 'quiz tag' to add tags to your files");
        return Ok(());
    }

    println!("📚 Files with tag '{}':\n", tag);
    for (i, file) in filtered_files.iter().enumerate() {
        println!("{}. 📄 {}", i + 1, file.filename);
        println!("   🆔 ID: {}", file.id);
        println!("   📊 Size: {} characters", file.content.len());
        println!("   ⏰ Uploaded: {}", file.uploaded_at.format("%Y-%m-%d %H:%M:%S UTC"));
        if !file.tags.is_empty() {
            println!("   🏷️  Tags: {}", file.tags.join(", "));
        }
        println!();
    }

    Ok(())
}

async fn bulk_tag_files(storage: &UserStorage, session: &Session, file_ids: String, tags: String) -> Result<()> {
    let file_id_list: Vec<String> = file_ids.split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let tag_list: Vec<String> = tags.split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if file_id_list.is_empty() {
        bail!("No valid file IDs provided");
    }

    if tag_list.is_empty() {
        bail!("No valid tags provided");
    }

    let mut success_count = 0;
    let mut failed_files = Vec::new();

    for file_id in &file_id_list {
        // Check if file exists
        if let Some(_file) = storage.get_file_by_id(file_id, &session.user_id).await? {
            for tag in &tag_list {
                storage.add_tag_to_file(file_id, &session.user_id, tag.clone()).await?;
            }
            success_count += 1;
        } else {
            failed_files.push(file_id.clone());
        }
    }

    println!("✅ Bulk tag operation completed!");
    println!("📊 Successfully tagged {} file(s)", success_count);
    println!("🏷️  Tags added: {}", tag_list.join(", "));
    
    if !failed_files.is_empty() {
        println!("⚠️  Failed to find {} file(s): {}", failed_files.len(), failed_files.join(", "));
    }

    Ok(())
}

async fn bulk_untag_files(storage: &UserStorage, session: &Session, file_ids: String, tag: String) -> Result<()> {
    let file_id_list: Vec<String> = file_ids.split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if file_id_list.is_empty() {
        bail!("No valid file IDs provided");
    }

    let mut success_count = 0;
    let mut failed_files = Vec::new();
    let mut tag_not_found = Vec::new();

    for file_id in &file_id_list {
        if let Some(file) = storage.get_file_by_id(file_id, &session.user_id).await? {
            if file.tags.contains(&tag) {
                storage.remove_tag_from_file(file_id, &session.user_id, &tag).await?;
                success_count += 1;
            } else {
                tag_not_found.push(file.filename.clone());
            }
        } else {
            failed_files.push(file_id.clone());
        }
    }

    if success_count > 0 {
        println!("✅ Bulk untag operation completed!");
        println!("📊 Successfully removed tag '{}' from {} file(s)", tag, success_count);
    }
    
    if !tag_not_found.is_empty() {
        println!("ℹ️  Tag '{}' not found on {} file(s): {}", tag, tag_not_found.len(), tag_not_found.join(", "));
    }

    if !failed_files.is_empty() {
        println!("⚠️  Failed to find {} file(s): {}", failed_files.len(), failed_files.join(", "));
    }

    if success_count == 0 && !tag_not_found.is_empty() {
        bail!("Cannot remove tag '{}' because it was not found on any of the selected files", tag);
    }

    Ok(())
}
