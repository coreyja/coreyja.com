---
title: "Why I'm Starting a Podcast (and What I've Been Building)"
date: 2026-03-06
slug: hello-world
youtube_id: "B12ZggV47rg"
youtube_url: "https://youtu.be/B12ZggV47rg"
audio_url: "https://coreyja-fm.s3.us-east-2.amazonaws.com/001/audio.mp3"
audio_length_bytes: 10777931
audio_duration: "00:24:30"
transcript_url: "https://coreyja-fm.s3.us-east-2.amazonaws.com/001/transcript.srt"
---

Welcome to the first episode of the coreyja.fm podcast! I'm Corey, I go by coreyja online, and I've been doing web dev for about 20 years. These days I'm focused on Rust and I recently took over leading the Battlesnake project.

This is a solo podcast about what I'm building, what's breaking, and what I'm learning. I've got a lot of side projects going and I want to do a better job sharing them with the world.

## What I Cover

**Project Tour** — A quick rundown of everything I'm actively working on:

- **Mull** — An AI agent orchestration and memory system. My "personal development environment" that remembers context across sessions and automates my development workflow.
- **Battlesnake** — I took over hosting and maintenance of [play.battlesnake.com](https://play.battlesnake.com) at the start of 2026 and I'm rewriting the engine in Rust.
- **GAR** — A tool to manage self-hosted GitHub Actions runners on a spare Mac, because my CI bills hit $100/month with all the AI-driven PRs.
- **Bake** — Takes WordPress sites and turns them into static sites served from a single Rust binary. One server, multiple sites, all from memory.
- **Lavender Iguana** — The business my wife and I run making WordPress sites for local businesses.
- **Small CLIs** — Stamp (Fastmail integration), Porkbun (domain management), Quiver (skills manager), and more.

**Deep Dive: The Mull Autopilot Pipeline** — The main event. I walk through the full pipeline that takes a task from idea to pull request:

1. **Task quality check** — A small model verifies the task has acceptance criteria and enough detail. If not, it launches an interactive session to interview you and flesh it out.
2. **Plan draft** — An agent drafts an implementation plan from the task.
3. **Enrich** — A dedicated step that searches the memory/learnings library for relevant past experience. This took memory utilization from ~3% to ~97%.
4. **Critique & revise loop** — Different models critique and revise the plan (Gemini for critique, Opus for revision). Loops until approved or hits a max round count.
5. **Human review** — The pipeline pauses here. You review the plan in markdown, edit it, chat about it, then approve.
6. **Implementation & review loop** — Writes failing tests first, then implements and reviews in a loop until the review agent approves.
7. **Pull request** — Opens a PR for final human review.

Over one weekend, this pipeline produced 42 merged PRs across 3 projects, with a 92% first-pass CI success rate. It's not magic — it's a productivity multiplier that lets me make progress on code while doing dishes or playing with my daughter.

## Links

- Blog: [coreyja.com](https://coreyja.com)
- Bluesky: [@coreyja.com](https://bsky.app/profile/coreyja.com)
- GitHub: [coreyja](https://github.com/coreyja)
- Battlesnake: [play.battlesnake.com](https://play.battlesnake.com)
- Email: [podcast@coreyja.com](mailto:podcast@coreyja.com)

**Next episode:** Battlesnake — where it's at, where it's going, and the Rust rewrite. See you in two weeks!
