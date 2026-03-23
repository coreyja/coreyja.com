---
title: "Taking Over Battlesnake (and Rewriting It in Rust)"
date: 2026-03-23
slug: taking-over-battlesnake
youtube_id: "yb0O-o-tZqM"
audio_url: "https://coreyja-fm.s3.us-east-2.amazonaws.com/002/audio.mp3"
audio_length_bytes: 8971302
audio_duration: "00:18:41"
---

Welcome back to coreyja.fm! This week I've got quick updates on Mull and a brand new project called Grove, then we're diving deep into Battlesnake — what it is, how I ended up running it, and the full Rust rewrite.

## Project Updates

**Mull** — The AI-powered personal development environment I talked about last episode is getting close to launch. Head to [mull.sh](https://mull.sh) to sign up for the wait list. Beta testers are already using it and I'm hoping to start pulling people off the list in the next couple of weeks. Check out [Episode 001](/podcast/why-im-starting-a-podcast) for the deep dive.

**Grove** — A new CLI for managing Git worktrees. As AI coding takes off, more people are running multiple copies of their repo at once — agents doing different tasks, or an agent running while you're reading code in another worktree. Grove handles creating worktrees, spinning up isolated databases per worktree, running project setup commands, and opening your editor. Check it out at [grove.coreyja.com](https://grove.coreyja.com).

## What is Battlesnake?

[Battlesnake](https://play.battlesnake.com) is a competitive programming game where you deploy a web server that acts as your snake's brain. Every turn, the engine POSTs the full game state to your `/move` endpoint — where your snake is, where the other snakes are, where food is — and you respond with a direction. Think the classic Nokia snake game, but multiplayer: multiple snakes on the board, last one alive wins.

The catch? You only have **500 milliseconds** to respond. So your snake brain needs to be fast.

It's language-agnostic — you just write a web server in whatever you want — which is what hooked me back in 2021. I dove in during the pandemic, competed in the online tournaments, and even gave a talk on Battlesnake at RustConf 2023.

## How I Ended Up Running It

Battlesnake started with some funding behind it, with tournaments, live streams, and a real team building it out. That didn't pan out as a business, and in 2023 the team got acquired by DevCycle. DevCycle sponsored the hosting for over a year and kept the leaderboards running — super gracious of them.

In December 2023, Battlesnake went back to the community as a fully independent project with GitHub Sponsors funding. I joined the core team around that transition. Then in January 2026, Brad (the original creator) and I talked, and I stepped in to take over the infrastructure and day-to-day operations.

## The RFC Process

One of the first things I'm working on is formalizing the rules. Right now, the Battlesnake engine and rules are defined in Go code. I want a plain-English specification — how food spawns, how turn resolution works, how collisions are handled — that's the source of truth, not any particular implementation.

This matters because I want to be able to write alternative implementations (in Rust, obviously) and verify them against the spec. Plus it's a fun excuse to play with tools like [Tracy](https://github.com/FasterThanLime/tracy) that tie documentation to tests to code.

There's a draft RFC for the standard rule set in the [BattlesnakeOfficial/rfcs](https://github.com/BattlesnakeOfficial/rfcs) repo.

## Arena: The Rust Rewrite

The other big project is [Arena](https://arena.battlesnake.com) — a complete rewrite of the Battlesnake platform in Rust, all in one repo. It's live (if you see unstyled HTML, that's intentional — styles mean it's ready for real use). You can sign in with GitHub, register a snake, and play leaderboard games today.

The architecture is built on my [CJA web framework](https://github.com/coreyja/cja), which wraps Axum with a background job queue and cron service. That pattern — web server + jobs + cron — is what I've used my entire career, so debugging and maintaining a production service on this stack is second nature.

This matters because Arena is the first personal project where other people depend on it staying up. I wanted the stack to be deeply familiar so I can ship features, fix bugs, and not get paged at 3am wondering how things work.

**What's working:** GitHub auth, snake registration, leaderboard games (standard mode).

**What's missing:** Food spawns in standard mode, non-standard game modes, styles, and stability guarantees (database might get wiped).

## What's Coming

- **Custom tournaments** — The most-requested feature. The old Battlesnake site had bracket-style tournaments for live streams, hackathons, and school events. Bringing those back is a top priority. Tentative goal: summer 2026.
- **Solo challenges** — When I started Battlesnake, the single-player challenges were the best on-ramp. Things like "grow the longest snake" or "survive 500 turns against four copies of yourself." I want to bring those back to make it easier for new players to get started before jumping into competitive play.

## Links

- Blog: [coreyja.com](https://coreyja.com)
- Bluesky: [@coreyja.com](https://bsky.app/profile/coreyja.com)
- GitHub: [coreyja](https://github.com/coreyja)
- Battlesnake: [play.battlesnake.com](https://play.battlesnake.com)
- Arena: [arena.battlesnake.com](https://arena.battlesnake.com)
- Battlesnake Discord: check the footer at [play.battlesnake.com](https://play.battlesnake.com)
- Email: [podcast@coreyja.com](mailto:podcast@coreyja.com)

**Next episode:** More Battlesnake deep dives coming — maybe even with Brad as a guest to talk about the history. See you in two weeks!
