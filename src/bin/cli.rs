use chrono::{DateTime, Local, Utc};
use clap::{Parser, Subcommand};
use prettytable::{Cell, Row, Table};
use reqwest;
use serde::{Deserialize, Serialize};

const API_URL: &str = "http://localhost:3000";

#[derive(Parser)]
#[command(name = "reminder")]
#[command(about = "A CLI tool for managing reminders", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Create a new reminder")]
    Create {
        #[arg(short, long, help = "The reminder message")]
        message: String,

        #[arg(short = 't', long, help = "Due time in ISO 8601 format (e.g., 2025-11-04T15:30:00Z)")]
        time: String,

        #[arg(short, long, help = "Optional username")]
        username: Option<String>,
    },

    #[command(about = "View upcoming reminders")]
    View,
}

#[derive(Debug, Serialize)]
struct CreateReminderRequest {
    message: String,
    due_time: String,
    username: Option<String>,
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
    username: Option<String>,
    sent: bool,
    created_at: String,
}

#[derive(Debug, Deserialize)]
struct ReminderListResponse {
    reminders: Vec<Reminder>,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Create {
            message,
            time,
            username,
        } => {
            if let Err(e) = create_reminder(message, time, username).await {
                eprintln!("❌ Error: {}", e);
                std::process::exit(1);
            }
        }
        Commands::View => {
            if let Err(e) = view_reminders().await {
                eprintln!("❌ Error: {}", e);
                std::process::exit(1);
            }
        }
    }
}

async fn create_reminder(
    message: String,
    time: String,
    username: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    let payload = CreateReminderRequest {
        message: message.clone(),
        due_time: time.clone(),
        username: username.clone(),
    };

    let response = client
        .post(format!("{}/reminders", API_URL))
        .json(&payload)
        .send()
        .await?;

    if !response.status().is_success() {
        let error_text = response.text().await?;
        return Err(format!("Failed to create reminder: {}", error_text).into());
    }

    let result: CreateReminderResponse = response.json().await?;

    let due_dt: DateTime<Utc> = result.due_time.parse()?;
    let local_time = due_dt.with_timezone(&Local);

    println!("✅ Reminder created successfully!");
    println!("   Message: {}", result.message);
    println!("   Due: {}", local_time.format("%Y-%m-%d %H:%M:%S %Z"));
    println!("   ID: {}", result.id);
    
    if let Some(user) = username {
        println!("   User: {}", user);
    }

    Ok(())
}

async fn view_reminders() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/reminders", API_URL))
        .send()
        .await?;

    if !response.status().is_success() {
        let error_text = response.text().await?;
        return Err(format!("Failed to fetch reminders: {}", error_text).into());
    }

    let result: ReminderListResponse = response.json().await?;

    if result.reminders.is_empty() {
        println!("📭 No upcoming reminders found.");
        return Ok(());
    }

    println!("\n📋 Upcoming Reminders ({})\n", result.reminders.len());

    let mut table = Table::new();
    table.add_row(Row::new(vec![
        Cell::new("ID"),
        Cell::new("Message"),
        Cell::new("Due Time"),
        Cell::new("User"),
    ]));

    for reminder in result.reminders {
        let due_dt: DateTime<Utc> = reminder.due_time.parse()?;
        let local_time = due_dt.with_timezone(&Local);

        let username = reminder
            .username
            .unwrap_or_else(|| "-".to_string());

        table.add_row(Row::new(vec![
            Cell::new(&reminder.id[..8]),
            Cell::new(&reminder.message),
            Cell::new(&local_time.format("%Y-%m-%d %H:%M:%S").to_string()),
            Cell::new(&username),
        ]));
    }

    table.printstd();
    println!();

    Ok(())
}
