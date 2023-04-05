---
title: Setting up new Laptop and Upgrading to Homebrew 2
author: Corey Alexander
date: 2019-04-19
tags:
  - homebrew
  - macos
color: purple
---

This blog post is gonna be a walk-through of setting up my new laptop, which led to [this PR](https://github.com/coreyja/dotfiles/pull/5/files) from my dotfiles repo. There were a few things I needed to update to get the laptop running. The biggest one was upgrading from Homebrew 1.x to Homebrew 2.0

## Setting up the new machine

Overall I was really happy with how painless the migration to a new machine was, thanks in large part to Homebrew and my `dotfiles` repo. With these two I was able to install everything I needed and have most of my configuration already working without any additional work.

I didn't record the exact commands I ran but it was something like this

```bash
cd ~
git init .
git add remote origin https://github.com/coreyja/dotfiles.git
git fetch origin
git checkout --track origin/main
/usr/bin/ruby -e "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install)"
brew bundle
```

What this does is checkout my `dotfiles` repo in the home dir. This isn't how some people do their dotfiles, where they store the version controlled files somewhere else and copy or symlink then into their homedir. Instead I keep my actual home directory under version control, but have the `.gitignore` set up as a white-list, where I have to specifically add files to be tracked. This works really well for me since I don't have to maintain two different directories. I wrote a bit more about that [setup here.](blog/2018/01/06/dotfiles-december-2018.html)

My first `brew bundle` didn't work, since there were some upgrades I needed to make! The rest of this post will be about the changes I had to make to get up and running! Overall I felt like I only had to make a few small changes and I was up and running really quickly!

## Homebrew 2

In February of this year, the Homebrew community [released Homebrew 2.0](https://brew.sh/2019/02/02/homebrew-2.0.0/) 🎉 🎉

First I want to say thanks to the whole Homebrew community for all the work they do! Getting out 2.0 is an awesome milestone!

### Removal of Options

There were a few changes in Brew 2 that affected my dotfiles, and setup.

The first was the removal of options for all formulae in `Homebrew/homebrew-core`. This is the default tap and as such many a few options I was using were no longer supported. There were a few options I was using that I simply removed, and I don't think I was actively using them. This included the `--with-webp` option for `imagemagick` and `--with-iri` for `wget`.

However there was one class of options that I did still want to support! And that is the `--with-default-names` option that I was using on a few GNU utils, including `coreutils`, `grep` and `sed`. This option made it so I could use the Homebrew installed versions of these utilities without using them with their `g` prefix. So for example I wanted to be able to use `sed` not `gsed` to use the GNU `sed` tool.

However it is relatively painless to accomplish this! I did it by adding the follow code snippets to my `.bash_profile`. These came directly from the `CAVEATS` section of each of the Brew installs

```bash
# Add GnuCoreUtils to the Path
export PATH="/usr/local/opt/coreutils/libexec/gnubin:$PATH"
export MANPATH="/usr/local/opt/coreutils/libexec/gnubin:$MANPATH"
# Add grep to the Path
export PATH="/usr/local/opt/grep/libexec/gnubin:$PATH"
export MANPATH="/usr/local/opt/grep/libexec/gnubin:$MANPATH"
# Add sed to the Path
export PATH="/usr/local/opt/gnu-sed/libexec/gnubin:$PATH"
export MANPATH="/usr/local/opt/coreutils/libexec/gnuman:$MANPATH"
```

This actually does one more thing besides just add these utilities to the PATH. It also sets the `MANPATH` so that you can do `man sed` and get the correct documentation for the GNU `sed` tool

## Other Brewfile Changes

I also made a few additional changes to my Brewfile not related to the upgrade.

I removed a few things that had trouble installing and I didn't use anymore including `svg-term` and `heroku`.

I also removed `gpg-agent` since it is bundled with `gpg` now.

I added `go` and `rustup` since my setup depends on some go and Rust utilites. I've had these on my machine previously but missed adding them to my dotfiles. I also realized that some of the GUI tools I wanted could be installed via `cask` so I added a few of those as well

## Added a base `.ruby-version` file

I was actually surprised I hadn't already done this when I setup the new laptop. But I wasn't keeping the `.ruby-version` file in my homedir under `git`, so I changed that by adding the latest currently available version of ruby as my default `.ruby-version`. I will still have projects that depend on different versions of Ruby, and have their own `.ruby-version` file. But this will give me a good default version, that is consistent between machines.
