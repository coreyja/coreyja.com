---

title: Introducing Sleet ☁️ ❄️
author: Corey Alexander
date: 2018-01-15
tags:
  - gem
  - sleet
  - circleci
  - rspec
  - ruby
color: blue

---

I am super excited to introduce a new gem I've been working on called [Sleet](https://github.com/coreyja/sleet)! It's still very much in an alpha/beta phase, (currently at version 0.3.1) but currently functional!

## What is Sleet?

Sleet is a tool to speed up your Ruby workflow, if you are using Rspec and CircleCI. It allows you to run a simple command `sleet`, and then run ONLY the examples that failed on your most recent CircleCI build!

This works by downloading the persistance files from the individual CircleCI containers, and combining them so you can have a single persistance file locally.

## Why is it called Sleet?

`Sleet` came about due to thinking about precipitation. Like precipitation it involves taking lots of small things from the clouds and combining them to form bigger things as they fall. I started to like the precipitation analogy so looked at what names were available. There weren't any existing gems named `Sleet` so that is what I went with!

## Why it was created

CI is an amazing tool! But I've always found it inconvient to have to copy and paste which specs failed from the CI environment when I wanted to run them locally. And when I heard about Rspec Persistance Files, I knew there should be a way to marry them together! Uploading the Rspec Persistance File from CI gets you almost all the way there, and if you run CI in a single threaded environment so you end up with one persistance file that may be enough! One of CircleCI's best features, in my opinion, is it's ability to parallelize specs. But this has the unfortunate side effect of creating multiple persistance files, and more often then not I found the failures I wanted spanned multiple files.

I'm also a sucker for automation, and this seemed like a problem that could be sovled! Luckily CircleCI has an API that makes all of the build and artifacts information availible, so I simply had to glue everything together.

## Getting Started

This assumes you already have Rspec persistance files set to upload in CircleCI and set up to use locally. For a guide that covers that too check out the [README](https://github.com/coreyja/sleet#getting-started)!

First install the gem.
(Sleet currently depends on `rugged` for its Git interactions, which requires `cmake` to install. On OSX you can install it with `brew install cmake`)

```
gem install sleet
```

Then simply cd to the directory of your project, and checkout a branch for which tests have run in CircleCI. Then simply run Sleet!

```
sleet
```

If your Rspec Persistance File is named `.rspec_example_statuses` both locally and in Circle, this should be all you need! If you are using Worklfows in CircleCI there is also support for that! Check out the [workflows option](https://github.com/coreyja/sleet#workflows).

I'm definitely gonna be writing more about Sleet soon, but seeing as I wrote most of the README and this post today I'm pretty done with writing for the day. I hope you give Sleet a look! And if you have any questions or comments, either make an issue on GitHub if appropriate, or shoot me an email at [coreyja@gmail.com](mailto:coreyja@gmail.com).
