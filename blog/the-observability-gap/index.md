---
title: "The Observability Gap: Why Your Agent Traces Are All Green and Your Output Is Still Wrong"
author: Corey Alexander
date: 2026-04-13
tags:
  - ai
  - agents
  - observability
  - mull
---

*Your agent tracing dashboard says everything is fine. Every span completed. Every tool call succeeded. Token costs are within budget. Latency is nominal. And the agent just deleted the wrong files.*

---

## The Dashboard That Lies

Here's what a modern agent observability tool shows you after a pipeline run:

```
Pipeline: implement-feature-xyz
Status: COMPLETED
Duration: 4m 32s
LLM Calls: 12
Total Tokens: 48,291
Tool Calls: 27 (27 succeeded, 0 failed)
Cost: $0.84
```

Every metric is green. The trace shows a clean execution path. Input prompts, output completions, tool invocations — all recorded, all successful.

The agent was supposed to update a config string from `v1` to `v2` across four files. It did that. It also reformatted the surrounding YAML, added comments, and reorganized key ordering. The diff is 200 lines when it should have been 4.

Where in that dashboard would you see the problem?

## What We Trace Today

The current generation of agent observability tools evolved from application performance monitoring (APM). They answer the question: **did the agent run?**

LangSmith gives you execution traces as trees of runs — every LLM call, tool invocation, and chain branch with token counts, latency percentiles, and cost breakdowns. Langfuse does similar with an open-source stack, adding prompt version management so you can A/B test prompt changes against cost and quality metrics. Phoenix layers in OpenTelemetry-native spans with embedding drift visualization — useful for catching when your RAG retrievals start returning irrelevant context. AgentOps is specifically designed for multi-agent systems, tracking inter-agent handoffs and per-agent cost attribution. Braintrust goes furthest on evaluation, letting you convert production traces into eval datasets and surface quality scores in CI/CD pull requests.

What they all track:

- **LLM call metadata**: model, tokens in/out, latency, cost per call
- **Execution flow**: which functions called which, in what order, with what arguments
- **Tool call results**: success/failure, return values, timing
- **Chain/agent topology**: how steps connect, where branching happens
- **Aggregate metrics**: success rates, cost trends, latency distributions

This is genuinely useful. If your agent crashes, times out, or burns through your API budget, these tools will tell you. If a tool call returns an error, you'll see it in the trace. If latency spikes, you'll know which step is slow.

But this is the equivalent of monitoring an API with HTTP status codes. A `200 OK` tells you the server responded. It doesn't tell you if the response was correct.

Some tools are reaching toward evaluation. Braintrust bakes LLM-as-judge scoring into the trace workflow. Phoenix offers built-in hallucination and QA correctness evaluators. Galileo recently launched "Tool Selection Quality" metrics claiming 93-97% accuracy at detecting when an agent chose the wrong tool. But these all require *you* to define what "correct" means — they provide the evaluation machinery, not the ground truth. And they all use another LLM as the judge, which means your correctness check is itself probabilistic.

## The Gap

The bugs that matter most in agent systems are the ones where everything "works":

**The Wanderer** reads the wrong file. The trace shows `read_file` succeeded, contents returned. It can't tell you the agent navigated to the wrong directory first.

**The Overachiever** applies the requested fix plus unrequested changes. Every tool call succeeds. The trace can't tell you that 90% of the changes weren't asked for.

**The Narrow Optimizer** makes tests pass by reverting a reviewer's intentional change. CI goes green. The trace shows "agent fixed test failure" — exactly what it was supposed to do.

In all three cases, the observability data says the agent succeeded. A human looking at the output says it failed. The gap between those two assessments is the observability gap.

This isn't hypothetical. Air Canada's chatbot hallucinated a bereavement fare policy that didn't exist. Every operational metric was green — normal latency, successful completion, no errors. The chatbot confidently told a customer about a discount policy the airline had never offered. Every observability tool on this list would have shown a clean dashboard for that session.

## Why APM Analogies Break Down

Traditional APM works because web services have a relatively simple success model. An API endpoint either returns the right data or it doesn't. You can validate response schemas, check status codes, compare against test fixtures.

Agent success is harder to define because agents are creative executors. The same prompt might produce ten valid outputs. "Did the agent do the right thing?" requires understanding intent, not just checking output format.

Consider how you'd write an assertion for each of these:

- "The agent should have only modified lines containing the version string" — requires understanding the diff semantics, not just whether the edit tool succeeded
- "The agent should not have explored directories outside the project root" — requires tracking the agent's navigation pattern, not just the final file reads
- "The agent should have prioritized the reviewer's comment over the test failure" — requires understanding the priority relationship between two pieces of context

None of these are expressible as span attributes or metric thresholds. They're behavioral properties of the agent's decision-making process.

## What Observability 2.0 Needs

If current tools answer "did the agent run?", the next generation needs to answer "did the agent do what I meant?"

### Behavioral Traces, Not Just Execution Traces

Current traces show you the sequence of operations. Behavioral traces would show you the sequence of *decisions*:

- What information did the agent seek before acting?
- What alternatives did it consider and reject?
- What was the agent's stated rationale for each action?
- Did the agent's exploration pattern match the expected scope?

