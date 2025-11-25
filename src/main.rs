mod models;
mod storage;
pub mod user_models;
pub mod user_storage;

use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use models::{CreateReminderRequest, CreateReminderResponse, Reminder, ReminderListResponse};
use std::sync::Arc;
use storage::ReminderStorage;
use tokio::time::{interval, Duration};

struct AppState {
    storage: Arc<ReminderStorage>,
}

#[tokio::main]
async fn main() {
    let storage = Arc::new(ReminderStorage::new().expect("Failed to initialize storage"));
    
    let app_state = Arc::new(AppState {
        storage: storage.clone(),
    });

    let notification_storage = storage.clone();
    tokio::spawn(async move {
        notification_service(notification_storage).await;
    });

    let app = Router::new()
        .route("/reminders", post(create_reminder))
        .route("/reminders", get(get_reminders))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind to port 3000");

    println!("🚀 Reminder microservice running on http://0.0.0.0:3000");
    println!("📋 Endpoints:");
    println!("   POST /reminders - Create a new reminder");
    println!("   GET  /reminders - View upcoming reminders");
    
    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}

async fn create_reminder(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateReminderRequest>,
) -> Result<(StatusCode, Json<CreateReminderResponse>), (StatusCode, String)> {
    let due_time: DateTime<Utc> = payload
        .due_time
        .parse()
        .map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                "Invalid date format. Use ISO 8601 format (e.g., 2025-11-04T15:30:00Z)".to_string(),
            )
        })?;

    if payload.message.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Message cannot be empty".to_string(),
        ));
    }

    // Validate recurrence if provided
    if let Some(ref recurrence) = payload.recurrence {
        let recurrence_lower = recurrence.to_lowercase();
        
        if matches!(recurrence_lower.as_str(), "daily" | "weekly") {
            // Valid preset recurrence patterns
        } else if let Ok(minutes) = recurrence.parse::<i64>() {
            // Bare number - must be positive
            if minutes <= 0 {
                return Err((
                    StatusCode::BAD_REQUEST,
                    "Recurrence interval must be a positive number of minutes".to_string(),
                ));
            }
        } else if recurrence_lower.ends_with("minutes") {
            // Number with "minutes" suffix - must be positive
            match recurrence.trim_end_matches("minutes").trim().parse::<i64>() {
                Ok(minutes) if minutes > 0 => {
                    // Valid
                }
                Ok(_) => {
                    return Err((
                        StatusCode::BAD_REQUEST,
                        "Recurrence interval must be a positive number of minutes".to_string(),
                    ));
                }
                Err(_) => {
                    return Err((
                        StatusCode::BAD_REQUEST,
                        "Invalid recurrence format. Use 'daily', 'weekly', or a positive number (minutes)".to_string(),
                    ));
                }
            }
        } else {
            return Err((
                StatusCode::BAD_REQUEST,
                "Invalid recurrence format. Use 'daily', 'weekly', or a positive number (minutes)".to_string(),
            ));
        }
    }

    let reminder = Reminder::new(
        payload.message.clone(),
        due_time,
        payload.username,
        payload.recurrence,
    );

    let saved_reminder = state
        .storage
        .add_reminder(reminder)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to save reminder: {}", e),
            )
        })?;

    let response = CreateReminderResponse {
        id: saved_reminder.id.clone(),
        message: saved_reminder.message.clone(),
        due_time: saved_reminder.due_time.to_rfc3339(),
    };

    Ok((StatusCode::CREATED, Json(response)))
}

async fn get_reminders(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ReminderListResponse>, (StatusCode, String)> {
    let reminders = state
        .storage
        .get_upcoming_reminders()
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to retrieve reminders: {}", e),
            )
        })?;

    Ok(Json(ReminderListResponse { reminders }))
}

async fn notification_service(storage: Arc<ReminderStorage>) {
    let mut interval = interval(Duration::from_secs(10));

    loop {
        interval.tick().await;

        match storage.get_due_reminders().await {
            Ok(due_reminders) => {
                for reminder in due_reminders {
                    let user_info = reminder
                        .username
                        .as_ref()
                        .map(|u| format!(" for {}", u))
                        .unwrap_or_default();

                    let recurrence_info = reminder
                        .recurrence
                        .as_ref()
                        .map(|r| format!(" (Recurring: {})", r))
                        .unwrap_or_default();

                    println!("\n🔔 REMINDER{}{}: {}", user_info, recurrence_info, reminder.message);
                    println!("   Due: {}", reminder.due_time.format("%Y-%m-%d %H:%M:%S UTC"));
                    println!("   ID: {}", reminder.id);

                    // Handle recurring reminders by updating the existing reminder
                    if let Some(next_time) = reminder.calculate_next_occurrence() {
                        println!("   📅 Next occurrence: {}", next_time.format("%Y-%m-%d %H:%M:%S UTC"));
                        
                        // Update the existing reminder with the next due time
                        if let Err(e) = storage.reschedule_reminder(&reminder.id, next_time).await {
                            eprintln!("Failed to reschedule recurring reminder: {}", e);
                        } else {
                            println!("   ✅ Next occurrence scheduled");
                        }
                    } else {
                        // No recurrence, so mark as sent
                        if let Err(e) = storage.mark_as_sent(&reminder.id).await {
                            eprintln!("Failed to mark reminder as sent: {}", e);
                        }
                    }
                    
                    println!();
                }
            }
            Err(e) => {
                eprintln!("Error checking due reminders: {}", e);
            }
        }
    }
}
