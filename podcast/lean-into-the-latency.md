---
title: "Lean Into the Latency (Why Async AI Workflows Win)"
date: 2026-04-07
slug: lean-into-the-latency
youtube_id: "PLACEHOLDER"
audio_url: "https://coreyja-fm.s3.us-east-2.amazonaws.com/003/audio.mp3"
audio_length_bytes: 9592599
audio_duration: "00:19:59"
transcript_url: "https://coreyja-fm.s3.us-east-2.amazonaws.com/003/transcript.srt"
---

Welcome back to coreyja.fm! This episode is all about one idea: latency is a feature, not a bug. I think AI agent workflows are fundamentally async, and the sooner we design around that instead of fighting it, the better off we'll be.

## The Core Argument

AI coding started with autocomplete — low-latency, bolt-on-to-existing-workflows stuff. That was the natural first step. But agent workflows have high latency by nature. You kick something off and come back 30 minutes or two hours later. Sitting there watching an agent work is fun but it's not the workflow we should be designing for.

The async approach is also more accessible. Fast mode exists, but it's expensive. Not everyone has millions to dump into low-latency inference. If you lean into the async nature, you can use normal-speed models or even local models running on consumer hardware. You're not sitting there watching — you start it, walk away, come back to results.

## Designing Around the Async Handoff

I think the industry needs to start acknowledging that async agent work is the norm, not the side quest. My workflow today is almost entirely async: hand off a task to an agent, get a PR or a plan back, review it, hand off the next step. It's not pair programming — it's more like managing a very fast junior developer who works while I do other things.

## When Sync Still Wins: Creative Work

Not everything should be async. Creative exploration — UI work, figuring out the shape of a problem, modeling something you don't fully understand yet — still wants synchronous flow. I use sync mode to explore and figure out what I want, then switch to async once the problem is well-defined enough to hand off.

## New Skills for an Async World

This shift makes some skills more important:

- **Context switching** — Working on multiple things while agents run in parallel becomes a real advantage.
- **Task decomposition** — Knowing when you have enough understanding to break work down, and how to break it down well. This has always been a skill, but it was more of a project management thing. Now it's core to the engineering workflow.
- **Knowing when to switch** — Recognizing when you're done exploring and it's time to hand off to the async pipeline.

## The Orchestration Problem

The move to async creates real orchestration challenges. You need notifications when agents finish. You need to chain agents together — in Mull, a plan agent finishes and a critique agent kicks off automatically. You need ways to snooze and come back to things. You need to handle the bottleneck where code is being written faster than you can review it (I currently have ~25 PRs waiting for review).

There are hard open questions: How much context do agents need? What happens when an agent hits a decision it can't make? Should agents run in parallel? How do you prevent collisions? Where does this run — local or cloud?

I think the current coding tools (Claude Code, OpenCode, Aider) are the building blocks. The next layer is orchestration on top of them. Maybe we get domain-specific agents instead of general-purpose ones. Nobody has the right answer yet, and that's what makes it exciting.

## A Prediction

In one year, the winners in AI-assisted coding will be the ones who embraced async. We'll move away from watching AI tools type out answers and toward a world where they produce results for us to review.

Give it a try: pick a task, decompose it, hand it off to an agent, and walk away. Come back later and see what you think.

## Links

- Blog: [coreyja.com](https://coreyja.com)
- Bluesky: [@coreyja.com](https://bsky.app/profile/coreyja.com)
- GitHub: [coreyja](https://github.com/coreyja)
- Mull: [mull.sh](https://mull.sh)
- Email: [podcast@coreyja.com](mailto:podcast@coreyja.com)

**Next episode:** Back in two weeks with more on what I've been building. See you then, team!
