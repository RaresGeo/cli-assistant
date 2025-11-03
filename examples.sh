#!/bin/bash

# Examples of using the assistant CLI tool

# =============================================================================
# BASIC USAGE
# =============================================================================

# Simple inline query
assistant what is 2+2

# Using a specific model
assistant -m codellama:7b explain this Python code: "lambda x: x**2"

# Adjusting creativity/temperature
assistant -t 0.1 what is the capital of France  # Very deterministic
assistant -t 0.9 write a haiku about programming  # More creative

# =============================================================================
# SHELL INTEGRATION
# =============================================================================

# Get help with the current directory
assistant explain what files are in $(pwd)

# Analyze a command's output
assistant explain this error: "$(cat error.log 2>&1)"

# Get git information
assistant summarize: "$(git log --oneline -10)"

# Find and explain large files
assistant explain these files: "$(du -sh * | sort -h | tail -5)"

# =============================================================================
# CODE ANALYSIS
# =============================================================================

# Analyze a specific file
assistant review this code: "$(cat main.py)"

# Debug an error
assistant -m codellama debug: "$(python script.py 2>&1)"

# Generate tests
echo "def add(a, b): return a + b" | assistant write unit tests for this function

# =============================================================================
# SYSTEM ADMINISTRATION
# =============================================================================

# Explain system status
assistant analyze: "$(df -h)" and suggest cleanup

# Process analysis
assistant what are these processes: "$(ps aux | head -10)"

# Network debugging
assistant explain: "$(netstat -tuln | grep LISTEN)"

# =============================================================================
# PIPING AND REDIRECTION
# =============================================================================

# Save response to file
assistant write a README for a Python project > README.md

# Pipe through less for long responses
assistant explain Docker best practices | less

# Chain with other commands
assistant list 10 Linux commands | while read cmd; do echo "Learning: $cmd"; done

# =============================================================================
# INTERACTIVE MODE EXAMPLES
# =============================================================================

# For longer prompts, just run assistant without arguments:
cat << 'EOF' | assistant
Please analyze this SQL query and suggest optimizations:

SELECT u.name, COUNT(o.id) as order_count, SUM(o.total) as total_spent
FROM users u
LEFT JOIN orders o ON u.id = o.user_id
WHERE o.created_at > '2024-01-01'
GROUP BY u.id, u.name
HAVING COUNT(o.id) > 5
ORDER BY total_spent DESC;
EOF

# =============================================================================
# ADVANCED PATTERNS
# =============================================================================

# Create a function for quick queries
ai() {
    assistant "$@"
}

# Create specialized functions
explain() {
    assistant -m codellama explain this code: "$@"
}

debug_error() {
    assistant -t 0.3 debug this error and suggest fixes: "$@"
}

# Git commit message generator
git_commit_message() {
    git diff --staged | assistant -t 0.3 write a conventional commit message for these changes
}

# Quick documentation lookup
docs() {
    assistant -t 0.1 "explain the $1 command in Linux with examples"
}

# =============================================================================
# WORKFLOW AUTOMATION
# =============================================================================

# Morning briefing
morning_brief() {
    echo "Good morning! Here's your brief:"
    assistant list 5 things to check when starting work as a developer
}

# Code review helper
review_pr() {
    git diff main..HEAD | assistant -m codellama review this pull request and suggest improvements
}

# Learning assistant
learn() {
    topic="$1"
    assistant create a learning plan for "$topic" with resources and milestones
}

# =============================================================================
# CONFIGURATION EXAMPLES
# =============================================================================

# Quick model switching
alias ai-fast='assistant -m llama3.2'
alias ai-code='assistant -m codellama:13b'
alias ai-creative='assistant -m mixtral:8x7b -t 0.9'

# Project-specific configs
# In project directory, create .assistant file:
# model=codellama:13b
# temperature=0.3
# Then: assistant $(cat .assistant) your prompt here
