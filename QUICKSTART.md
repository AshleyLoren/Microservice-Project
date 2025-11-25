Quiz CLI - Quick Start Guide
Running the Quiz CLI
The quiz CLI is a command-line tool for managing your study materials and generating quizzes.

# Getting Started
1. Create Your Account

'' ./target/debug/quiz signup -u your_username -p your_password

2. Log In

'' ./target/debug/quiz login -u your_username -p your_password

#3. Upload a Study File

./target/debug/quiz upload -f path/to/your/notes.txt

#4. List Your Files

'' ./target/debug/quiz list

5. Check Who's Logged In

'' ./target/debug/quiz whoami

6. Log Out

'' ./target/debug/quiz logout

Example Workflow
# Step 1: Sign up
'' ./target/debug/quiz signup -u alice -p securepass123
# Step 2: Log in
'' ./target/debug/quiz login -u alice -p securepass123
# Step 3: Create a study file (example)
cat > biology_notes.txt << 'EOF'
Photosynthesis
- Process by which plants convert light energy into chemical energy
- Takes place in chloroplasts
- Requires: sunlight, water, and carbon dioxide
- Produces: glucose and oxygen
EOF

# Step 4: Upload the file
./target/debug/quiz upload -f biology_notes.txt

# Step 5: View your files
'' ./target/debug/quiz list

### Tips

- Keep your password secure (minimum 6 characters)
- You stay logged in between commands until you logout
- Upload text files with clear, structured notes for best quiz results
- Use `./target/debug/quiz --help` to see all available commands

### What's Next?

Once you've uploaded your study materials, future updates will enable AI-powered quiz generation to help you study effectively!
