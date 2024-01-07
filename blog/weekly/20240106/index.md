---
title: coreyja - 2023 Review and 2024 Preview
author: Corey Alexander
date: 2024-01-06
is_newsletter: true
---

Hey Team!

It's the first week of 2024 and I wanted to write something to kick off the New Year!
I'd also like to try and get one of these newsletters out each month, so let's call this the January Edition!

## 2023 In Review

2023 was an awesome year!

### Lavender Iguana

Brandi and I launched Lavender Iguana, our Design and Development business. And through that we've been producing our first podcast, which has been a learning experience and a great time!
If anyone reading this is looking to make a podcast and doesn't know where to start, reach out and I'll connect you with Brandi to get started!

### [caje](https://coreyja.com/projects/caje)

`caje` was one of the most comprehensive projects we tackled this year!
It all started with a comment on an existing video, asking about making a CDN in Rust.

[YouTube Playlist](https://www.youtube.com/playlist?list=PL0FtqJaYsqZ2v0FezJa15ynwBpo7KE8Xa)

And from there we made _eight_ full length streams where we build up `caje` from scratch in Rust!

We work on sitting in the middle between users and the origin servers, and saving any request we see to a file system based cache.
But from there we had to make it a bit more global, after all this is supposed to be a Content Distribution Network, we can't be only in a single region!
So we expanded `caje` to run in multiple regions, and share the state of all files in the cache with SQLite. We are using [`LiteFS`](https://fly.io/docs/litefs/) to make our SQLite file distributed across our `caje` cluster!

### [Moodboard App](https://coreyja.com/projects/moodboards)

At the end of the year we started on this new app that Brandi is helping plan out and design!

The idea is to create an app to help designers and their clients get on the same page for imagery on the site! The goal is to build a "dating-app" like experience where the Client can swipe through a set of photos and choose if they do or do not like them for the aesthetic of their site or brand!

The [YouTube Playlist for the build is here.](https://www.youtube.com/playlist?list=PL0FtqJaYsqZ0V50h8Qt6pDWhKryfFxMsU)

### Advent of Code

And in December this year I started Advent of Code again! I say 'started' because, as is tradition, I only got part of the way through with doing these as streams. You can see my solutions for the first half-ish of December before I ran out of steam [on YouTube](https://www.youtube.com/playlist?list=PL0FtqJaYsqZ2-Bms2mSVvn08bVdAMmp2j).

## Looking forward to 2024

While I'm sure I'll find many small projects to fill my time, there is one big one on the horizon I'm really excited to get started on and share more about!

In 2024 I'm going to be working on my video course currently titled "Writing a Web Framework in Rust"! The goal is to be a course for people who may not know Rust, but are interested in learning both about Rust and seeing how to build a Web framework from the ground up! We are going to only use the Standard Library and build everything up for ourselves. The first chapter starts by opening a TcpStream and looping over the lines until we can parse an HTTP message, I think you all are going to love it!

If you want to get a sneak peek at that course be sure to [Subscribe to my Newsletter](https://coreyja.com/newsletter) and/or [Join my Discord](https://discord.gg/RrXRfJNQJX).

I'll be giving my Sponsors a sneak peek while I record episodes so if you don't want to miss anything, be sure to help support me in making these videos and courses and sign up for my [Github Sponsors](https://github.com/sponsors/coreyja)!
