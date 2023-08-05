---
title: August 4th Update - Server Side vs Client Side Rendering
author: Corey Alexander
date: 2023-08-04
is_newsletter: true
---

Hey Team, how is everyone's Friday going?

I had a really fun week that I'm excited to share about!

And after that I think we'll talk about Client Side vs Server Side rendering, and how neither is inherently "faster" they just shift who does the computation.

## This Week

### Stream Backlog

I've been streaming pretty regularly for the last year or so. Which means I've got a decent backlog of videos. But unfortunately I've been bad about consistently uploading them to Youtube. Twitch VODs only last a couple weeks before they expire, and they don't have any discoverability. I've got all my old stream recordings in an S3 Bucket, but they needed to be organized and have titles and descriptions written.

Enter new AI models!

I've been using Whisper to transcribe the few streams I did upload to YouTube, but this week I had some time to expand on that idea!

I used Whisper to transcribe each video. I fed that transcript into GPT3.5 in chunks to summarize. This summary was fed into GPT4 to generate the titles and descriptions for each video! I did the Whisper transcription locally with [`whisper-rs`](https://github.com/tazz4843/whisper-rs) but used the OpenAI LLM models for the summarization and description generation. It would be fun to try a Llama 2 model for this sometime, to see how a fully local model can compare!

The results are currently being added to YouTube! I also scripted the final upload, but that means I have to obey the API rate limits now! I've already maxed out my uploads for today, but be on the look out for more videos from the backlog to trickle onto my YouTube channel.
Be sure to subscribe if you don't want to miss any of my past streams!
[https://www.youtube.com/@coreyja](https://www.youtube.com/@coreyja)

I'm also going to add a list of all my old streams to my blog! That might be after I finish uploading the backlog, but it might sneak out before that.

### Background Removal

As some of the backlog videos make their way to YouTube I was happily surprised that one specific video was getting a good bit of attention this morning!

[Rust Programming: Removing Background from Videos Using AI | Coreyja Live Stream](https://www.youtube.com/watch?v=XF_25BPWlIQ)

And like I mentioned in a previous newsletter, there is a new technique for this that I've been meaning to try! So, I think we'll have to give that a shot on an upcoming stream!
Maybe even this Sunday! Tune in on [Twitch](https://www.twitch.tv/coreyja) this Sunday at Noon Eastern to find out!

## Corey's Ramblings

### Server Side vs Client Side Rendering

I was hanging out with a friend this week, and the topic of client side vs server side rendering came up. And they asked if I thought one was "faster" than the other, and I want to expand on my answer for you all this week!

First I want to give a quick overview of server side and client side rendering and how the approaches differ.

And to do that we have to talk about browsers a bit. Modern browsers are engineering marvels, but at their core they take an HTML file (along with some CSS styles) and render a page for viewers. The structure and content of the HTML document decide what the end result looks like.

Where that HTML is _generated_ or rendered, is the difference in Server Side and Client Side rendering.

In the "old school" Server Side rendering, your browser will make a request to the server, which will respond with the full HTML page for the browser to render. The HTML already has all the content and structure the browser needs to render it.

And in Client Side rendering, the browser requests a page from the server. But instead of returning the full HTML for that specific page, we instead return a static HTML "shell" that loads some Javascript. When the browser loads this HTML file, the Javascript runs and creates the HTML (or DOM) structure right there in the clients browser. You do still need to send the data to the client. Maybe that's bundled in the Javascript, or maybe it's fetched with API calls. But importantly we don't send full HTML down to the client, besides that initial shell.

Of course there are hybrid approaches. For example the server can send HTML 'fragments' of the page that the frontend inserts into the correct 'slots'.

So which is "faster"? Assuming that we are talking about faster to load a page for the User.

And I'm going to take the boring predictable answer off, it depends.

I don't think about it as much in terms of which is raw "faster", I try to frame it around "who" is doing the computation.

Someone has to assemble the HTML structure. HTML is what the browser understands, so that's what we are going to produce. So it's not so much that one can be significantly faster in all cases, they both have to do the same work. The big difference though is _who_ has to do the work! But there are some clever optimizations to think about.

In a server rendered app, the server needs to produce the full HTML, while all the browser needs to do is render the HTML. This is easier on the client and more intensive on the server.

So if you have an underpowered server, maybe offloading some of the work to your clients can improve the overall experience.

On the other hand, if your clients are running older or underpowered devices, it might be worth it for your beefy server to do the bit of extra work.

There is an important optimization that we should talk about though; caching!

In the above examples we assumed that for each request we needed to generate the HTML. But if we can avoid rendering the HTML all together we might be able to get a real speedup without extra computation. And that's where caching comes in!

Unfortunately it's not a silver bullet. But in cases where the HTML of individual pages doesn't change from User to User, we may be able to cache the result! There are lots of ways to implement this, from caching CDNs to pre-rendering the content and serving it as static. But for now we can just think about it as the server render the HTML the first time a page is requested and then storing it away in memory. The next time someone wants to look at that page, we can simply pull the HTML we already generated out of the cache!

Now the gotcha here, is that we need to be sure that the HTML _really_ doesn't change from user to user. Do you have a username or name on the page? Then maybe we can't cache the whole page and can only cache specific sections.

And this type of caching only really makes sense for server side rendered apps. You could cache rendered HTML on the client side, but since it can't be shared with other users it's less useful than in the server side example.

But  whole page caching like this rarely applies to today's web apps with logins. You may be able to have a cache for logged out users, but if the entirety of your app sits behind authentication there is less of a chance caching will save you much.

So which is faster? :shrug:
I think the better question to think about is faster for _who_?
