---
title: Introducing Bytes
author: Corey Alexander
date: 2024-09-03
tags:
  - bytes
  - challenges
  - rust
---

## Introduction

Hey Team! 👋

Been awhile; too long really! Life got in the way, but I’m back and ready to do more streams and write more posts.

And today I’m super excited to introduce one of the things that got be back and excited about making content again!

Today I’m launching Byte Challenges, or Bytes for short! These are going to be short ‘games’, designed to test your programming and code review skills. We are launching with a “Bug Hunt” but hope to have different formats in the future.

If you want to skip right to the game you can find the [first Byte here](/bytes/level-0-0)!
Be sure to checkout the leaderboard too, and see how you stack up.

## Background and Concept

These challenges are using a platform one of my friends, [Ali](https://x.com/alialobai1), is working on called, `cookd`!
`cookd` as a platform currently focuses on code review and reviewing diffs. It’s built as an interviewing tool, and it thats interesting to you definitely reach out to Ali, he’d love to give you a demo!
But the interviewing angle isn’t as fun to me personally, I’m here for the games!

I’ve always said that one of the most important skills for a software developer working with a team, is the ability to read code well and provide code review to their teammates. As I’ve become a more senior engineer I’ve seen how good Pull Request reviews can be a tool to teach and mentor others. But the flip side to that is reading code and reviewing others work, is also a skill that needs practice to get better.

So when Ali reached out to show me `cookd` and it’s focus on Pull Request reviews, I knew he was onto something and I wanted to integrate it into my site!

## The Birth of Bytes

Ali explained the platform to me, but it wasn’t until I was solving my first puzzle that I fell in love with the format.

We were on a video call chatting about what each of us had been up to recently, and Ali shared a link and told me to share me screen when I clicked on it. He wanted to see my live reaction to what we’d been cooking up, and I don’t think I disappointed him. The premise of the game was simple, he had taken a bit of open source code me and our mutual friend Seif had been working on and hidden a few bugs in it.

Ali wanted to get my take on the platform, and see what I thought of everything. But I was way too distracted by the puzzle to care about anything else, it was a perfect Nerd Snipe for me. Especially with the bit of gamification thrown in; the counter on the side telling me how many bugs I had left to find and the timer constantly ticking down were all I could pay attention to. I turned on my internal ‘streamer mode’, and started thinking out loud trying to track down the third and final bug that was hidden. I was instantly hooked and after I failed to find the third bug we chatted about how I could start building these on my own and even integrate them into my site!

We talked about the types of bug that I thought were good for this format, and the ones I thought were less fun and more annoying. For example that third bug that I was searching for, was a syntax error with the Rust turbo fish operator. The code read something like `collect::Vec<i32>()` when it should have been `collect::<Vec<i32>>()`, note the missing `<>` around the `Vet<i32>`. While that _was_ a bug in the code, it wasn’t one I would expect a human reviewer to catch. That’s the compilers job after all!

Ali had been working on AI assisted tools to help create these games at scale, but I knew I was more interested in making hard crafted artisanal challenges for all of you, and with the Bytes were born!

## Introducing Level 0-0

Today I am very excited to introduce to you all the first Byte challenge! The first few of these will be in Rust, but I plan to branch out to other languages soon. Reach out and let me know what languages and types of puzzles you are interested in seeing!

The first puzzle is titled `Level 0-0` and I think its a great into to the platform and concept. It is in Rust, but I promise none of the bugs will be syntax errors, and I tried my best to make it pretty language agnostic as well. Even if you aren’t super familiar with Rust I encourage you to give this one a try, I think you’ll be surprised how approachable it is!

I don’t want to spoil too much about this puzzle, you’ll just have to play it for yourself!

You can try your Bug Hunting skills on this first challenge here: <https://coreyja.com/bytes/level-0-0>
And after you’ve given it your best shot, check out the Leaderboard to see how others have done!

## Plans for “Bytes”

The plan is to publish these one a week, on Monday’s. (This one is coming out on Tuesday, because it was a US holiday yesterday and definitely not because I hadn’t written this post yet!)

On Monday’s a new Byte will be released, and I’ll spam it across my social media accounts!

To follow up on Friday’s I’ll post a video ‘wrap-up’ talking about this weeks challenge, and showing the answers. For the one’s my friends write, I’ll do a live attempt at trying to solve it myself as part of the wrap-up!

If you can’t wait till Friday to get the solution, consider sponsoring my work on Github Sponsors and you’ll get private access to the solution video as soon as the challenges come out on Monday.

The leaderboards will based on the honor system. If you’ve watched the solution videos, please don’t go solve the challenge to get a perfect score on the leaderboard.
Don’t be the reason I need to implement safe guards to keep this fun for everyone!

## Call to Action

And this is the part of the blog where I kinda want you to click away.

To try this first challenge click here: <https://coreyja.com/bytes/level-0-0>
Or to view the leaderboards head over here:<https://coreyja.com/bytes/level-0-0/leaderboard>
If you want to sponsor my check out my Github Sponsors Profile: <https://github.com/sponsors/coreyja>
And if you have feedback reach out! You can join my Discord sever and we can chat live: <https://coreyja.club>
Or you can send me an [email](mailto:bytes-feedback@coreyja.com) or send a toot via one of my social media channels. I’ll be most active on Mastodon, but will do my best to respond on all platforms! <https://toot.cat/@coreyja>

And don’t forget to come back for the wrap-up video this Friday! If you want to be notified for each future Byte challenge, consider subscribing to my newsletter and each new challenge will land right in your inbox every Monday morning!

I’m really excited to see what you all think about these challenges! Please do reach out and share your thoughts.

In the meantime, I’m doing to be watching the leaderboard to see how everyone does! Good luck!
