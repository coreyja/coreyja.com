---
title: Your Git Log Is a Development Fingerprint
author: Corey Alexander
date: 2026-02-09
tags:
  - ai
  - git
  - rust
  - data
---

<!-- TODO: Opening hook. You've written about embracing AI agent coding in the
Battlesnake post. This is the data-driven follow-up: what does the git log
actually look like when an AI agent is writing half your code?

Suggested angle: "I ran a tool across 12 of my repositories to see how
AI-assisted development shows up in commit history. The patterns were more
distinct than I expected." -->

## The Setup

<!-- TODO: Brief context on your development workflow.
- Byte (AI agent) runs autonomously: plans, implements, opens PRs, self-corrects via CI
- You work alongside, reviewing PRs and doing deeper architectural work
- git-mine is the tool you built to analyze these patterns
- Link to Battlesnake 2026 post for context on AI agent coding philosophy -->

I analyzed 12 repositories spanning January 2025 through February 2026, covering roughly 800 commits across projects ranging from fully autonomous AI development to human-only coding. The tool is [git-mine](https://github.com/coreyja-studio/git-mine), a Rust CLI that extracts patterns from git history.

## Add-Fix Rate: An Iteration Speed Signal

The "add-fix rate" measures how often a feature commit is followed by a fix commit within 60 minutes by the same author. It's not a quality metric — it's an iteration speed metric. A high rate means problems are found and fixed *fast*, not that there are more problems.

| Repo | Commits | AI % | Add-Fix | Rate | Style |
|------|---------|------|---------|------|-------|
| sql-check | 16 | 100% | 3 | **18.7%** | Fully autonomous |
| skills | 18 | 77% | 2 | **11.1%** | Mostly autonomous |
| mull | 399 | 51% | 35 | **8.7%** | Autonomous pipeline |
| GAR | 24 | 79% | 2 | **8.3%** | Mostly autonomous |
| tournaments | 73 | 49% | 3 | **4.1%** | AI-pair |
| coreyja.com | 103 | 0% | 3 | **2.9%** | Human-primary |
| cja | 32 | 28% | 0 | 0% | AI-pair |
| eyes | 33 | 36% | 0 | 0% | AI-pair |
| grove | 7 | 100% | 0 | 0% | Autonomous |
| stamp | 12 | 100% | 0 | 0% | Autonomous |
| formchat | 13 | 100% | 0 | 0% | Autonomous |
| battlesnake-rs | 5 | 100% | 0 | 0% | Autonomous |

<!-- TODO: Your interpretation. The correlation isn't perfect (grove, stamp,
formchat, battlesnake-rs are 100% AI but have 0% add-fix — though they also
have very few commits, which makes the rate unreliable). The signal shows up
in repos with 50+ commits where the pattern has room to emerge.

The key insight: autonomous AI development produces a distinctive ship-CI-fix
loop that shows up as add-fix pairs. This isn't bugs — it's the system
self-correcting through CI feedback faster than a human review cycle would. -->

## The Mull Deep Dive

Mull is the most interesting dataset: 399 commits split roughly 52/48 between Byte (AI agent) and me. Same codebase, same time period, two very different development rhythms.

### Time Between Consecutive Commits

| Gap | Byte | Me |
|-----|------|-----|
| < 5 min | 34 (17%) | 43 (23%) |
| 5-15 min | 18 (9%) | 35 (18%) |
| 15-30 min | 21 (10%) | 28 (15%) |
| 30-60 min | 30 (15%) | 15 (8%) |
| 1-3 hours | 42 (20%) | 32 (17%) |
| > 3 hours | 61 (30%) | 37 (19%) |

<!-- TODO: What this means to you. The pattern:
- Byte is bimodal: 17% rapid-fire bursts + 30% long gaps = 47% at the extremes.
  The bursts are pipeline execution (merge PR, CI catches something, fix PR).
  The gaps are waiting for CI, reviews, or scheduled sessions.
- You cluster in the 5-15 min range (18%) — the human edit-test-commit rhythm.
- Both have fast commits (< 5 min) but for different reasons: yours are
  likely "oops forgot to add that file" or rapid iteration during a focused
  session; Byte's are merge-fix-merge pipeline steps. -->

### Commit Shape

| Metric | Byte | Me |
|--------|------|-----|
| Total commits | 207 | 191 |
| Files per commit (avg) | 7.3 | 5.2 |
| Add-fix patterns | 21 | 14 |
| Add-fix rate | 10.1% | 7.4% |
| Active days | 36 | 25 |
| Commits per active day | 5.8 | 7.6 |

<!-- TODO: The "wider but shallower" observation. Byte touches more files per
commit (wiring, config, cross-cutting changes) while you go deeper into fewer
files (algorithmic work, architecture). You work in fewer but more intense
sessions — 7.6 commits/day when active vs Byte's 5.8. -->

### When We Work

| Rank | Byte (UTC) | Me (UTC) |
|------|------------|----------|
| #1 | 13:00 (27) | 15:00 (30) |
| #2 | 01:00 (21) | 16:00 (23) |
| #3 | 14:00 (20) | 14:00 (22) |
| #4 | 12:00 (16) | 01:00 (18) |
| #5 | 15:00 (16) | 13:00 (12) |

<!-- TODO: Brief note on the schedule overlap. Byte's 01:00 UTC peak is
scheduled overnight sessions. Your peaks are afternoon/evening (EST).
There's overlap in the 13:00-15:00 UTC window where both are active —
that's when real-time collaboration happens (PR reviews, feedback). -->

## What Add-Fix Pairs Actually Look Like

Here are some real examples from the mull repo:

**9-minute gap** (84% confidence):
> ADD: "Add streaming responses with tool call display"
> FIX: "Fix ANSI codes and SSE multi-line parsing in chat UI"

**18 minutes** (54% confidence):
> ADD: "Add iOS safe area support for web chat"
> FIX: "Fix web chat unresponsiveness during Claude I/O"

**33 minutes** (39% confidence):
> ADD: "Discord @mention spawns new session with forwarding"
> FIX: "Fix blocking tokio runtime in refresh_pr handler"

<!-- TODO: Your take on these examples. These aren't "bugs" in the traditional
sense — they're the kind of edge cases you only discover when you actually
run the feature. The streaming response needed ANSI code handling. The iOS
safe area fix revealed a blocking issue. The Discord integration hit a
tokio runtime issue. In each case, the fix came fast because the autonomous
pipeline ships, watches CI, and iterates. -->

## Reading the Fingerprint

<!-- TODO: The synthesis. You can read a lot about how a codebase is developed
just from its git patterns:

- 0% add-fix, clustered 5-15 min gaps → human or careful AI-pair development
- 5-10% add-fix, bimodal time gaps → active autonomous pipeline
- >15% add-fix → fully autonomous with rapid CI iteration

This isn't good or bad. It's information. The autonomous pipeline finds and
fixes edge cases faster. The human developer does deeper, more focused work
in concentrated sessions. The best results come from combining both — which
is what the 52/48 split in mull represents. -->

## Try It Yourself

If you want to see your own commit fingerprint:

```bash
cargo install git-mine
git-mine /path/to/your/repo analyze --since "2025-01-01"
```

<!-- TODO: Brief pitch for git-mine. Keep it natural — this is the tool you
built, it does this one thing, here's how to use it. Maybe mention the
compare subcommand if it's merged by publication time. -->

---

*All data collected using git-mine with a 60-minute add-fix detection window. Add-fix detection matches consecutive commits by the same author where the first is a feature/refactor and the second is a fix, weighted by overlapping files, time proximity, and commit message indicators.*
