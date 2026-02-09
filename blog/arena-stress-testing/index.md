---
title: Stress Testing Battlesnake Arena
author: Corey Alexander
date: 2026-02-08
tags:
  - battlesnake
  - rust
  - infrastructure
---

Hey Team!

I've been making some good progress on the new Battlesnake in Rust re-write, called Arena, and this weekend ran some stress tests against it and wanted to share the results!

Let's start with some good headline results before I yammer for a bit! The basic idea of these tests was to help me tune various parameters and measure performance for some simulated Leaderboard type scenarios.
For a real leaderboard run on Battlesnake I estimated there are about 6,500 games a day. I could have counted games on the real server, but a rough estimate is fine. That was my target this weekend for stress testing! See how fast and how well I can handle a bulk of 6 or 7 thousand games!

The server processed games at a steady ~102 games/min regardless of queue depth. With 6,500 games queued, the system would take roughly 63 minutes to fully drain.

| Metric                            | Value                 |
| --------------------------------- | --------------------- |
| Games finished (in 15 min window) | 1,837 / 6,501 (28.3%) |
| Stuck (running)                   | 25                    |
| Still waiting                     | 4,639 (71.3%)         |
| Effective throughput              | 102.2 games/min       |
| Estimated full drain time         | ~63 minutes           |

**But there are some disclaimers to these numbers!**

While the number of games is tuned to about a Leaderboard run, the total drain time is NOT really how fast I think we could run a leaderboard. This is the 'best case' scenario for drain time, as I am running a snake that returns really fast. In real life we'll need to wait on snake HTTP calls, so the real drain time is probably 4-5x that estimate. That's a rough guess, but I'll validate it with proper latency testing soon enough!

You might also notice 25 games stuck in a "running" state in the table above. Those are games that didn't finish cleanly during the test window. Still tracking that down.

For the snakes here I'm using Bombastic Bob, [one of my snakes](https://terrarium.coreyja.com/) who chooses a random direction to go in each time. For most of the tests here Bob was returning right away, and with network latency was clocking in right around 50-60ms. I'll follow up with some more testing where Bob sleeps for different amounts of time each move to simulate a more realistic game. But for today I wanted to see how far I could push the Arena codebase.

## The Tuning

Arena is deployed on Google Cloud Run, and the main knob I was turning this weekend was **job workers** — basically the number of games the server runs concurrently. I started with everything turned down, only 4 workers. And that's really why the initial throughput was so slow, about ~4 games/min. More workers means more concurrent games, so cranking that up was the obvious first move.

But it wasn't that simple! As I increased workers I started hitting out-of-memory kills. At one point I had 25 workers on 4 GB of RAM and was _still_ getting OOM'd. I spent a while chasing that, bumping memory allocations up, before I found the real culprit: my observability stack.

I had a custom metrics pipeline that was creating an **unbounded queue** of telemetry events. With 25 games running concurrently and verbose logging, that queue was just ballooning in memory unchecked. Once I ripped all that out, memory usage dropped to about **80 MB**. So from 4 GB to 80 MB — the games themselves are pretty lightweight, I was just overloading the server with my own telemetry.

And once the memory issue was resolved, everything else got better too. The server wasn't constantly fighting for resources anymore, so I could actually test the worker count properly.

I pushed all the way up to **100 workers** on a single vCPU, but jitter started getting too high. Jitter here is the amount of time it takes for the async scheduler to come back around and give our game tasks control again. If jitter gets too high, Arena might not be able to accurately time snake responses — which matters when you're enforcing move timeouts.

So the sweet spot looks like it's somewhere between **25 and 100 workers per vCPU**. That said, jitter might not matter as much if I have region endpoints handle the timing instead of the game server itself. That's still an open design question.

## What's Next

The big takeaway: on a single vCPU with some config tuning, we went from 4 games/min to 102 games/min — a **24x improvement**.

Next up is testing with realistic snake response times by adding artificial latency to Bob, and figuring out if we need to scale horizontally or if a beefier single instance gets us there. Stay tuned!
