---
title: "coreyja.fm Episode 2: Taking Over Battlesnake"
author: Corey Alexander
date: 2026-03-23
is_newsletter: true
---

Hey Team! Episode 2 is out — this one's all about Battlesnake.

You can listen to [Episode 2 here](/podcast/taking-over-battlesnake).

## Quick Project Updates

Before diving into Battlesnake, I covered a couple things:

**Mull** is getting close to launch. If you want to try it, head to [mull.sh](https://mull.sh) and sign up for the wait list — I'm hoping to start pulling people off in the next couple weeks.

**Grove** is a new CLI I built for managing Git worktrees. With AI agents writing code in parallel, you end up needing multiple copies of your repo running at once. Grove handles creating worktrees, spinning up isolated databases, running setup commands, and opening your editor. It's on GitHub at [coreyjastudio/grove](https://github.com/coreyjastudio/grove).

## The Battlesnake Story

The bulk of the episode is the Battlesnake deep dive. If you haven't seen it, [Battlesnake](https://play.battlesnake.com) is a competitive programming game — you write a web server that controls a snake, the engine POSTs the game state to your `/move` endpoint every turn, and you have 500ms to respond with a direction. It's multiplayer snake where the last one alive wins.

I got hooked in 2021, competed in tournaments, and gave a talk at RustConf 2023 about it. When the original team moved on and the project went community-driven, I joined the core team. Then in January this year, I took over the infrastructure and day-to-day operations from Brad (the original creator).

## What I'm Building

Two big things are happening:

**The RFC process** — The Battlesnake rules have always been defined by the Go implementation. I'm writing them up as a plain-English specification so we can build alternative implementations (in Rust, naturally) and verify them against the spec. There's a [draft RFC](https://github.com/BattlesnakeOfficial/rfcs) for the standard rule set already.

**Arena** — A complete rewrite of the platform in Rust, live at [arena.battlesnake.com](https://arena.battlesnake.com). You can sign in with GitHub, register a snake, and play leaderboard games today. It's built on my [CJA framework](https://github.com/coreyja/cja) (Axum + job queue + cron), which is a stack I know inside and out. That matters because this is my first personal project where other people depend on uptime.

## What's Coming

**Custom tournaments** are the most-requested feature — bracket-style events for live streams, hackathons, and school groups. Targeting summer 2026.

**Solo challenges** were the best on-ramp when I started playing. Things like "grow the longest snake" or "survive 500 turns." Bringing those back to make it easier for new players.

Next episode might have Brad on as a guest to talk about the history of Battlesnake. See you in two weeks!

As always — [coreyja.com](https://coreyja.com), [@coreyja.com on Bluesky](https://bsky.app/profile/coreyja.com), [coreyja on GitHub](https://github.com/coreyja). Or just reply to this email.
