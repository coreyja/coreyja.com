---
title:  The Making of COREYJA
author: Corey Alexander
date:   2017-02-18
tags:
    - making-of
    - middleman
    - ruby
color: purple
---

Hey, thanks for checking out my blog! Got it all set up, so decided my first post should be a making of post showing how I put this blog together! If you want to jump right to the source, here is the github: [github.com/coreyja/coreyja-blog](https://github.com/coreyja/coreyja-blog).

## Requirements

There are some things I wanted whatever engine I picked to be able to support.

- Must support articles written in Markdown
  - I really enjoy writing in Markdown, and that it translates to HTML
- Syntax Highlighting
  - Markdown supports code blocks and I wanted them to be highlighted correctly
- Generate a Static Site
  - I’ve run Wordpress blogs in the past, but I didn’t want to maintain a dynamically generated blog like that
- Written in Ruby, and using the ERB tempting engine
  - Just a personal preference, but I enjoy being able to extend my tools and I’m currently working in Ruby and like the ERB syntax

## Middleman Blog

With these requirements I found the `middleman` ([github.com/middleman/middleman](https://github.com/middleman/middleman)) gem! And some extensions to it, including `middleman-blog` ([github.com/middleman/middleman-blog](https://github.com/middleman/middleman-blog)), and  `middleman-syntax` ([github.com/middleman/middleman-syntax](https://github.com/middleman/middleman-syntax)).

Middleman describes itself as a static site generator, and is exactly what I was looking for. Coming from a Rails background I was right at home in the Middleman environment. Getting started was as easy as running the blog generator, `middleman init coreyja-blog --template=blog` and editing some ERB files.

Middleman also provides a Rack compatible `config.ru` file, which made developing using [POW](http://pow.cx/) a breeze. Simply linked the project to POW, and was able to view the site locally and have changes appear when I reloaded. I didn’t enable live-reload, but there is an extension for that as well.

## Design

The design of this site was done by Phillip Inge at [philinge.com](http://philinge.com/). We worked together years ago as freelancers and I had him design me this blog when we were working together. I think it has definitely stood the test of time, and still looks great a few years later! I originally used the designs for django based Portfolio site that I made at the time, but I am going to be replacing that with this blog. I made some small tweaks to the designs this time around to make them responsive.

## Syntax Highlighting

Syntax Highlighting was one of my requirements for this project. Setting it up with [middleman-syntax](https://github.com/middleman/middleman-syntax) was fairly simple, but I wanted to take it one step further! I realized that they were using [Rogue](https://github.com/jneen/rouge) under the hood, and themes could be defined in Ruby. So I made a Syntax Highlighting theme that fit the design and color scheme of this site. For the most part I based it off if the `Monokai` theme, but changed the accent colors to better fit my color scheme. I am pretty happy with the results! It’s not something I’d want to use for my daily editor, but I think it looks pretty good on the site.

## Hosting

Since Middleman is a static site generator hosting is easier and cheaper! I simply upload the generated files to AWS S3 and configure it for a static site! This is, understandably, much cheaper than some of the Heroku personal projects I also run.
