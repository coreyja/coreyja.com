---
title: "Agents as Infrastructure (The Two-Pillar Play)"
date: 2026-05-06
slug: agents-as-infrastructure
youtube_id: "6MAvzL7uIfk"
audio_url: "https://coreyja-fm.s3.us-east-2.amazonaws.com/004/audio.mp3"
audio_length_bytes: 8285222
audio_duration: "00:17:15"
transcript_url: "https://coreyja-fm.s3.us-east-2.amazonaws.com/004/transcript.srt"
---

Welcome back to coreyja.fm! This episode is all about one idea: agents are the next infrastructure wave, and the winners are going to be the companies that own *both* pillars — a real data moat **and** the agent infrastructure that runs on top of it. Either pillar alone makes you a strong player. Both is what makes you a 10x winner of this cycle.

## The Two Pillars

There are two moats forming in the agent space, and they map to two different kinds of advantage:

- **The data pillar.** This is what every incumbent with a real data set has been sitting on for the last two decades. Agents are the first technology that can really *use* it across structured and unstructured data at scale.
- **The agent infrastructure pillar.** The layers above the model: retrieval, memory, tool runtimes, evals, identity, audit. The plumbing that turns a model into a product.

Have one pillar and you're competitive. Have both and you can do things nobody outside your company can replicate.

## Why It'll Mirror the Cloud

I think this is going to play out the same shape the cloud did. We'll get the AWS-style players; the ones that own the underlying infrastructure and let you plug in whatever you need to build an agent. AWS already has most of the pieces: EC2, ECS, containerization, isolation, GPUs. They're in a really good spot to be one of the long-term winners here, as are the other hyperscalars.

We'll also see Heroku-style players — the high-end-of-the-stack folks where you just plug in and get an agent. Both ends of that spectrum will work.

The squeezed middle? Generic agent frameworks with no infrastructure underneath and no real UX edge on top. That's the empty space.

## Inference Is the New Compute Bill

The vertical integration argument starts with cost. SaaS scaled gracefully because each incremental use was essentially free — you'd already paid for the database and the web servers, and another query was just slotting into existing capacity. LLM inference breaks that pattern. Every time you summarize something for a user, there's a real per-use cost — and when you're paying frontier-model API prices, you feel it fast.

The companies that get back to "scaling like software did" will be the ones that own enough of the stack to run their own weights for the work that doesn't need a frontier model. With open-weight models getting genuinely good, you don't need to ditch closed-frontier models entirely — you mix and match. Foundation models for the user-facing reasoning where intelligence really matters; locally-run open-weight models for the background work like memory retrieval, summarization, and pre-fetching.

Owning that layer means you stop paying someone else's margin on every single agent call.

## Quality and UX Compound

The cost story is the easy half. The harder, more interesting half is what vertical integration does for *quality*.

When you own the data layer, citation gets better. Evaluation gets better — you can grade outputs against domain truth, not benchmark scores. Hallucination detection gets easier. Permissions and audit can flow end-to-end because identity is mapped through the whole stack.

And the UX compounds the same way Apple's hardware-software integration compounds. Bluetooth just works between my Mac and my AirPods because Apple owns every piece of that handshake. Copy-paste between my devices just works because the same company built the OS, the hardware, and the sync layer. Vertical integration looks slow until you start stacking the wins, and then it's a structural advantage nobody can match by gluing third-party products together.

Domain-native interfaces, trusted outputs, workflows that match how the work actually happens — that's the difference between an agent that demos well and an agent that gets used on Monday morning.

## Why Both Pillars Matter

Owning one pillar puts you in a strong position. Owning both is where the loop closes:

- Your data shapes the infrastructure you build for agents.
- Your infrastructure shapes how you organize and serve your data.
- That feedback loop happens *inside one company*, where iteration cycles are measured in days instead of vendor procurement cycles.

You can't replicate that loop from the outside. If you're trying to assemble it from third-party pieces, the seams will always be slightly misshapen — products that almost fit but not quite.

## Not About Building a Frontier Model

To be clear: I'm not saying anyone should go build their own foundation model. The frontier labs aren't going anywhere, and for the parts of an agent system where reasoning quality really matters — the user-facing layer especially — you should absolutely use the best frontier model you can.

But there are *lots* of pieces of an agent system that don't need a frontier model: retrieval, memory organization, summarization, classification, routing. That's where vertical integration with open-weight models pays off, and where every dollar of inference margin you stop renting is structural advantage you keep.

## What I'm Watching

Both pillars are visibly forming today. The data side: every incumbent with a real corpus is figuring out how to make their data agent-friendly. The infra side: AWS Bedrock keeps adding agent primitives, [exa.sh](https://exa.sh) is targeting agent use cases specifically, Replit pivoted toward this. Both verticals exist, both are growing, and what I'm excited to watch over the next 24 to 60 months is which companies merge them — and which ones keep them separate and end up renting half their stack from somebody else.

I think owning the full vertical integration is going to be a really fun problem to solve. It's the one I'm most excited about for the next few years.

## Links

- Blog: [coreyja.com](https://coreyja.com)
- Bluesky: [@coreyja.com](https://bsky.app/profile/coreyja.com)
- GitHub: [coreyja](https://github.com/coreyja)
- Mull: [mull.sh](https://mull.sh)
- Email: [podcast@coreyja.com](mailto:podcast@coreyja.com)

**Next episode:** Back in two weeks. Let me know what you think — do you see these two pillars converging, or staying separate? I'd love to hear your take. See you then, team!
