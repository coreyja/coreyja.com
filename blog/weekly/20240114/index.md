---
title: coreyja weekly - January 12th
author: Corey Alexander
date: 2024-01-12
is_newsletter: true
---

Hey Team, let’s see if we can’t get back to doing these weekly!

## Week Recap

### Three Pillars of a Web App

I don’t think this was technically this week, but I hadn’t mentioned it here so might as well now!

I published a new Blog Post titled “Three Pillars of a Web App” of a web app, where I talk about my ideal web app structure. And the three processes to go into that: Server, worker and cron.

[Check out the full post on my blog! ](https://coreyja.com/posts/three-pillars/)

### Sunday Stream

[Watch the full stream on YouTube here](https://youtu.be/jdjxuU0kSRc)

I did my first stream of the New Year last Sunday the 7th! Was good to get back to streaming more ‘normal’ projects after doing Advent of Code in December and then taking a bit of a break for the Holidays.

On this stream we worked on creating a small “framework” for my Cron and Worker processes in Rust. It’s following the same patterns from my “The Pillars of a Web App” post that I mentioned above.

We broke out a small crate called `cja`, which holds the code for our custom Cron and Job framework!

SideNote: I’m really happy with the `cja` name for that crate! Because it stands for “Cron, Jobs and Axum” and just so happens to perfectly match my initials!

We used this new `cja` framework to add Cron and Jobs to a new app I’m working on called `status`. That app isn’t ready for prime time yet, but it will be an uptime monitoring service that will hopefully grow into a more full features monitoring platform!

## Things I Saw This Week

I’m still experimenting with the format of this blog, and this week I want to try a small section about news or projects that I came across recently that looked interesting!

### `quickwit`

Quickwit is a project that came across my feeds this week, and I immediately knew I was going to have to check it out. They advertise themselves as: “Quickwit is the fastest search engine on cloud storage. It's the perfect fit for observability use cases”

This sounded perfect to me! Back many moons ago now I was working on a small side project to make log data cheaper to store for my scale of projects. It used S3 as a storage backend, and I think that direction is really powerful in terms of controlling cost especially!

And `quickwit` takes the same approach but are MUCH farther along than I ever made it!

It provides a pretty all included DB, specifically tailored to Logs and Traces. I’m pretty excited about it but haven’t tried it out at all yet. I’m thinking I might try to integrate it with `status` up above once that is off the ground a bit.

### `Challenging projects every programmer should try`

https://austinhenley.com/blog/challengingprojects.html

This is a blog post from a couple years back now, and it’s simply a list of “challenging” projects for you to try and experiment with! I’m not sure I agree with the ‘every programmer should try’ bit, but if you are interested in them the challenges do seem fun!

I’m going to keep them in my back pocket for a rainy day when I’m bored. Definitely let me know if you try any out!

## Github Sponsors

If you enjoy this newsletter and my other content, please consider sponsoring me on [Github Sponsors!](https://github.com/sponsors/coreyja)

Every sponsor helps me be able to continue to working on all these projects I love, and creating posts and videos for all of you!

I also want to give all my Sponsors a bit of a bonus for helping support me and my content. Right now if you sponsor me you’ll get access to a super special private channel in my Discord, where you’ll be able to interact with me directly and ask questions or get help!

And in the future I plan on giving Sponsors access to my side projects! `status` that I mentioned earlier might be the first example of this, and I will likely go back and get `caje` ready for users too and add that.

So, sponsor me on Github to have access to all my projects and apps! I haven’t decided on a price point for the “all inclusive” sponsorship yet, but if you sponsor _before_ I get that added you’ll be grandfathered into “all access” for the life of your subscription!

[Sponsor now!](https://github.com/sponsors/coreyja)
