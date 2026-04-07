---
title: "Lean Into the Latency"
date: 2026-04-07
slug: lean-into-the-latency
youtube_id: "TODO"
audio_url: "https://coreyja-fm.s3.us-east-2.amazonaws.com/003/audio.mp3"
audio_length_bytes: 0
audio_duration: "00:20:00"
---

AI coding tools started with autocomplete — fast, low-latency, bolted onto existing workflows. But I don't think that's where we end up. In this episode I make the case that latency is the *feature* of AI agent workflows, not a bug we need to eliminate.

## The Core Idea

Agent workflows are inherently async. You kick off a task, come back 30 minutes later, review the result. Trying to make that synchronous — watching the agent type, waiting for it to finish — fights the natural shape of the work. And making it faster (like Claude Code's fast mode) costs real money. Not everyone has millions to burn on low-latency inference.

The alternative: lean into the async. Use normal-speed models, or even local models. It's fine if they're slow — you're not watching. You start something, go do something else, come back when it's done.

## When Async Doesn't Work

Not everything fits the async model. I've found that **creative exploration** — the work where you don't know what the end result looks like yet — really wants synchronous flow. UI work, architecture exploration, figuring out the shape of the problem. That's pair programming territory.

But once you know the shape? That's when you hand it off. Write the plan, decompose the tasks, let the agents implement. The creative synchronous work feeds into the async implementation pipeline.

## New Skills for an Async World

If this is the direction things are heading, some skills get more important:

- **Context switching** — Working on multiple things in parallel instead of one deep flow state. This is controversial and I'm not claiming it's easy.
- **Task decomposition** — Knowing when you have enough understanding to break work down, and how to slice it so an agent can execute independently.
- **Async handoff design** — Building workflows where the human makes decisions and the agent does the heavy lifting, with clean handoff points between them.

These aren't new skills — they're project management skills that have always existed. But they might become core software engineering skills instead of adjacent ones.

## The Orchestration Problem

"Kick off a task and come back later" sounds simple, but it opens a huge design space:

- **Notifications** — You need to know when it's done. But not be spammed.
- **Chaining** — One agent finishes a plan, another agent critiques it, a third implements. How do you wire that up?
- **Decision points** — What happens when the agent hits a question that isn't answered anywhere? Should it guess? Ask you? How?
- **Parallelism** — Multiple agents on different tasks. How do you keep them from conflicting?
- **The review bottleneck** — Right now I have 25 PRs waiting for my review. Agents produce code faster than I can review it. That's a new kind of problem.

I think the current crop of coding tools — Claude Code, OpenCode, Pye — are the building blocks. The orchestration layer that ties them together is still being figured out.

## The Prediction

In one year, I think the winners of AI coding workflows will be the ones who embraced async. We'll move away from needing to watch our AI tools type and toward a world where they produce results for us to review.

Try it this week: pick a task, hand it to an agent, walk away. Come back. Look at the result. Do it five times. See how it feels. Let me know what you think.

## Links

- Blog: [coreyja.com](https://coreyja.com)
- Bluesky: [@coreyja.com](https://bsky.app/profile/coreyja.com)
- GitHub: [coreyja](https://github.com/coreyja)
- Mull: [mull.sh](https://mull.sh)
- Email: [podcast@coreyja.com](mailto:podcast@coreyja.com)

**Next episode:** Back in two weeks with more on what I've been building. If you have thoughts on async vs sync AI workflows, I'd love to hear from you.
