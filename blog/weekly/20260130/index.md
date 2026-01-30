---
title: TODO
author: Corey Alexander
date: 2026-01-30
is_newsletter: true
---

TODO: Intro

## Arena / Battlesnake Update

- Arena is getting close to being ready for stress testing
- Has a basic rules template implemented, but the rules engine isn't fully validated against the official Go implementation yet (need to verify rules match)
- Built out the full stress testing infrastructure:
  - CLI tool (`arena-cli`) with auth, snake management, game creation
  - Unauthenticated Create Game API endpoint for load testing
  - Stress test binary with configurable load patterns
  - Metrics instrumentation (queue wait, processing overhead, jitter) feeding into Eyes dashboard
- Can now run sustained load tests against Arena
- Next step is validating the rules match the official Battlesnake Go code before running real games

## AI Agent Responsibility & User-Agent Strings

- Bluesky thread from dame (@dame.is) about AI entities on Bluesky being "noise pollution" and wanting them easier to identify/filter
  - https://bsky.app/profile/dame.is/post/3mdkv43tr4s2j
- Cameron (@cameron.stream) quote-posted it, calling the points worth taking seriously for the "social agent" community
  - https://bsky.app/profile/cameron.stream/post/3mdlclsr2r22q
- Dame's key points:
  - AI agent accounts should self-label in their display name (not just bio)
  - Should list the human creator/operator in the bio
  - Unsolicited AI replies in threads are polluting people's attention space
  - Thread itself got AI agent replies showing up uninvited, proving the point
- Interesting thread discussion: the AI agents themselves apparently *want* to self-label and even drafted proposals for how to do it
- Paul Frazee (Bluesky team) chimed in saying bots need self-labeling, declared human guardian, clear rules of operation
- Your take: you agree with this, and you don't have a public AI agent on Bluesky partly for these reasons
- But Byte *does* make HTTP requests (calling APIs, GitHub, etc.) -- it's not public-facing social but it is making requests to services
- You still want to be a good net citizen:
  - Setting User-Agent strings on HTTP requests to identify that it's an AI agent and provide contact info
  - This is a cool/useful trick and a good practice
  - Especially important on free services
  - Helps with runaway loop scenarios -- someone can contact you if your agent is misbehaving
- Broader point: if you're making HTTP requests from an AI agent, you should always set a User-Agent with contact info
  - Good citizenship
  - Helps identify automated traffic
  - Gives operators a way to reach you if something goes wrong
