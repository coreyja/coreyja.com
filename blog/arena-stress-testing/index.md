---
title: Stress Testing Battlesnake Arena
author: Corey Alexander
date: 2026-02-08
tags:
  - battlesnake
  - rust
  - infrastructure
---

<!-- TODO: Intro paragraph. What is Arena, why stress test it, what prompted this -->

## The Best Run: 6,500 Games With No Rate Limiter

<!-- TODO: Context - this was the final run after removing the rate limiter, simulating a full leaderboard calculation -->

| Metric | Value |
|---|---|
| Total games attempted | 6,501 |
| Successfully created | 6,501 (100%) |
| Failed (rate limited) | 0 |
| Avg creation latency | 434ms |
| p50 creation latency | 450ms |
| p95 creation latency | 605ms |
| p99 creation latency | 670ms |

<!-- TODO: Reaction to these numbers - 100% creation, sub-500ms latency -->

### Processing Throughput

The server processed games at a steady ~102 games/min regardless of queue depth. With 6,500 games queued, the system would take roughly 63 minutes to fully drain.

| Metric | Value |
|---|---|
| Games finished (in 15 min window) | 1,837 / 6,501 (28.3%) |
| Stuck (running) | 25 |
| Still waiting | 4,639 (71.3%) |
| Effective throughput | 102.2 games/min |
| Estimated full drain time | ~63 minutes |

### Server-Side Queue Drain Over Time

| Time (s) | Games Waiting | Games Running | Games Finished (1h) | Jobs Ready |
|---|---|---|---|---|
| 0 | 6,236 | 383 | 3,265 | 6,235 |
| 60 | 6,128 | 382 | 3,375 | 6,127 |
| 120 | 6,007 | 383 | 3,495 | 6,006 |
| 180 | 5,904 | 383 | 3,597 | 5,903 |
| 240 | 5,804 | 383 | 3,697 | 5,803 |
| 340 | 5,627 | 383 | 3,874 | 5,626 |
| 440 | 5,455 | 383 | 4,046 | 5,454 |
| 530 | 5,288 | 383 | 4,213 | 5,287 |
| 630 | 5,116 | 383 | 4,385 | 5,116 |
| 720 | 4,947 | 382 | 4,556 | 4,946 |
| 800 | 4,793 | 383 | 4,708 | 4,792 |
| 891 | 4,634 | 381 | 4,758 | 4,633 |

<!-- TODO: Talk about the steady drain rate and what this means for leaderboard feasibility -->

### Server-Side Peak Stats

| Metric | Value |
|---|---|
| Peak jobs ready | 6,323 |
| Peak jobs running | 56 |
| Peak jobs scheduled | 328 |
| Peak games waiting | 6,324 |
| Peak games running | 383 |
| Games finished (1h window) | 4,758 |

<!-- TODO: Discuss graceful degradation - queue depth doesn't affect processing speed -->

## Before vs After: Rate Limiter Impact

<!-- TODO: Talk about how the rate limiter was accidentally acting as a load shedder -->

### Same Test, Rate Limiter On vs Off

| Metric | Run 4 (Rate Limited) | Run 5 (No Rate Limiter) |
|---|---|---|
| Target games | 6,500 | 6,500 |
| Creation success | 4.3% (377 / 8,668) | 100% (6,501 / 6,501) |
| Avg creation latency | 9,036ms | 434ms |
| Games finished | 377 (100% of created) | 1,837 (28.3% of created) |
| Stuck games | 0 | 25 |
| Effective throughput | 85.0 games/min | 102.2 games/min |
| Peak queue depth | 105 ready | 6,323 ready |

<!-- TODO: The rate limiter was rejecting 95.7% of requests. Once removed, creation latency dropped 20x -->

## The Journey: How We Got Here

<!-- TODO: Brief narrative of the day - started with baseline, iterated through configs -->

### Infrastructure Config Evolution

| Config | Rev 00050 | Rev 00051 | Rev 00052 | Rev 00053 (Final) |
|---|---|---|---|---|
| Memory | 512Mi | 2Gi | 4Gi | 4Gi |
| CPU | 1000m | 1000m | 1000m | 1000m |
| Tokio Workers | 100x | 2x | 2x | 2x |
| PG Max Connections | default | 30 | 55 | 55 |
| Job Workers | 50 | 50 | 50 | 25 |
| Job Poll Interval | 200ms | 200ms | 200ms | 200ms |

<!-- TODO: Talk about each change and why it was made -->

### 100-Game Benchmark Across Configs

| Config | Run | Completed | Stuck | Time | Throughput |
|---|---|---|---|---|---|
| Baseline (1-2 workers) | 1 | 99 / 100 | 1 | ~23 min | 4.3/min |
| 50 workers, 55 PG, 2Gi | 3 | 100 / 100 | 0 | ~6 min | 16.7/min |
| 50 workers, 55 PG, 4Gi | 4 | 100 / 100 | 0 | ~90 sec | 67/min |
| 25 workers, 55 PG, 4Gi | Run 2 (stress binary) | 274 / 274 | 0 | 288s | 56.9/min |

