---
title: Battlesnake in 2026
author: Corey Alexander
date: 2026-01-10
tags:
  - battlesnake
  - rust
---

I've been involved with Battlesnake for 5 years now, [since 2021](https://github.com/coreyja/battlesnake-rs/commit/ebd84eecc9ca89f7c05259a60c8ff764b469435f) when I built my first snake. It feels like both forever and no time at all.

But as of this month, I'm taking over and running the Battlesnake infra going forward. After Battlesnake was [given back to the community](https://docs.battlesnake.com/blog/2023/12/27/battlesnake-needs-your-support-in-2024) Brad was running the infra on his own and out of his pocket. At the end of December 2025, he was ready to step down and after some long discussions, we decided that I would take over and run the Battlesnake infra! And that's basically done as of now.

play.battlesnake.com is and will continue to be running! Oh and I'm going to be re-writing it in Rust.

---

Two of the new things that I'm excited about right now are community access to game archives and bringing back custom tournaments. I talked about both of these a bit more in the [official blog post](https://docs.battlesnake.com/blog/2026/01/05/battlesnake-2026-update).

Custom Tournaments and ideally Leaderboards, will be part of my re-write. The biggest open question with leaderboards is game volume. I'm hoping to have my new infra ready for some load testing this month; and after that I should have a better idea how many games we can support.

The game archives are being recorded and stored as you read this! This archiving is the first part of the new `arena` service, which will be the main hub of my consolidation and Rustification of the infra. It has a read-only connection to the existing engine DB, and it's archiving new games to Object Storage every hour. These are currently stored individually for each game, and are optimized for replay. Next up is a way to view games straight from Object Storage via this Rust service so that we can keep games accessible for viewing longer term. The biggest cost will actually become retrieval fees over bandwidth likely, as these are small objects. But the costs should be minimal, at least for legitimate game replay. And I'm hoping some kind of bulk offerings might stop the worst of the scrapers.

## Object Storage Archive Deep Dive

The current bucket storage is per game_id, which makes it less useful for bulk-y purposes like ML training or engine verification. The second one there is the one that I'm the most interested in. I want a large set of games to verify my Rust rules engine against. And I think it would be a good thing to provide to the community as well.

So I plan on rolling up games on some basis, maybe daily to start, and bundling them so you could download a single day's worth of games, which would contain 20-30k games in an archive. This would make it relatively easy to script up downloading a whole year's worth of games, and maybe at the end of the year I'll think about going one yearly bundle as well.

The key insight that led to thinking about this is that the storage of game records is pretty small, especially after compression. So the main costs are actually retrieval fees, not storage fees; especially for bulk access. So by duplicating data for different use cases, we can hopefully have the best of both worlds.

While I have a decent handle on how I want to do this technically, I'm less sure about how to release it to everyone, both in terms of fairness and costs. It's something that I'm thinking of doing tiered access where free access is maybe gated on Battlesnake auth but limited. And with a GitHub Sponsorship you get extended access. But if this is to be used for ML training, want to balance being fair to players while encouraging donations.

## Engine verification

I mentioned that my personal use case for the large archive of games, is to verify the correctness of other Battlesnake game engines. And by game-engine I mean a bit of code that evaluates an existing board, and moves for each snake and returns a result.

I have an existing engine, and I've done some light 'fuzz' testing with random moves, but having a lot of games to run against will be great! There are still some gotchas, like the randomness of food spawns but I'm confident we can find a solution. For the randomness I think it might be 'fun' to have a specific pseudo-random number generator and publish the seed with each game, to encourage verification. I'm also thinking it would be fun to have a 'leaderboard' of other engines. We could compare across languages for completeness, and potentially speed. Let me know if you'd be interested in this, just another random tangent you could work on in Battlesnake land!

## The Rust Rewrite

I've got to get it out of the way. Yes, I am planning on re-writing a lot of Battlesnake code in Rust. But no, it's not cause I think Rust is better than the original languages. It's simply because for something I'm maintaining in my spare time, I want it to be in a language and environment, I know and like. And right now for me that's Rust.

A code base you authored, in a language and environment you are familiar with is a good code base to work in. And that's what I want for the Battlesnake infra.

We aren't going to be able to replace everything in one go, but most pieces of the Battlesnake infra are candidates for Oxidation.

For now I am working on a repo currently called `arena`, which is my Battlesnake Webserver and Game Runner. It will likely also be where the 'official' Rust engine will live. I considered `core` as a better repo name if it ends up housing multiple components, so let me know if you have any thoughts on that.
This repo currently houses a basic GitHub OAuth flow and Snake Registration pages, as well as the crons that power the Archive system.

## Embracing AI Agent Coding

One thing I knew I wanted to include in this post was my feelings on AI Coding as it relates to Battlesnake and these infra changes. And then I read this post by Rain on Bluesky that captured it perfectly:

> I'd much rather people spend the extra time improving code quality. Do the thing that's annoying to write and has many edge cases to test but has a cleaner UX. Aggressively refactor. Tackle the TODOs
>
> — [Rain (@sunshowers.io)](https://bsky.app/profile/sunshowers.io/post/3mbcik4ld6s2q)

If you poked around the repo, you might have gotten the sense that there was some AI coding happening, and you wouldn't be wrong. I find myself writing less and less code by hand nowadays, and writing almost exclusively with AI agents.

And one thing it's definitely allowed me to do is build new features for personal projects that wouldn't have existed otherwise. Secondary views for things, better error handling and documentation. And this is what I want to extend to the Battlesnake infra and core code bases.

AI Coding, not to lower the quality of the code or community but to raise the bar on developer and player experience.

---

I'm incredibly excited for Battlesnake in 2026. Running a production system that powers millions of games a month is the kind of challenge I love — and I can't wait to see what this community builds next.

If you want to help keep the servers running, consider [sponsoring BattlesnakeOfficial on GitHub](https://github.com/sponsors/BattlesnakeOfficial). Every contribution helps cover infrastructure costs and keeps the games going.

See you in the arena.
