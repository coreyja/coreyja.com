---
title: "coreyja.fm Episode 3: Lean Into the Latency"
author: Corey Alexander
date: 2026-04-07
is_newsletter: true
---

Hey Team! Episode 3 is out — this one's about why I think async is the natural mode for AI agent workflows, and why fighting the latency is the wrong move.

You can listen to [Episode 3 here](/podcast/lean-into-the-latency).

## The Big Idea

AI coding started with autocomplete — low-latency, bolted onto existing workflows. That was a great first step, but I don't think it's where we end up. Agent workflows are high-latency by nature. You kick off a task and come back 30 minutes later. Instead of fighting that with expensive fast modes, I think we should lean into it and design our workflows around the async handoff.

This is more accessible too. Not everyone can afford fast-mode pricing. If you're okay with async, you can use normal-speed or even local models. You're not watching — you're doing other things and coming back to results.

## Sync Still Has a Place

Not everything should be async. Creative exploration — UI work, figuring out the shape of a problem you don't fully understand — still wants synchronous flow. I use sync to explore, then switch to async once I know what to ask for. The creative work is where pair programming with an agent still shines.

## New Skills

This shift makes some things more important: context switching between parallel tasks, knowing when to decompose work and hand it off, and building orchestration around the async handoffs (notifications, agent chaining, snoozing). In [Mull](https://mull.sh), once a plan finishes, a critique agent kicks off automatically — that kind of pipeline is where I think the tooling needs to go.

## The Bottleneck Problem

One real issue I'm hitting: code is being written faster than I can review it. I've got ~25 PRs waiting for review right now. That's a new kind of problem that doesn't exist in synchronous workflows. There are hard questions about agent context, parallel execution, and human-in-the-loop decisions that nobody has fully figured out yet.

## My Prediction

In one year, the winners in AI-assisted coding will be the ones who embraced async. We'll move from watching AI type to just reviewing what it produced.

Give it a try this week: pick a task, hand it off to an agent, walk away. Come back and see what you think.

As always — [coreyja.com](https://coreyja.com), [@coreyja.com on Bluesky](https://bsky.app/profile/coreyja.com), [coreyja on GitHub](https://github.com/coreyja). Or just reply to this email.
