---
title: coreyja weekly - February 27th
author: Corey Alexander
date: 2024-02-27
is_newsletter: true
---

Hello everyone, and a warm welcome to our new subscribers!

In this edition, we delve into some exciting updates: my refocused plan, a preview of our upcoming logo redesign, and insights into my server-rendered blog's halted migration to Leptos, a Rust WASM-based framework.

## Refocused Plan

Recently I’ve been trying out a few different things! I’ve been doing my project streams like “usual” but I also tried to make a few “standard” length Youtube Videos.

And I don’t think they are _bad_, but they aren’t as great as they could be. And they didn’t get the attention on Youtube my streams do.

And a 10 minute edited video may take me multiple times longer than a single 2-3 hour stream! So I’m going to de-prioritize those videos for a bit, and focus on more streaming and project work.

On that front “Status” is becoming “Up Guardian” and has a brand new logo in the works thanks to my wife Brandi! I might show the  new branding on stream next time, so look there and I’ll also feature it my next newsletter.

## New `coreyja` Logo

On the subject of branding, Brandi has been hard at work refining my personal brand identity, resulting in a sleek, new logo. Below is a sneak peek at one of the variants variants:

Screenshot 2024-02-27 at 9.42.07 AM.png

We also have some 'flat' variations designed for print purposes.

I’m excited to roll these new logos out to my site!

I was waiting for a recent refactor to be done on my blog before I updated the UI and logos, but recently I decided to put that refactor on hold.
So it’s time to re-brand! Look for a mid-week stream where we change out the logos, and refine the UI a bit.

## Keeping `coreyja.com` fully server rendered

I’ve been recently playing around with [Leptos, A cutting-edge Rust framework for the modern web.](https://leptos.dev/), and I do _really_ like it! And I’m going to continue using it for UpGuardian.

It’s isomorphic framework, where your application runs on BOTH the client and the server. It’s pretty cool!
It’s combines Server Side Rendering and Client Side Rendering, and gives you a single “app” that runs on both the server and the client. When someone requests a page for the first time it is rendered on the server and sent to the browser. But if you navigate from one page to the next it makes a request for the data it needs to render the new page, and renders it on the client side. And it does this all with me only writing a single version of my app and components!

But after playing with the migration for a bit, I’ve decided that for my blog, Leptos isn’t quite the right fit.
And the biggest reason is probably that my blog doesn’t _need_ any client side logic, it’s purely static HTML and CSS right now. When I started the migration I decided I _needed_ to have interactive code snippets on my blog and kinda went off the deep end lol. But reigning it back it, I want to think about a different approach to client side interactivity for my site.

The problem I ran into with Leptos, is that _sometimes_ I don’t really _want_ things to render on both the server side _and_ on the client side. My example was about markdown rendering, and then code snippet syntax highlighting. I ran into a few hiccups with my first implementation, but nothing too bad. But when I finished there was a good second or so where the page wasn’t usable cause it was parsing my markdown and doing syntax highlighting. And I _know_ I probably could have optimized this away. I might have been holding Leptos, or one of the “plugin” crates I choose wrong. Or rather I likely definitely was! The point isn’t that I _couldn’t_ solve the issue, it’s that I _had_ to solve it.

See my current site is simple. Server rendered HTML, plain and simple. I avoided having to render on the client side, by not having a client side! Not revolutionary, but sometimes we have to remind ourselves of the basics lol.

I’d spent this week separating out my application. Finding the crates that could and couldn’t run on the client side in WASM, and refactoring my code to separate out the two.

And it turns out that for me, for this app, client side rendering doesn’t make sense. I wasn’t getting any of its benefits and paying for all its costs.

So I’m going to leave the Leptos branch around for awhile, but likely not continue it for this project. I got farther on this migration than on UpGuardian, so I’ll definitely want to reference this code for at least the next week or two.

Long term I want to use my blog to explore a different way to do full stack development. I’m not sure what that is yet, but the idea is “sprinkles of interactivity”. I’ve used [Stimulus](https://stimulus.hotwired.dev/) with [Rails](https://rubyonrails.org/) before, and I like some aspects about it. And there are parts of Leptos that I really liked and will take inspiration from! But for now I don’t have a specific use case for client side logic on my blog yet, so I’ll kick that can down the road a bit and keep brainstorming.

If you have thoughts on Client Side vs Server Side rendering, or sprinkles of WASM I’d love to chat, just reply to this email!
