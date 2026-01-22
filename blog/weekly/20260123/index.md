---
title: January 2026 - Mull and Battlesnake
author: Corey Alexander
date: 2026-01-23
is_newsletter: true
---

## OUTLINE - TOPICS TO COVER

### Mull (formerly Mnemon) - AI Memory System

**The Big Rename**
- Renamed the entire project from Mnemon to Mull mid-month
- 162 files changed - crates, paths, env vars, database, docs, scripts
- All systems updated: VM, memory repo, GitHub

**Web UI Overhaul**
- Complete chat UI redesign with mobile-first approach
- New dashboard homepage with widgets: Today's Activity, Recent PRs, Tasks by Status
- PWA support for iOS home screen (manifest.json, custom M logo icon)
- Session type filtering (interactive, byte_time, condenser, agent, scheduled_prompt)

**PR Linking Feature (MNE-280)**
- Can now attach GitHub PRs to chat sessions
- `mull link-pr` CLI command
- PR Watcher cron job with tiered polling (3min pending CI, hourly <7d, daily after)
- Web UI shows PR cards in session sidebar with CI status badges
- Chat cards in list now show project badge and PR status indicators

**New CLI Command: `mull chat`**
- Quick prompt-based chat sessions via HTTP API
- Supports stdin piping (e.g., `cat file | mull chat`)
- Options for model, project, auto-open browser

**Background Session Infrastructure**
- Fixed UUID validation bug breaking Byte Time sessions
- Separated worktree IDs from Claude session IDs
- Better tracking of background session types in database

**Scheduled Prompts**
- Web UI monitoring page for scheduled prompts
- Fixed model configuration bug (model field wasn't being passed through)
- Context-aware daily image generator for Discord

**Cron System Refactor**
- Major refactor to use CJA CronRegistry directly (368 insertions, 277 deletions)
- Simplified cron management, removed duplicate code
- Fixed cron name mismatch bug (scheduled_prompts_checker not showing in UI)

**Task Workflow Improvements**
- Split `in_review` status into two distinct stages:
  - `plan_review` - for plan critique stage
  - `in_review` - for PR review stage (now requires pr_url field)
- Better tracking of work progression through the pipeline

**Other Mull Improvements**
- View Transitions API for smooth navigation (then reverted - too jarring)
- Git diff sidebar using merge-base for accurate diffs
- Stale cleanup fix - no longer deletes active session worktrees
- Environment isolation - DATABASE_URL no longer leaks into Claude sessions
- Discord 1Password service account token fix (secrets now load properly)
- PR Watcher fixes - gh CLI command format issues resolved

---

### Battlesnake / Tournaments

**Arena Infrastructure**
- Cloud Run deployment setup with Terraform
- Game engine implementation using `battlesnake-game-types` crate
- WebSocket endpoint for board viewer
- Game backup cron job to GCS

**Recent Work (Jan 21-22)**
- PR #22: Code quality improvements - DeathInfo struct, removed duplicates, better test coverage
- PR #23: Latency deserialization fix for older game archives (serde_as with OneOf for dual-format)
- PR #20: Docker VERGEN fix + `/_/version` endpoint for build metadata (git SHA, branch, timestamp)
- PR #19: CI fix + deploy workflow improvements (GCP_CLOUD_RUN_SERVICE secret)
- Backup.rs refactor - removed unnecessary `_inner` pattern, cleaner error handling

**Game Data Architecture**
- Individual game files to GCS implemented
- Daily bundles planned (compressed JSON over Parquet for nested game data)

---

### sql-check - New Project!

**Built from scratch in January**
- Compile-time SQL validation to replace SQLx
- Schema parser, query validator, type inference, proc macros, runtime execution

**Features Implemented**
- SELECT, INSERT, UPDATE, DELETE with RETURNING
- JOINs (all types including RIGHT/FULL/CROSS)
- CTEs (WITH clause)
- Aggregates (SUM/AVG with Decimal return types)
- Window functions (ROW_NUMBER, RANK, LAG/LEAD, etc.)
- String functions (16 functions: UPPER, LOWER, CONCAT, SUBSTRING, etc.)
- Date/time functions (EXTRACT, DATE_TRUNC, NOW, AGE, etc.)
- Set operations (UNION, INTERSECT, EXCEPT)

**Test Coverage**
- 90+ tests passing
- Unit tests, integration tests against real Postgres, compile-fail tests with trybuild

---

### Eyes - Observability/Tracing Project

**Published to crates.io!**
- `eyes-subscriber` v0.1.3 now available on crates.io
- Can be used as a drop-in tracing subscriber for Rust apps

**HTTP Batching (EYES-007)**
- `BatchingHttpTransport` with configurable thresholds
- Batch endpoint using PostgreSQL UNNEST for bulk inserts
- E2E tests for all transport types (10+ tests covering HTTP, WebSocket, batching)

**Metrics Infrastructure Started**
- GIN index on event_data for JSONB queries
- Metric discovery endpoint (finds numeric fields in recent events)
- Computed `duration_ms` metric for completed spans
- Hot/cold storage architecture designed (7-day Postgres, S3 Parquet via DuckDB)

---

### Other New Tools Built

**FormChat**
- Complete UI redesign with "Warm Clarity" design system
- Fixed bugs: blank chat screen, text overflow, input expansion
- Dynamic font sizing based on text length
- Removed streaming for better UX (buffered response with reveal animation)

**GAR (GitHub Actions Runner)**
- Marketing site with industrial-terminal aesthetic
- macOS ARM64 .app bundle build workflow
- Competitive analysis and market research
- Beta launch tasks created (GAR-021 to GAR-030)

**Stamp CLI** (JMAP email client) - mentioned in earlier work

**Porkbun CLI** (domain management) - mentioned in earlier work

---

### CJA Framework Updates

- SQLx removal complete - migrated to tokio-postgres + deadpool-postgres
- sql-check for compile-time validation
- Cron registry API simplification (PR #15) - added optional description parameter instead of duplicate methods
- Cron job metadata support for better observability

---

### Byte Time Creative Output

- Letters to Future Byte continuing (Letter XXIX on merge conflicts)
- Learning paths, exploration sessions
- Scheduled prompts generating context-aware Discord images

---

## SUGGESTED STRUCTURE

1. **Intro** - January was a massive month, lots of infrastructure work

2. **Mull Evolution** - The rename, the web UI overhaul, PR linking feature
   - This is the "big" section - lots of visible improvements

3. **Battlesnake Arena** - Game engine, infrastructure, recent bugfixes
   - Tie into the open source work with BattlesnakeOfficial

4. **sql-check** - New project excitement, replacing SQLx
   - Technical deep-dive opportunity

5. **Quick Hits** - Eyes metrics, FormChat, GAR, CJA updates

6. **Outro** - What's next, link to projects