When our Wanderer bug hit, the execution trace showed `read_file("/path/to/transcript.md") -> success`. What we actually needed to see was: "Agent received path `/correct/transcript.md` in prompt. Agent then explored `ls .`, found `scratch/`, listed `scratch/`, found a different `transcript.md`, and read that one instead."

The trail of exploration *before* the final action is where the bug lives. Current tools don't capture it because it looks like normal tool usage — just some `ls` and `read` calls that all succeeded.

### Diff-Level Validation

For code-modifying agents, the simplest useful assertion is: **how big was the diff compared to how big it should have been?**

If a task says "change string X to Y in 4 files," the expected diff is ~4-8 lines. If the actual diff is 200 lines, something went wrong — even if every line in the diff is valid code.

```
Expected diff scope: 4 files, ~8 lines changed
Actual diff scope: 4 files, 214 lines changed
ALERT: Diff 26x larger than expected
```

This isn't AI. It's arithmetic. And it would have caught our Overachiever bug immediately.

### Environment Snapshots

When an agent session starts, capture the state of its working environment:

- What files are visible?
- What's in the working directory?
- Are there multiple files with similar names that could cause confusion?

When our Wanderer agent explored the filesystem and found the wrong transcript, the problem was discoverable from the environment: there were multiple `transcript.md` files in reachable directories. An environment audit before execution would have flagged the ambiguity.

### Intent Assertions

The hardest but most valuable layer: letting developers express *what they meant*, not just what they asked for.

This might look like:

```yaml
intent:
  scope: "modify only version strings"
  files: ["config.yml", "package.json", "Cargo.toml", "pyproject.toml"]
  constraints:
    - "no formatting changes"
    - "no comment additions"
    - "no key reordering"
  max_diff_lines: 20
```

The agent receives the same prompt it always did. But the observability layer validates the output against the intent specification. Violations trigger alerts, not just logs.

This is analogous to property-based testing. You don't specify the exact output — you specify properties the output must satisfy. "The diff should only contain version string changes" is a property. "The diff should be less than 20 lines" is a property. Current tools can't express or check either one.

## What We Actually Built

In Mull, we've started building some of these layers — not as a general-purpose observability tool, but as pipeline guardrails:

**Diff size validation**: After the implementation step, we check the diff size against the plan's scope. An implementation plan that says "update 3 files" shouldn't produce a 500-line diff.

**Task spec quality gates**: Before a plan is even written, we validate that the task definition is specific enough to implement. Vague tasks produce vague implementations that are harder to validate.

**PR review as behavioral assertion**: Our pipeline includes an AI reviewer that checks the implementation against the plan. "Did the changes match what was planned?" is a behavioral question, not an operational one.

**Pipeline step isolation**: Each step runs in its own session with its own worktree. If a step's output is wrong, the blast radius is contained. The next step can validate the previous step's work before building on it.

None of this shows up in a trace dashboard. It's all in the pipeline logic itself — assertions embedded in the workflow rather than observed from outside.

## The Fundamental Problem

Here's why agent observability is hard: **the ground truth lives in the developer's head.**

When a web service returns wrong data, you can compare it against a database query or a test fixture. The expected output exists somewhere in the system.

When an agent produces wrong output, the only reference for "wrong" is the developer's intent — which was expressed as a natural language prompt, interpreted by a non-deterministic model. There's no fixture. There's no schema. There's just the gap between what someone meant and what the model found most probable.

Current observability tools can't close this gap because they're looking at the wrong layer. They're watching the machine execute. They need to be watching whether the execution aligns with human intent.

## Toward Behavioral Observability

The path forward isn't replacing current tools — they're necessary for operational health. It's layering behavioral observability on top:

**Layer 1: Operational** (current tools solve this)
- Did the agent run? Did it crash? How much did it cost?

**Layer 2: Structural** (partially solved)
- Did the output have the right format? Did it touch the expected files?

**Layer 3: Behavioral** (mostly unsolved)
- Did the agent do what was intended? Did it stay within scope? Did it consider the right context?

**Layer 4: Semantic** (open research)
- Is the output *correct* in the domain sense? Does the code work? Does the analysis make sense?

Most teams are stuck at Layer 1. A few have Layer 2 through ad-hoc validation. Almost nobody has Layer 3. Layer 4 might require the agent itself to participate in its own evaluation — which introduces its own set of problems.

## The APM Lesson

When web APM was new, teams monitored uptime and response times. That was enough until it wasn't. Then they added tracing (Zipkin, Jaeger). Then distributed tracing. Then synthetic monitoring. Then real user monitoring. Each layer caught failures the previous layer missed.

Agent observability is at the "uptime and response times" stage. We track whether agents run and how much they cost. The next decade will be about building the behavioral tracing, intent validation, and semantic monitoring layers that catch the failures our current dashboards can't see.

The OpenTelemetry AI working group acknowledged this directly: "We're shifting from observing 'system behavior' to observing 'semantic quality,' which means we can't just record what was called — we need to understand why it was called and how good the result was." As of early 2026, this remains aspirational.

The tools that crack behavioral observability — that can tell you "the agent stayed within scope" or "the agent's exploration pattern was unusual" — will be as transformative for agent systems as distributed tracing was for microservices.

Until then, your traces will keep showing green while your agents do the wrong thing.

---

*All examples are from [Mull](https://mull.dev), a persistent AI agent system in production use.*