<!-- TODO: The 15x improvement from baseline to optimal config -->

### Full Run Comparison (stress-test Binary)

| Config | Run | Games Created | Finished | Stuck | Time | Throughput |
|---|---|---|---|---|---|---|
| Warm rate limiter | 2 | 274 | 274 (100%) | 0 | 288s | 56.9/min |
| Cold rate limiter, high concurrency | 3 | 486 | 562 (95.9%) | 24 | 485s | 69.5/min |
| Rate limited leaderboard sim | 4 | 377 | 377 (100%) | 0 | 266s | 85.0/min |
| No rate limiter, leaderboard sim | 5 | 6,501 | 1,837 (28.3%) | 25 | 1,078s | 102.2/min |

<!-- TODO: Narrative about each run building on findings from the last -->

## The Bottleneck Progression

<!-- TODO: Each section below is a bottleneck we found and fixed, in order -->

### 1. Connection Pool Contention

<!-- TODO: 50 workers competing for 30 connections = 20 always blocked -->

| Metric | Before (30 conn) | After (55 conn) | Change |
|---|---|---|---|
| DB write avg | 397ms | 170ms | -57% |
| DB write max | 738ms | 267ms | -64% |
| Scheduler jitter avg | ~2.5ms | 146us | -94% |
| Scheduler jitter max | ~10ms | 2.7ms | -73% |

<!-- TODO: Write latency included pool checkout wait time, so the "DB was slow" was actually "pool was starved" -->

### 2. OOM Kills

<!-- TODO: Workers dying mid-game, leaving jobs locked for 2 hours -->

#### Memory Profile (100 Games, 50 Workers, 4Gi)

| Time | Memory Usage |
|---|---|
| Idle | 320 MiB |
| +2 min | 953 MiB |
| +4 min | 2,157 MiB |
| +7 min | 2,841 MiB |
| +9 min | 3,477 MiB |
| +10 min (peak) | 3,792 MiB (93% of limit) |
| +11 min (draining) | 3,683 MiB |
| +12 min (done) | 356 MiB |

<!-- TODO: ~38 MiB per concurrent game. Memory freed completely after games finish (no leak). -->

#### OOM Impact on Game Completion

| Run | Workers | Memory | Games | Completed | Stuck | Notes |
|---|---|---|---|---|---|---|
| Run 6 | 50 | 4Gi | 100 | 63 | 37 | OOM killed mid-batch |
| Run 7 | 50 | 4Gi | 90 | 50 | 40 | Same pattern |
| Run 9 | 50 | 4Gi | 1000 | ~83 visible | 17+ | Burst-crash-restart cycles |
| Run 3 (binary) | 25 | 4Gi | 486 | 562 | 24 | Higher concurrency config |
| Run 2 (binary) | 25 | 4Gi | 274 | 274 | 0 | Controlled load |

<!-- TODO: Reducing workers from 50 to 25 halved memory pressure and mostly eliminated OOM -->

### 3. Rate Limiter

<!-- TODO: How we discovered the rate limiter was the ceiling -->

#### Rate Limiter Behavior at Different Batch Intervals

| Interval | Batch Size | Success Rate | Effective Create Rate |
|---|---|---|---|
| Every 2 min (cold start) | 100 | 100% (batch 1) | 100/batch |
| Every 2 min (steady state) | 100 | 70-74% | ~72/batch |
| Every 1 min (warm) | 100 | 47-70% | ~55/min |
| Every 1 min (cold) | 100 | 89-100% | ~97/min |
| Every 1 min (leaderboard) | 2,167 | 4.3% | ~126/min |
| Post-deploy (no limiter) | 2,167 | 100% | 2,167/batch |

<!-- TODO: The rate limiter was per-user and didn't distinguish stress tests from normal usage -->

#### Batch-by-Batch: Rate Limiter Impact (Run 1)

| Batch | Time | Attempted | Succeeded | Failed | Success % | Avg Latency | p50 | p95 | p99 |
|---|---|---|---|---|---|---|---|---|---|
| 1 | 0:00 | 100 | 100 | 0 | 100% | 2,091ms | 2,134ms | 3,396ms | 3,501ms |
| 2 | 2:00 | 100 | 74 | 26 | 74% | - | - | - | - |
| 3 | 4:00 | 100 | 70 | 30 | 70% | - | - | - | - |
| 4 | 6:00 | 100 | 71 | 29 | 71% | - | - | - | - |

#### Batch-by-Batch: No Rate Limiter (Run 5)

| Batch | Time | Attempted | Succeeded | Failed | Success % | Avg Latency |
|---|---|---|---|---|---|---|
| 1 | 0:00 | 2,167 | 2,167 | 0 | 100% | 469ms |
| 2 | 1:00 | 2,167 | 2,167 | 0 | 100% | 408ms |
| 3 | 2:00 | 2,167 | 2,167 | 0 | 100% | 425ms |

