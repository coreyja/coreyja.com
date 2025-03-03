# CLAUDE.md - coreyja.com

## Commands
- Build: `cargo build`
- Run server: `cargo watch -x run --no-gitignore`
- Run with Procfile: `server: cd server && PORT=3002 cargo watch -x run --no-gitignore`
- Run TailwindCSS: `tailwindcss -i server/src/styles/tailwind.css -o target/tailwind.css --watch`
- Test: `cargo test` (all tests), `cargo test -p <crate_name>` (package), `cargo test <test_name>` (specific test)
- Lint: `cargo clippy --all-targets --no-deps`
- Docs: `cargo doc --workspace --no-deps`

## Code Style
- Rust edition 2021
- Clippy pedantic level set to "deny"
- Unsafe code is forbidden
- Imports should follow Rust standard organization
- Error handling via thiserror crate with structured errors
- Naming conventions: snake_case for variables/functions, CamelCase for types/structs
- Workspace structure: server, cja, db, posts, tracing-common crates
- Default run target is "server" binary
- Use async/await with Tokio for async operations
- Use Axum for HTTP handling
- Type-safe database access with SQLx