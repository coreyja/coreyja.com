# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview

This is coreyja.com - a personal website built with:

- **Backend**: Rust with Axum web framework
- **Frontend**: Server-side rendered HTML (Maud templates) for most pages
- **Admin UI**: React + TypeScript for the thread visualization page only
- **Database**: PostgreSQL with SQLx
- **Version Control**: Uses `jj` instead of `git`

## Build Commands

### Quick Build

```bash
# Full build (frontend + Rust release)
./scripts/build-all.sh

# Development build (frontend + Rust debug)
./scripts/dev-build.sh

# Frontend only
./scripts/build-frontend.sh
```

**IMPORTANT**: Do not run the server binary or `./scripts/start.sh` as it starts an interactive web server that will block the terminal.

## Development Commands

### Auto-fix and Format

```bash
# Fix all linting/formatting issues
# This also generates the sqlx files we need to checkin to the repo
./scripts/auto-fix-all.sh
```

### Frontend Development

```bash
cd thread-frontend
npm run dev          # Start dev server
npm run test         # Run tests
npm run test:ui      # Run tests with UI
npm run lint         # Check linting
npm run lint:fix     # Fix linting issues
npm run format       # Format code
npm run typecheck    # Type checking
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
- **`/thread-frontend`** - React admin UI for thread visualization (built and embedded into server binary)

### Key Technologies

- **Web Framework**: Axum with Tower middleware
- **Database**: PostgreSQL with SQLx (compile-time checked queries)
- **Frontend**: Maud templates for server-side HTML generation
- **Admin UI**: React with TanStack Router/Query for thread visualization only
- **Templating**: Maud for server-side HTML generation
- **Admin Authentication**: GitHub/Google OAuth, JWT sessions
- **Discord Bot**: Serenity framework with Poise commands
- **AI Agent**: Thread and stitch data model for agent interactions

### Database

- Migrations in `/db/migrations/`
- Uses SQLx with compile-time query verification
- Environment variable: `DATABASE_URL=postgres://localhost:5432/byte`

### Frontend Architecture

The site uses two approaches for the frontend:

1. **Main Site**: Server-side rendered HTML using Maud templates
2. **Admin Thread Viewer**: React SPA for interactive thread/stitch visualization

The React admin UI is built and embedded into the Rust binary at compile time:

1. React app built with Vite (located in `/thread-frontend`)
2. Assets included in Rust binary using `include_dir!`
3. Served by Axum for the admin interface

## Version Control with jj

This project uses `jj` instead of `git`:

```bash
jj describe -m "commit message"  # Describe current change
jj new                          # Create new change
jj status                       # Show status
jj log                         # Show history
```

Don't run a `jj describe` without first running `jj status` to understand if the current changes already have a description.

You should prefer to start new work in a new commit. Use `jj status` to understand if you are currently in an empty commit. If so proceed. If not, you should make a new commit. First we need to check if the current commit has a description. If it does, you are free to make a new commit with `jj new`. If it does not you should first run `jj diff` to understand the current commit. Then describe it, before making a new commit.

## Environment Setup

Required environment variables (see `.envrc` for full list):

- `DATABASE_URL` - PostgreSQL connection
- `APP_BASE_URL` - Application base URL
- Various API keys for integrations (GitHub, Google, OpenAI, etc.)

## Testing Approach

Follow TDD when writing code:

1. Write minimal test first
2. Implement code to pass test
3. Write next test and implement
4. Continue iteratively
