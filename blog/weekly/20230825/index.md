---
title: August 25th Update - Structural vs Nominal Types
author: Corey Alexander
date: 2023-08-25
is_newsletter: true
---

Hey Team! Hope everyone is having good week.

My ramble this week is going to be my initial thoughts on a blog post I want to write. Its working title is "Structural vs Nominal Typing: Comparing TypeScript and Rust".
But first let's catch up on my week!

## This Week

### Byte Speaks!

One of the projects we've been working on during stream is turning Byte into a full fledge chatbot. I want to be able to talk to him during stream, and we want you all to be able to chat with them in Twitch Chat.
And on stream on Wednesday we got a version of Byte that can respond to my questions in something approaching real time! The latency is definitely something we'll work on, but it works end to end now!

The process currently is that we fill up buffers of data to feed into Whisper (via [whisper-cpp](https://whisper.ggerganov.com/)) to transcribe.
And we search through those transcripts for the keywords "Byte" or "Bite". When we find the keywords we grab a bit of extra transcription and pipe that plus our custom prompt into OpenAI GPT.
We wrapped all this up on stream on Wednesday, and I gotta say; It was really cool to see it working end to end. I was asking questions on stream and getting responses read aloud back to me!
If you want to skip to the action, [here is the end of stream to check out](https://youtu.be/BUZ9ien9r2U?si=DTqLpczxi30eCp-1&t=5913)!

There is a bit of latency and CPU usage. This is because I'm transcribing everything I say, just to listen for the keywords. What I think we'll try next is to use keyword detection as the audio level, and only transcribe when the keyword is found. I found [Rustpotter](https://github.com/GiviMAD/rustpotter) which looks to be exactly what we are looking for! So stay tuned for trying this out on stream next time. You can find my streams live at [twitch.tv/coreyja](https://twitch.tv/coreyja) and recorded at [youtube.com/@coreyja](https://youtube.com/@coreyja)

## Streams

### August 20th - [Exploring Rust: Crafting a CDN from Scratch - Part 1 | Live Coding Session](https://youtu.be/4DKm8lEYQ6o)

On last Sundays stream we started on a new project! We are going to build a CDN in Rust. For this stream we working on getting a caching proxy setup. There is lots to add like proper checking of cache-control headers, deploying to multiple regions and pushing updates to other nodes!

### August 23rd - [BYTE SPEAKS!! | Live Coding with coreyja | Voice Transcription and Bot Programming in Rust](https://youtu.be/BUZ9ien9r2U)

This is Wednesday's stream, and it's where we got our Byte bot really off the ground! We'd started this project before, but removed a bit of it as we got started and went a different direction. Before we were trying the experimental [candle](https://github.com/huggingface/candle) ML framework from huggingface. This was just lower level than we needed, so switching to [whisper-rs](https://github.com/tazz4843/whisper-rs) (which is powered by [whisper-cpp](https://github.com/ggerganov/whisper.cpp)) got us quicker transcription speeds!

With that in place, we were able to start transcribing in chunks and searching for our keyword. This is very barebones right now, but surprisingly effective!

Can't wait for more streams where we can work on Byte. Next up is probably getting them wired up to also respond to Twitch chats!

## coreyja's Ramblings

### Structural vs Nominal Types

This is a concept I was introduced to recently by a coworker, and it really helped give me a good framework to think about how to compare Typescript and Rust.

Typescript is a structurally typed language, which broadly means the types are defined by the structure for the data.
Rust on the other hand, is nominally typed. Every type I declare is unique even if they have the same value.

In Typescript we can pass an object to a function as long as it has _at least_ all the attributes we need. It's free to have more!
In Rust it's not about the _data_ it's about the specific type. Even if two structs have the exact same fields, they can't be used for each other.

I want to expand on this into a fuller blog post!
So let me know what you think!

Come stop by the Discord, I'd love to chat!
[https://discord.gg/RrXRfJNQJX](https://discord.gg/RrXRfJNQJX)