## Game Completion Performance

<!-- TODO: How long do games actually take to run? -->

### Completion Latency by Run

| Run | Games | p50 Completion | p95 Completion | p99 Completion |
|---|---|---|---|---|
| Run 2 (274 games) | 274 | 26.9s | 52.2s | 60.6s |
| Run 3 (486 games) | 562 | 42.1s | 76.2s | 123.3s |
| Run 4 (377 games) | 377 | 39.7s | 64.0s | 68.2s |
| Run 5 (6,501 games) | 1,837 | 557s | 1,013s | 1,054s |

<!-- TODO: At lower volume, games take ~27-42s median (reasonable for a 60-100+ turn game with 200ms snake response time). At 6500 games the wait in queue dominates -->

### Polling Timeline: How Fast Did Games Finish? (Run 4, 377 games)

| Time After Polling Start | Unfinished Games | % Done |
|---|---|---|
| 0s | 377 | 0% |
| 5s | 131 | 65% |
| 25s | 94 | 75% |
| 45s | 54 | 86% |
| 65s | 17 | 95% |
| 75s | 2 | 99% |
| 85s | 0 | 100% |

### Polling Timeline: 6,500 Games (Run 5)

| Time After Polling Start | Unfinished Games | Finished (1h) |
|---|---|---|
| 0s | 6,236 | 3,265 |
| 120s | 6,007 | 3,495 |
| 240s | 5,804 | 3,697 |
| 440s | 5,455 | 4,046 |
| 630s | 5,116 | 4,385 |
| 891s | 4,634 | 4,758 |

## Server-Side Stats Across Runs

<!-- TODO: What the admin endpoint revealed -->

| Metric | Run 1 | Run 2 | Run 3 | Run 4 | Run 5 |
|---|---|---|---|---|---|
| Peak jobs ready | - | 38 | 179 | 105 | 6,323 |
| Peak jobs running | 32 | 32 | 56 | 56 | 56 |
| Peak games waiting | 76 | 39 | 94 | 106 | 6,324 |
| Peak games running | 359 | 359 | 383 | 383 | 383 |
| Admin snapshots | - | 57 | 179 | 44 | 216 |

## Stuck Games Analysis

<!-- TODO: Why games get stuck and what causes it -->

| Root Cause | Symptom | Fix |
|---|---|---|
| OOM kill mid-game | Workers die, job lock held for 2 hours | Reduce workers (50 -> 25) |
| Deploy mid-game | Same as OOM - workers interrupted | Graceful shutdown / drain |
| High concurrency config | More games accepted than workers can handle | Match Cloud Run concurrency to worker capacity |

### Stuck Games by Run

| Run | Total Games | Stuck | % Stuck | Root Cause |
|---|---|---|---|---|
| Baseline Run 1 | 100 | 1 | 1% | Deploy mid-game |
| Run 6 (50 workers) | 100 | 37 | 37% | OOM kill |
| Run 7 (50 workers) | 90 | 40 | 44% | OOM kill |
| Run 9 (50 workers, 1000 games) | 1000 | 17+ | ~2%+ | OOM cycles |
| Binary Run 2 (25 workers) | 274 | 0 | 0% | - |
| Binary Run 3 (25 workers) | 486 | 24 | 4.9% | OOM at higher concurrency |
| Binary Run 5 (25 workers, 6500) | 6,501 | 25 | 0.4% | Occasional OOM spikes |

## Numbers That Matter

<!-- TODO: The key takeaways, one sentence each -->

| What | Number |
|---|---|
| Baseline throughput | 4.3 games/min |
| Optimized throughput (100 games) | 67 games/min |
| Sustained throughput (6500 games) | 102.2 games/min |
| Improvement over baseline | ~24x |
| Memory per concurrent game | ~38 MiB |
| Baseline memory | ~320 MiB |
| Creation latency (no rate limiter) | 434ms avg |
| Creation latency (rate limited) | 9,036ms avg |
| Game completion time (p50) | ~27-42s |
| Time to drain 6,500 games | ~63 minutes |
| Max concurrent games observed | 383 |

## What's Next

<!-- TODO: Where do we go from here? Scaling options, leaderboard feasibility, next steps -->

### Scaling Options for Leaderboard

| Option | Estimated Impact | Tradeoff |
|---|---|---|
| Double workers (25 -> 50) + double memory (4Gi -> 8Gi) | ~200 games/min | 2x cost |
| Optimize per-game memory (reduce ~38 MiB footprint) | More concurrency at same memory | Engineering effort |
| Direct job queue insertion (skip HTTP API) | Eliminates creation overhead | Bypasses API validation |
| Accept 63-minute window | No changes needed | Leaderboards delayed |

<!-- TODO: Final thoughts -->
