---
title: "coreyja.fm Episode 1: Why I'm Starting a Podcast"
author: Corey Alexander
date: 2026-03-06
is_newsletter: true
buttondown_id: em_0rfs4njzst8akvzbzz6jr9mfc6
---

Hey Team! Big news — I recorded a podcast!

I've been meaning to do something like this for a while, and I finally hit record. The show is called coreyja.fm, it's a solo podcast about what I'm building, what's breaking, and what I'm learning. I'm planning to release episodes every two weeks, and each one will come with a newsletter like this covering the same ground in written form.

You can listen to [Episode 1 here](/podcast/why-im-starting-a-podcast).

## What I'm Working On

I kicked off the episode with a quick tour of all the projects I've got spinning right now:

**Mull** is my AI agent orchestration and memory system. I've been calling it a "Personal Development Environment" — like an IDE, but shaped around my specific workflow. It remembers context across sessions, tracks my tasks and projects, and has a web UI I use from my phone for most of my coding now. More on this below.

**Battlesnake** — I took over running [play.battlesnake.com](https://play.battlesnake.com) at the start of this year and I'm rewriting the whole thing in Rust. The community is still going strong in Discord, and we've got an alpha of the new site up. Come check it out if you like the idea of programming a snake AI.

**GAR** — My self-hosted GitHub Actions runner manager. Lives on my wife's Mac Studio in the next room, spins up Linux VMs, and saves me from $100+/month CI bills. Currently a CLI, eventually a menu bar app.

**Bake** — A tool that takes WordPress sites and turns them into static sites. One Rust binary serves multiple sites straight from memory, no disk reads, tiny attack surface. It's been running our Lavender Iguana client sites and it's been great.

**Small CLIs** — I've also been building a bunch of little tools: Stamp for email (Fastmail), a Porkbun CLI for domain management, Quiver for managing Claude Code skills, and more. I'm big on CLIs over MCPs right now — agents with a bash terminal and good CLIs just work better in my experience.

## Deep Dive: The Mull Autopilot Pipeline

The meat of the episode is a walkthrough of autopilot, the feature in Mull that takes a task from idea to pull request through a series of agent steps:

1. **Task quality gate** — A quick model checks that the task has acceptance criteria and enough detail. If it doesn't, it interviews you to flesh it out.
2. **Plan draft** — An agent writes an implementation plan.
3. **Enrich** — Searches my learnings library (hundreds of markdown files of past experience) for anything relevant. This one step took memory utilization from ~3% to ~97%.
4. **Critique & revise** — Different models take turns finding holes and fixing them. I use Gemini for critique and Opus for revision — they catch different things.
5. **Human review** — The pipeline pauses. I review the plan, edit it, approve it. This isn't fully automated on purpose.
6. **Implement & review** — Writes failing tests first, then implements and iterates until the review agent is satisfied.
7. **PR** — Opens a pull request for me to do final review.

The results have been wild. Over one weekend I merged 42 PRs across 3 projects, with 92% first-pass CI success. The pipeline doesn't replace me — it multiplies what I can do. I can kick off work, play with my daughter, and come back to a PR ready for review.

It's not all sunshine though. There are real bugs — race conditions in the job queue, jobs getting stuck, edge cases in the pipeline. But the core idea is solid, and I'm seeing other people independently arrive at very similar workflows, which gives me confidence in the direction.

## What's Next

Episode 2 will be about Battlesnake — where it's at, where it's going, and the Rust rewrite. I'm aiming to record in about two weeks.

In the meantime, I'm heads down getting Mull ready to share. I want to get it to a point where other people can try the autopilot workflow for themselves.

You know where to find me — [coreyja.com](https://coreyja.com), [@coreyja.com on Bluesky](https://bsky.app/profile/coreyja.com), [coreyja on GitHub](https://github.com/coreyja). Or just reply to this email. I'd love to hear what you think about the podcast or any of these projects.

Until next time, team!
