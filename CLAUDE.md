# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview

This is coreyja.com - a personal website built with:

- **Backend**: Rust with Axum web framework
- **Frontend**: Server-side rendered HTML (Maud templates)
- **Database**: PostgreSQL with SQLx
- **Version Control**: Uses `jj` instead of `git`

## Build Commands

### Quick Build

```bash
# Full build (Rust release)
./scripts/build-all.sh

# Development build (Rust debug)
./scripts/dev-build.sh
```

**IMPORTANT**: Do not run the server binary or `./scripts/start.sh`. The server will be tested locally by the user - the agent doesn't need to worry about running or testing the server.

## Development Commands

### Auto-fix and Format

```bash
# Fix all linting/formatting issues
# This also generates the sqlx files we need to checkin to the repo
./scripts/auto-fix-all.sh
```

### Rust Development

```bash
# Run tests
cargo test
cargo test --workspace

# Run specific test
cargo test test_name

# Linting and formatting
cargo clippy --all-targets --all-features --workspace --tests
cargo fmt

# Database operations. From the `db` directory.
cargo sqlx migrate run
cargo sqlx prepare --all --workspace -- --all-targets # This is also part of the auto-fix-all.sh script
```

## Architecture

### Workspace Structure

- **`/server`** - Main web server (Axum)
- **`/db`** - Database models and migrations
- **`/posts`** - Blog post handling

### Key Technologies

- **Web Framework**: Axum with Tower middleware
- **Database**: PostgreSQL with SQLx (compile-time checked queries)
- **Templating**: Maud for server-side HTML generation
- **Admin Authentication**: GitHub/Google OAuth, JWT sessions
- **Discord Bot**: Serenity framework with Poise commands
- **AI Agent**: Thread and stitch data model for agent interactions

### Database

- Migrations in `/db/migrations/`
- Uses SQLx with compile-time query verification
- Environment variable: `DATABASE_URL=postgres://localhost:5432/byte`

### Frontend Architecture

The site uses server-side rendered HTML using Maud templates for all pages, including the admin interface. This provides a simple, fast, and maintainable frontend with zero JavaScript dependencies.

## Version Control with jj

This project uses `jj` instead of `git`.

### Key Rule: ALWAYS run `jj status` first

Before any jj operation, run `jj status` to understand your current state.

### Essential Commands:

```bash
jj status                       # Shows current state - ALWAYS run this first
jj diff                         # Review changes in current commit
jj describe -m "commit message" # Describe current commit
jj new                          # Create new empty commit
jj squash                       # Move current changes into previous commit
jj log                          # Show commit history
```

### Basic Workflow:

1. **Check your state:** Run `jj status`

   - "Working copy is clean" = you're in an empty commit, ready to work
   - Shows file changes = you have uncommitted work
   - Shows description = current commit is already described

2. **Make changes:** Edit files as needed

3. **Describe your work:** Run `jj describe -m "Clear commit message"`

4. **Start new work:** Run `jj new` (creates empty commit for next task)

### When to use `jj squash`:

Started a new commit but realized the changes belong with the previous one? Just run `jj squash` to move them back. This is the preferred approach - don't worry about getting it perfect the first time.

### Example:

```bash
# Check state before starting
jj status

# Work on feature A
# ... edit files ...
jj describe -m "Add user authentication"

# Start feature B
jj new
# ... edit files ...
# Oops, these changes are still part of authentication
jj squash  # Moves changes back to "Add user authentication"

# Now really start feature B
jj new
# ... edit different files ...
jj describe -m "Add user profile page"
```

### Working with Bookmarks & PRs:

```bash
jj bookmark list                # List all bookmarks
jj git push --change @           # Push current commit, auto-creates bookmark
jj bookmark set <name> -r @    # Set bookmark manually if needed
```

### PR Workflow:

All changes must go through PRs - no direct pushes to main.

1. Make your changes and describe them
2. Run `jj git push --change @` to push and create a bookmark (note the bookmark name it creates)
3. Create PR with GitHub CLI: `gh pr create --head <bookmark-name>`
   - You must specify the bookmark name with `--head` since gh uses git and won't know the current jj bookmark
   - Get the bookmark name from the output of `jj git push` or run `jj log -r @ --no-graph` to see it

## Environment Setup

Required environment variables (see `.envrc` for full list):

- `DATABASE_URL` - PostgreSQL connection (database will be pre-configured, agent doesn't need to set this up)
- `APP_BASE_URL` - Application base URL
- Various API keys for integrations (GitHub, Google, OpenAI, etc.)

Note: Database setup and migrations are handled by the user. The agent can run migrations with `cargo sqlx migrate run` if needed, but shouldn't need to create or configure the database.

## Testing Approach

Follow TDD when writing code:

1. Write minimal test first
2. Implement code to pass test
3. Write next test and implement
4. Continue iteratively
