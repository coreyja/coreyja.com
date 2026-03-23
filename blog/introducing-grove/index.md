---
title: Introducing Grove
author: Corey Alexander
date: 2026-03-23
tags:
  - rust
  - grove
  - developer-tools
---

I've been working with git worktrees heavily for the past year or so. They've become essential to how I work — especially as I've leaned more into AI coding agents, where each agent gets its own isolated worktree to work in without stepping on anyone else's toes.

But managing worktrees across multiple projects has always been... friction-y. Creating a worktree means remembering the right path conventions, then setting up environment variables for it, making sure the database exists, running setup scripts. Every time. For every project.

So I built Grove.

## What is Grove?

Grove is a lightweight CLI tool that manages a "grove" of git (and jj!) repositories. It does three things:

1. **Project registry** — Track your repos under short names
2. **Layered environment variables** — Per-project and per-worktree env vars that auto-inject into your shell
3. **Centralized worktree management** — Create, list, and clean up worktrees with automatic database provisioning and setup hooks

That's it. It's intentionally small. It doesn't try to be a build system, a package manager, or a CI tool. It just removes the friction from the multi-repo, multi-worktree workflow.

## The Problem

Here's what my workflow used to look like when starting work on a feature:

```bash
cd ~/code/myproject
git worktree add ~/active/myproject-cool-feature -b cool-feature
cd ~/active/myproject-cool-feature
export DATABASE_URL=postgres:///myproject_cool_feature
createdb myproject_cool_feature
cargo sqlx database setup
yarn install
```

Six commands, every time. And that's the happy path where I remember everything. Half the time I'd forget to set `DATABASE_URL` and wonder why my tests were hitting the wrong database, or I'd forget to run the setup command and waste ten minutes debugging a missing migration.

Multiply that by the number of projects I touch in a week, and it adds up.

## What it Looks Like Now

```bash
grove start myproject cool-feature
```

That's it. One command. Grove creates the worktree, provisions a fresh database from a template, runs the post-create hooks (migrations, dependency installs, whatever you've configured), sets up the environment variables, and opens your editor.

## How the Pieces Fit Together

### Project Registry

You register projects once:

```bash
grove add myproject ~/code/myproject
grove add frontend ~/code/frontend
grove add backend ~/code/backend
```

Then `grove list` shows everything at a glance.

### Layered Env Vars

This is the part I'm most proud of. Environment variables resolve in three layers, highest priority first:

1. **Worktree-level** — Overrides for a specific worktree
2. **Project-level** — Defaults for the project
3. **Repo-level** — Committed to the repo in `.grove/config.toml`

That third layer is great for teams. You commit sensible defaults to the repo, and each developer can override what they need without touching shared config.

```bash
# Set a project-wide default
grove env set myproject DATABASE_URL postgres:///myproject_dev

# Override for a specific worktree
grove env set myproject DATABASE_URL postgres:///myproject_feature --worktree cool-feature

# See where each value comes from
grove env list myproject
```

`grove env list` shows you which layer each variable is coming from, so you can debug "why is this set to that?" without hunting through dotfiles.

### mise Integration

Grove ships with a [mise](https://mise.jdx.dev/) plugin. After a one-time `grove init-mise`, your grove env vars automatically inject into your shell whenever you `cd` into a managed project. No sourcing, no shell hooks to maintain — it just works.

### Repo-Scoped Config

For team projects, you can commit a `.grove/config.toml` to your repo:

```toml
name = "myproject"

[database]
url_template = "postgres:///{{db_name}}"
setup_command = "cargo sqlx database setup"

[hooks]
post_create = ["yarn install", "cargo build"]

[env]
RUST_LOG = "debug"
NODE_ENV = "development"
```

When anyone on the team runs `grove start`, they get the same setup — databases provisioned, hooks run, env vars set. The team gets consistency, and individuals can still override anything they need.

## Why Worktrees?

If you haven't used git worktrees before, they let you check out multiple branches of the same repo simultaneously, each in their own directory. Unlike cloning the repo multiple times, worktrees share the same `.git` database, so they're cheap and stay in sync.

For me, worktrees are essential for a few reasons:

- **AI agents**: I run multiple coding agents in parallel, each in their own worktree. They can't conflict with each other because they're working in completely separate directories with separate databases.
- **Context switching**: I can leave a feature branch exactly where it is — editor state, running processes, everything — and start a new worktree for an urgent fix.
- **Code review**: I can check out a PR in a worktree to test it without touching my current work.

Grove also supports **jj workspaces** if you're a [Jujutsu](https://martinvonz.github.io/jj/) user. It auto-detects which VCS your project uses and does the right thing.

## The Stack

Grove is written in Rust, because that's what I reach for these days. It's a single binary with no runtime dependencies. Configuration is all TOML files — no databases, no daemons, no network calls.

The project is [open source on GitHub](https://github.com/coreyja-studio/grove), and there are pre-built binaries for macOS (Intel and Apple Silicon) and Linux. Or `cargo install` if you prefer building from source.

## What's Next

Grove is still young but it's been solid for my daily workflow. A few things I'm working on:

- **`grove init`** — An interactive onboarding flow for new users
- **Shell completions** — Tab completion via clap for all commands
- **Better first-run experience** — Right now if you run grove without any config it's not the friendliest error message. Fixing that.

If you work across multiple repos, or you've been curious about worktrees but put off by the setup friction, give Grove a try. I'd love to hear how it works for your workflow.

Check out the docs at [grove.coreyja.com](https://grove.coreyja.com) or the [GitHub repo](https://github.com/coreyja-studio/grove).
