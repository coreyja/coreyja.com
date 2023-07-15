---
title: coreyja weekly - July 13th Update
author: Corey Alexander
date: 2023-07-13
is_newsletter: true
---

First off, thanks for reading!
This is my very first newsletter so you are here for the very beginning.

We're going to keep these a bit more casual.
I want to give little updates about things I worked on during the week.
And then I want to talk a bit about some topic, today that'll be "My Perfect Web Framework".
Maybe these topics will make their way to full fledged post.
And maybe they won't.
No pressure!

So without further ado, let's kick off our first coreyja weekly!

---

## This Week

### Combat Reptile Dataset

A few weeks ago @ambadoom, the author of Combat Reptile, released a dataset with over 80,000 games that Combat Reptile played in!
It's an awesome dataset, and I'm super grateful that @ambadoon released it.
Even cooler, its licensed as [CC0](https://creativecommons.org/publicdomain/zero/1.0/) so its open for anyone to use.

I want to provide a way for the community to be able to browse the dataset online.
I'd recently heard about [Datasette](https://datasette.io) and this seemed like a perfect place for it.
So this week I got one setup, and you can check out the dataset on my [data.coreyja.com](https://data.coreyja.com/combat-reptile_dump_2023-06-27) instance!

The only issue I had getting the dataset uploaded was that the original sqlite file used [`STRICT` mode](https://www.sqlite.org/stricttables.html).
And the version of sqlite on my Datasette instance didn't support STRICT.
After trying to get an updated sqlite, I realized it was easier to make a copy of the DB locally that didn't use `STRICT`.
With my new 'relaxed' sqlite file, I was able to upload it to Datasette without issues.

The last thing I added last week as the ability to decode the `zstd` encoded `record` field.
This is where the actual game JSON is located, and if you click to download the `game` format from my Datasette you will get the full decoded JSON!

In the future I'd love to figure out a way to get Datasette to emulate the battlesnake engine and send the game JSON down a websocket.
That would allow these saved games to be replayed in the standard [Board repo](https://github.com/BattlesnakeOfficial/board)!

### Blog Open Telemetry

The other thing I did this week was upgrade my OpenTelemetry setup.
I'm using [`tracing`](https://docs.rs/tracing) and OpenTelemetry to send to traces to Honeycomb.
At the start of the week I was getting request headers in Honeycomb as a single JSON blob.
What I wanted instead was to have a way to query for individual header values, and was hoping that Honeycomb would decode that JSON blob for me.

I asked in a Discord I'm in what people were using for OTEL cloud providers, and also explained what I was working on.
And the basic answer (said much nicer than this!) was that I was using OTEL wrong.
Oops. 😆

So on stream on Wednesday we fixed that!
Instead of sending a single JSON blob we pull out the specific ones we care about, and attach them more manually.
A bit more code written, but I was also doing the oopsie of sending _ALL_ the headers before. This could very well include secret values in the future.
The new version that lists the specific headers we want it much better.
While we were there we also added a few response headers and other relevant values.

Check out the [diff here](https://github.com/coreyja/coreyja.com/commit/32cdb0dec5bd7b1c88fb3e1d4543ff13344c8e65)

## My Perfect Web Framework

Sometime soon I'd like to create a course about building a web framework from the ground up.
And to do that I have to figure out what the web framework I'd want to build would look like.
What things are important to me.
Which things from existing frameworks do I want to steal.
And what do I want to cut.

So that's what this section is for!
It's by definition going to be very personal.
I'm not trying to say this would necessarily be the _best_ framework.
It's just the one I personally want to build.
And hopefully use too!

We'll call this framework MPWF.

MPWF is written in a strongly typed language.
I think type systems are powerful tools for library builders.
They allow us to model our problems in ways so that the compiler can enforce the rules we need consumers to follow.
For instance if something can only be called once, we can ideally encode that into the type system, so that it _can't_ ever be called twice.

Today my current favorite and top candidate is Rust.
I also just have _fun_ coding in Rust.
And for MPFW, that's what matters.

MPWF pushes as much as possible into the type system, so that Rust can check as much as possible for us.

For the things even a strongly typed language can protect against, like logic bugs, there is testing.
Testing isn't an afterthought in MPWF.
It's baked into the framework.
MPWF nudges you into "full stack integration" TDD.
Where when you are adding a new user facing feature, you first work to describe what you want the user to do in the form of a test.
Since this is a web framework, that likely means spinning up a headless browser, and clicking buttons and filling in forms.
And the best way to access this testing will be through accessible HTML.
MPWF will focus on producing (and consuming) accessible HTML patterns.

And since we are already spinning up a browser and using our app, why don't we record demo videos?
We can use them to debug issues while building, or for showing off complicated flows and edge cases.

MPWF will focus on high level integration tests and allow anything lower level to the individual developer.

So far I've said a lot about backend-y focused things.
But MPFW is full stack.
Which likely means it's going to compile to WASM to run in browsers.
MPFW will provide any necessary JS glue so that developers only need to program in Rust.

MPFW on the frontend focuses on the HTML.
It does not have a virtual DOM, and does not follow the React paradigm.
MPFW is an exploration on different frontend techniques.
To be honest this is the part I have fleshed out the least, and the parts I have the least experience with.
But I want to do something different, so MPFW is part experiment on the frontend.

Since we are programming in a single language it should feel 'seamless' to exchange data between the frontend and backend.
MPFW doesn't go as far as to say you won't notice what's on the frontend vs backend.
It just provides a nice strongly typed bridge between the two.
This also means it provides tools to handle "Blue/Green" deploys, where we have two different versions of the app running.
Clients could connect to either a "blue" or "green" for each request.
MPFW makes it impossible to have blue/green issues at deploy time, by pushing this into the type system.
If you make a blue/green "mistake" at the bridge layer, your code won't compile.

MPFW is a developers first framework.
And as such it focused on the developer experience.
In service of this it has a dedicated language server that runs locally and integrates with your editor.
It can help you hop around the repo, and provide additional contexts to aid in editing and refactoring code.

MPFW includes OpenTelemetry integrated by default, and encouraged in documentation and patterns.
All the way through to tests, everything reports.

MPFW provides both background job and cron support.
It works off multiple data storages, but specifically focus on sqlite.
This decision is to enable small hobby deployments to run on a single server, without additional external components.

And as I mention deploys, MPFW deploys to a single executable.
Assets and templates, even the frontend WASM blob, included.
This may lead to a big binary, but that is not the focus of MPWF.
This makes deploys easy.
Even better docker container generation is handled by MPWF.

And way more!
Each time I start writing this paragraph, I think about another thing to jot down.
But I think we'll call it here for today.
Stay tuned for more updates on MPWF!
Most of these features are likely a ways off, but we'll build it together and have a blast doing it!

What does Your Favorite Web Framework look like? Reach out and let me know!
