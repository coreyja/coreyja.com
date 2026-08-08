---
title: "coreyja.fm Episode 4: Agents as Infrastructure"
author: Corey Alexander
date: 2026-05-07
is_newsletter: true
---

Hey Team! Episode 4 is out — this one's about why I think agents are the next infrastructure wave, and why the real winners are going to be the companies that own *both* a real data moat and the agent infrastructure that runs on top of it.

You can listen to [Episode 4 here](/podcast/agents-as-infrastructure).

## The Two Pillars

Two moats are forming in the agent space, and they map to two different kinds of advantage:

- **The data pillar.** What every incumbent with a real data set has been sitting on for the last two decades. Agents are the first technology that can actually *use* it across structured and unstructured data at scale.
- **The agent infrastructure pillar.** Everything above the model — retrieval, memory, tool runtimes, evals, identity, audit. The plumbing that turns a model into a product.

Have one pillar and you're competitive. Have both and you can do things nobody outside your company can replicate.

## It'll Mirror the Cloud

I think this plays out the same shape the cloud did. We'll get AWS-style players that own the underlying infrastructure and let you plug in whatever you need. AWS already has most of the pieces — EC2, ECS, isolation, GPUs — and the other hyperscalers aren't far behind.

We'll also see Heroku-style players at the high end of the stack, where you just plug in and get an agent. Both ends of that spectrum will work.

The squeezed middle? Generic agent frameworks with no infrastructure underneath and no real UX edge on top. I'd bet that space ends up much more barren.

## Inference Is the New Compute Bill

SaaS scaled gracefully because every incremental use was essentially free — you'd already paid for the database and web servers. LLM inference breaks that pattern. Every summary, every retrieval, every tool call has a real per-use cost, and at frontier-model API prices, you feel it fast.

The companies that get back to "scaling like software did" will be the ones that own enough of the stack to run their own weights for the work that doesn't need a frontier model. Foundation models for the user-facing reasoning where intelligence really matters; locally-run open-weight models for the background work — memory retrieval, summarization, pre-fetching. Owning that layer means you stop paying someone else's margin on every single agent call.

## Quality and UX Compound

Cost is the easy half. The harder half is what vertical integration does for *quality*. When you own the data layer, citation gets better. Evaluation gets better — you can grade outputs against domain truth, not benchmark scores. Hallucination detection gets easier. Permissions and audit flow end-to-end because identity is mapped through the whole stack.

The UX compounds the same way Apple's hardware-software integration does. Bluetooth between my Mac and AirPods just works because Apple owns every piece of the handshake. Vertical integration looks slow until the wins stack up — and then it's a structural advantage you can't match by gluing third-party products together.

## Not About Building a Frontier Model

To be clear: I'm not saying anyone should go build their own foundation model. The frontier labs aren't going anywhere, and for the parts of an agent system where reasoning quality really matters, you should use the best frontier model you can. But there are *lots* of pieces that don't need one — retrieval, memory organization, classification, routing — and that's where vertical integration with open-weight models pays off.

## What I'm Watching

Both pillars are visibly forming today. Every incumbent with a real corpus is figuring out how to make their data agent-friendly. AWS Bedrock keeps adding agent primitives, [exa.sh](https://exa.sh) is targeting agent use cases specifically, Replit pivoted toward this. What I'm excited to watch over the next 24 to 60 months is which companies merge the two pillars — and which ones keep them separate and end up renting half their stack from somebody else.

I think owning the full vertical integration is going to be a really fun problem to solve. It's the one I'm most excited about for the next few years.

As always — [coreyja.com](https://coreyja.com), [@coreyja.com on Bluesky](https://bsky.app/profile/coreyja.com), [coreyja on GitHub](https://github.com/coreyja). Or just reply to this email and tell me what you think — do you see these two pillars converging, or staying separate?
