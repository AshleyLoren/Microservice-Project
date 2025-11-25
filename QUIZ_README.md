# Quiz CLI - Study Quiz Generator

A Rust CLI application for managing user accounts and uploading text files to generate study quizzes.

## Implemented User Stories
Authentication & Account Management

## User Account Creation - Create a new account with username and password
## User Login - Secure authentication with bcrypt password hashing
## File Upload (Microservice 4) 3. File Uploading - Upload text files with type validation (only .txt), empty file detection, and reliability checks

## File Preview & Metadata Confirmation - View file contents and confirm before uploading
## File Tagging (Microservice 5) 5. File Tagging - Organize uploaded files with custom tags and filter by tag

## Bulk Tag Edits - Apply or remove tags from multiple files simultaneously
## Study Notifications (Microservice 6) 7. Reminders with Title, Memo, and Time - Create study reminders with optional memo

## Recurring Study Reminders - Set up daily, weekly, or custom interval reminders
## Installation
## Build the quiz CLI:

'' cargo build --release --bin quiz

Usage
1. Create an Account
'' ./target/debug/quiz signup -u <username> -p <password>

# Example:

'' ./target/debug/quiz signup -u john -p mypassword123

2. Log In
'' ./target/debug/quiz login -u <username> -p <password>

# Example:

''  ./target/debug/quiz login -u john -p mypassword123

3. Upload a Text File
Upload a text file with automatic validation, preview, and confirmation:

''  ./target/debug/quiz upload -f <filepath>

# Example:

./target/debug/quiz upload -f study_notes.txt

File Upload Features:

- Only .txt files are accepted (other file types are rejected)
- Empty files are detected and rejected with clear error messages
- File preview shows first 200 characters before upload
- Displays filename and size in characters and KB
- Requires confirmation (yes/no) before finalizing upload
- Reliable upload handling - no partial or corrupted files

## Additional Commands
# Check who is logged in:

'' ./target/debug/quiz whoami

# List your uploaded files:

'' ./target/debug/quiz list

# Log out:

'' ./target/debug/quiz logout

4. Tag Files for Organization
Add tags to a single file:

./target/debug/quiz tag -f <file-id> -t "tag1,tag2,tag3"

# Example:

./target/debug/quiz tag -f abc123-def456 -t "biology,cells,important"

Remove a tag from a single file:

./target/debug/quiz untag -f <file-id> -t <tag-name>

Bulk tag multiple files at once:

./target/debug/quiz bulk-tag -f "file-id-1,file-id-2,file-id-3" -t "tag1,tag2"

# Example:

./target/debug/quiz bulk-tag -f "abc123,def456,ghi789" -t "exam,important"

Bulk remove tags from multiple files:

./target/debug/quiz bulk-untag -f "file-id-1,file-id-2" -t "tag-name"

Filter files by specific tag:

./target/debug/quiz filter-by-tag -t <tag-name>

# Example:

./target/debug/quiz filter-by-tag -t biology

5. Create Study Notifications
Create a one-time reminder (memo is optional):

./target/debug/quiz notify -n "Title" -t "2025-11-20T15:00:00Z"

Create a reminder with memo:

./target/debug/quiz notify -n "Title" -m "Memo text" -t "2025-11-20T15:00:00Z"

Create a recurring reminder:

./target/debug/quiz notify -n "Title" -m "Memo" -t "2025-11-20T15:00:00Z" -r "daily"