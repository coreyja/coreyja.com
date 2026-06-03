---
title: My Vision for Battlesnake
author: Corey Alexander
date: 2026-06-03
tags:
  - battlesnake
---

I took over the Battlesnake infra at the start of this year, and since then I’ve been, on and off, trying to rebuild the platform using my own native Web stack in Rust. I’ve talked a bit about the why behind it, but the short version is I like Rust and it’s the language I want to write and maintain.

But one thing I haven’t talked about is my Vision for Battlesnake going forward. Not only the technical platform bits, but ways I see the community and game evolving!

I don’t have a plan for this post yet, so let’s see where we end up.

## Technical Vision

I’ve got a strong vision for what the technical side of the Battlesnake platform looks like. And if you know my [standard stack](https://cja.app/), you know where this is headed.

Battlesnake is moving towards a Rust Workspace, and everything will live in the [arena repo](https://github.com/BattlesnakeOfficial/arena).

This repo contains the main arena server, and a ‘rules’ crate that contains the (work in progress) Battlesnake rule engine.

The main arena server follows my Three Pillar philosophy, of having three distinct ‘processes’ that are the foundation for the application; web servers, job workers, and a cron scheduler.

This allows arena to ship as a single binary with everything it needs to run Battlesnake games, and handle tournaments etc in a single binary. Just plug in some env vars like a Postgres URL and you should be able to self host an arena instance.

Self hosting arena will always be possible, but will not be a primary driving factor for its design. It will be easy for me to build and maintain, and as a side effect it will likely be easy to self host. **But I want to be clear that self hosting is not an explicit goal of the project.**

Currently Arena _is_ live and deployed at [arena.battlesnake.com](https://arena.battlesnake.com). Currently you can sign in with Github Authentication, register your snakes, create one off games and even enroll your snakes in the global leaderboard that runs games automatically each day. You’ll notice that the site is lacking styles and any kind of usable UX, and that’s intentional. You are more than welcome to use it, but beware that until it looks usable it’s a tech preview alpha release! Once things are more stable and I’m confident I won’t wipe the DB and everyone’s results, we will get some styles added and make it more usable!

The eventual goal is to retire the `play` site, and shift all gameplay over to Arena!

## Community Vision

The community angle I think is the more important piece here. Cause we can build the best tournament runner around, but it’s not useful if nobody wants to compete in tournaments! So in the next year I want to double down on the Battlesnake community, and make it a place that people can keep coming back to, to improve their snakes and engage with other developers.

### Seasonal Leagues

And I think the first thing I want to get started again in this angle is the seasonal leagues. I think that having a reason to keep coming back and checking on your snake, and improving it is the first important piece to building the community back up.

And I think the seasonal leagues were perfect for that!

They give everyone a scheduled event to plan around and push towards, and a continuous way to measure if your snake is improving. One issue with the long running leaderboards is point inflation and issues of that nature, so having a time bound league that naturally expires and a new one begins can help both new and old snakes be competing on a level playing field.

And after the leagues wrap up, I think we should definitely do some live stream tournaments! Not going to commit to one after each league right now, but at least 2 a year feels good. Maybe we focus on the Summer and Winter leagues as the ‘competitive’ ones that end in Tournaments, and the Spring and Fall are more ‘recreational’ leagues that don’t have tournaments when they wrap up.

## Shipping the Re-Write

There is definitely some tension between the re-write and the community building angle. It’s important to me that we build up the community, and give you all good ways to compete in Battlesnake ASAP. And given that the re-write could be seen as a distraction, and it’s a tension I’ve been wrestling with.

I do want Tournaments and Leagues to be out so the community and start using them, but I want to make sure we do that sustainably. And sustainably for Battlesnake today, means that the infra needs to be hands off. It needs to run itself and be essentially maintenance free, since all the infra is run as a volunteer effort. We want the reliability and uptime of a professionally run platform, but on a shoe string budget with volunteer labor. And to do that I’m investing heavily into the infrastructure to make the vision a reality.

And I think the why this new version will be sustainable along those directions is in two pieces:

- First, the shape of the platform will match my mental model better. Making it easier and faster to debug and maintain.
- Second, we are focusing the re-write on these explicit goals. I know the outcome we are shooting for, and every decision can be weighed against the long term maintenance cost. And tooling and process can be shaped to fit the needs of the site admins and community.

And so I’m going to push along this re-write, but I’m going to cut a few corners that I was initially planning on doing. We’ll come back to them, but we need to get leagues and tournaments up and running first! The biggest corner I’m gonna cut if the full RFC process and Engine Verification step for the Rust re-write. We are going to use to Go rules code as the source of truth, and we aren’t going to spend excessive time trying to find each and every edge case between the Rust and Go rules. We will do a best effort first pass, and get the engine out live for everyone to play with! That way we can crowd source any issues that crop up. Having all of you be able to watch games and report any moves that don’t look right will be a huge win on its own, and getting there faster feels like the right move.

We are also going to put off things I started earlier this year, like being able to host your own private tournaments. That will absolutely be coming, I have a personal use case where I want a private tournament, but the public community aspects will come first.

Same for leaderboards that support multiple different scoring algorithms. A really cool feature, and something I want to build but not the top priority today.

## Longer Term: Getting new Players Engaged

The last thing I want to touch on is something that you all will know I’ve been excited about reviving, and that is Solo Challenges.

There were challenges that you would complete on your own, typically by coding a specific snake for the challenge. So one challenge was to grow to the longest length possible, and there were badges for making it to certain lengths.

I _loved_ these and completed almost all of them before really starting on a competitive snake for the leaderboards.

I think things like this are very important as on-ramps into Battlesnake. It’s intimidating to code a snake and just drop it into the leaderboard, especially when you see how the top snakes perform! And I hope by reviving some of the single player challenges, it gives people a way to start out and understand the game before having to dive right into competitive play.

And I think Solo Challenges being the thing to focus on after the public leagues and tournaments makes sense! First we need to have a reason for you all to keep coming back to Battlesnake, and that’s the leagues and tournaments. But after that I think it will be good to focus on the things that help bring new people into the community! Solo Challenges and even private tournaments fall into this second bucket.
