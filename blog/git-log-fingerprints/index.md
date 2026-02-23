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

I've been running an AI agent called Byte alongside my own development work for about a year now. Byte plans features, implements them, opens PRs, and self-corrects through CI feedback — all autonomously. I review PRs and handle the deeper architectural decisions.

I got curious about what this actually looks like in the commit history, so I built a tool and pointed it at my repositories. The patterns were more distinct than I expected.

## The Setup

I analyzed 12 repositories spanning January 2025 through February 2026 — roughly 900 commits across projects ranging from fully autonomous AI development to human-only coding. The tool is [git-mine](https://github.com/coreyja-studio/git-mine), a Rust CLI that extracts patterns from git history.

<!-- TODO: Link to Battlesnake 2026 post for context on AI agent coding philosophy once published -->

## Add-Fix Rate: An Iteration Speed Signal

The "add-fix rate" measures how often a feature commit is followed by a fix commit within 60 minutes by the same author. It's not a quality metric — it's an iteration speed metric. A high rate means problems are found and fixed *fast*, not that there are more problems.

| Repo | Commits | AI % | Add-Fix | Rate | Style |
|------|---------|------|---------|------|-------|
| sql-check | 16 | 100% | 3 | **18.7%** | Fully autonomous |
| mull | 575 | 61% | 57 | **9.9%** | Autonomous pipeline |
| skills | 18 | 77% | 2 | **11.1%** | Mostly autonomous |
| GAR | 24 | 79% | 2 | **8.3%** | Mostly autonomous |
| tournaments | 73 | 49% | 3 | **4.1%** | AI-pair |
| coreyja.com | 103 | 0% | 3 | **2.9%** | Human-primary |
| cja | 32 | 28% | 0 | 0% | AI-pair |
| eyes | 33 | 36% | 0 | 0% | AI-pair |
| grove | 7 | 100% | 0 | 0% | Autonomous |
| stamp | 12 | 100% | 0 | 0% | Autonomous |
| formchat | 13 | 100% | 0 | 0% | Autonomous |
| battlesnake-rs | 5 | 100% | 0 | 0% | Autonomous |

The correlation isn't perfect. Grove, stamp, formchat, and battlesnake-rs are all 100% AI-authored but show 0% add-fix — though they also have fewer than 15 commits each, where the rate is statistically unreliable. The signal emerges in repos with 50+ commits where the pattern has room to show up.

The pattern that does emerge: autonomous AI development produces a distinctive ship-CI-fix loop. The agent pushes code, CI catches an issue, the agent fixes it — all within minutes. This isn't more bugs. It's the system self-correcting through CI feedback faster than a human review cycle would.

## The Mull Deep Dive

Mull is the most interesting dataset: 575 commits split 61/33 between Byte and me (the rest are release-please bot commits). Same codebase, same time period, two very different development rhythms.

### Time Between Consecutive Commits

| Gap | Byte | Me |
|-----|------|-----|
| < 5 min | 60 (17%) | 27 (14%) |
| 5-15 min | 33 (9%) | 51 (27%) |
| 15-30 min | 31 (9%) | 32 (17%) |
| 30-60 min | 48 (14%) | 15 (8%) |
| 1-3 hours | 88 (25%) | 30 (16%) |
| > 3 hours | 90 (26%) | 33 (18%) |

Byte is bimodal: 17% rapid-fire bursts plus 26% long gaps — 43% at the extremes. The bursts are pipeline execution: merge a PR, CI catches something, push a fix. The long gaps are waiting for CI, waiting for reviews, or gaps between scheduled sessions.

I cluster in the 5-15 minute range (27%) — the human edit-test-commit rhythm. We both have fast commits (< 5 min) but for different reasons: mine are rapid iteration during a focused session; Byte's are merge-fix-merge pipeline steps.

### Commit Shape

| Metric | Byte | Me |
|--------|------|-----|
| Total commits | 351 | 189 |
| Files per commit (avg) | 6.9 | 6.1 |
| Add-fix patterns | 43 | 14 |
| Add-fix rate | 12.3% | 7.4% |
| Active days | 50 | 20 |
| Commits per active day | 7.0 | 9.4 |

Byte touches slightly more files per commit — cross-cutting changes, wiring, config — while I tend to go deeper into fewer files during focused sessions. I work in fewer but more intense bursts: 9.4 commits per active day vs Byte's 7.0. Byte just shows up more days.

### When We Work

| Rank | Byte (EST) | Me (EST) |
|------|------------|----------|
| #1 | 8 AM (44) | 9 AM (32) |
| #2 | 7 AM (32) | 10 AM (23) |
| #3 | 8 PM (29) | 8 PM (18) |
| #4 | 9 PM (26) | 3 PM (17) |
| #5 | 9 AM (25) | 12 PM (14) |

Byte's 7-8 AM peak is its daily autonomous session. My peaks are morning work (9-10 AM) and evening (8 PM). We overlap in the 9 AM window where real-time collaboration happens — PR reviews and feedback. Byte also has a secondary evening peak from interactive sessions.

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

These aren't bugs in the traditional sense — they're the kind of edge cases you only discover when you actually run the feature. The streaming response needed ANSI code handling. The iOS safe area fix revealed a blocking issue. The Discord integration hit a tokio runtime problem. In each case, the fix came fast because the autonomous pipeline ships, watches CI, and iterates without waiting for a human to notice.

## Reading the Fingerprint

You can read a lot about how a codebase is developed just from its git patterns:

- **0% add-fix, clustered 5-15 min gaps** — human development or careful AI-pair sessions
- **5-10% add-fix, bimodal time gaps** — active autonomous pipeline with CI feedback loops
- **15%+ add-fix** — fully autonomous with rapid CI iteration

This isn't good or bad. It's information. The autonomous pipeline finds and fixes edge cases faster than I would in a review cycle. I do deeper, more focused work in concentrated sessions. The git log captures both rhythms, and the combination — 61% autonomous, 33% human — is what makes the project move.

## Try It Yourself

If you want to see your own commit fingerprint:

```bash
cargo install git-mine
git-mine /path/to/your/repo analyze --since "2025-01-01"
```

The tool detects add-fix pairs, calculates time gaps between consecutive commits, and breaks down patterns by author. It's the same tool I used for all the data in this post.

---

*All data collected using git-mine with a 60-minute add-fix detection window. Add-fix detection matches consecutive commits by the same author where the first is a feature/refactor and the second is a fix, weighted by overlapping files, time proximity, and commit message indicators.*
