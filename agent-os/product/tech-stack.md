# Tech Stack

## Backend Framework & Language

### Rust with Axum
- **Primary Language:** Rust for performance, memory safety, and type safety
- **Web Framework:** Axum - ergonomic, modular web framework built on Tokio and Tower
- **Runtime:** Tokio - async runtime with multi-threaded worker pool (4 workers)
- **Middleware:** Tower for HTTP middleware and service composition

## Database & ORM

### PostgreSQL with SQLx
- **Database:** PostgreSQL for relational data storage
- **Query Interface:** SQLx for compile-time checked SQL queries
- **Migrations:** SQLx migrations in `/db/migrations/`
- **Type Safety:** Compile-time verification of SQL queries against schema

## Frontend Architecture

### Server-Side Rendering (Primary)
- **Template Engine:** Maud - type-safe HTML templates in Rust
- **Rendering:** Server-side HTML generation for most pages
- **Benefits:** Fast page loads, minimal JavaScript, excellent SEO

## AI & Agent System

### OpenAI Integration
- **API:** OpenAI API for AI agent capabilities
- **Thread Model:** Custom thread and stitch data model for conversation tracking
- **Memory System:** Persistent agent memory across conversations
- **Agent Builder:** Custom ThreadBuilder for constructing agent contexts

## Discord Bot

### Serenity Framework
- **Bot Framework:** Serenity for Discord bot implementation
- **Commands:** Poise framework for slash commands and prefix commands
- **Integration:** Discord webhook and API integration for bi-directional communication

## Authentication & Security

### OAuth & JWT
- **OAuth Providers:** GitHub OAuth and Google OAuth for admin authentication
- **Session Management:** JWT (JSON Web Tokens) for secure session handling
- **Encryption:** Custom encryption utilities for sensitive data
- **Sentry:** Error tracking and monitoring with Sentry integration

## Platform Integrations

### Third-Party APIs
- **Linear:** Linear API for task and issue management
- **GitHub:** GitHub API for repository and commit tracking
- **Google:** Google APIs for calendar and workspace integration
- **Bluesky:** Bluesky API for social media integration
- **Twitch:** Twitch API integration (present in codebase)

## Development Tools

### Version Control
- **VCS:** jj (Jujutsu) instead of git for version control
- **Workflow:** Bookmark-based workflow with `jj` commands
- **Git Compatibility:** jj provides git interop for GitHub PRs

### Testing
- **Rust Testing:** Cargo test with workspace-wide test suite
- **Frontend Testing:** Vitest with React Testing Library
- **Coverage:** Vitest coverage with v8 provider
- **UI Testing:** Vitest UI for interactive test running

### Code Quality
- **Rust Linting:** Clippy for Rust linting with strict rules
- **Rust Formatting:** rustfmt for consistent code formatting
- **TypeScript:** TypeScript 5.8 for type safety in React code
- **ESLint:** ESLint with TypeScript and React plugins
- **Prettier:** Prettier for frontend code formatting

## Build & Deployment

### Build Process
- **Multi-Stage Build:** Frontend built first, then embedded in Rust binary
- **Scripts:** Custom bash scripts (`build-all.sh`, `dev-build.sh`, `auto-fix-all.sh`)
- **Development Mode:** Debug builds for fast iteration
- **Production Mode:** Release builds with optimizations

### Cron & Background Jobs
- **Job System:** Custom job system for scheduled tasks
- **Cron Jobs:** Cron scheduling for recurring operations (e.g., Discord refresh, standups)
- **Background Processing:** Async task processing with Tokio

## Monitoring & Observability

### Logging & Tracing
- **Tracing:** Custom tracing setup with structured logging
- **Error Tracking:** Sentry for production error monitoring and alerting
- **Performance:** Performance tracking utilities built into the application

## Development Environment

### Package Management
- **Rust:** Cargo for Rust dependency management
- **Frontend:** npm for JavaScript/TypeScript dependencies
- **Workspace:** Cargo workspace with multiple crates (`/server`, `/db`, `/posts`)

### Environment Configuration
- **Environment Variables:** `.envrc` for local development configuration
- **Required Variables:** `DATABASE_URL`, `APP_BASE_URL`, various API keys
- **Secret Management:** Encrypted storage for sensitive credentials

## Content Management

### Markdown Processing
- **Parser:** `markdown` crate with MDast (Markdown Abstract Syntax Tree)
- **Frontmatter:** YAML frontmatter support for post metadata
- **Extensions:** GFM footnotes and custom markdown constructs
- **Storage:** Posts stored as markdown files, embedded with `include_dir!`

## Validation & CLI

### Command Line Interface
- **CLI Parser:** Clap for command-line argument parsing
- **Commands:** Custom command system for various operations
- **Validation:** Built-in validation command for checking system health
