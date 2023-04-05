---
title: Dotfiles - January 2018
author: Corey Alexander
date: 2018-01-06
tags:
  - dotfiles
  - bash
  - tmux
  - vim
color: orange
---

# History

My dotfiles repo started out as a fork from [some dude](https://github.com/mathiasbynens). The main reason for making the repo was to share my config between my work and home machine. That worked for awhile, I liked having a decent point to start from. Gave me lots of good aliases, most of which I still use! Wasn't a huge fan of having to use a script to copy the actual repo files to my home dir. Was an additional step, and casued me to usually test out things on my 'real' homedir and forget to copy them to my repo :facepalm:
I shamelessy stole the idea of making my actual home dir my Dotfiles from [Nick Lopez](https://github.com/nlopez)! Went from the standard git whitelist, to more of a blacklist format. Starts out with making a gitignore that ignores everything and then tell it to which files you don't want to ignore. Now no more scripts and crap!

Then I decided to ditch Rubymine for VIM! This time I wanted to start from scratch and not start from someone else's config. So I deleted the vim files from the original config and started from scratch. I really enjoyed starting with a lightweight vim. It made me realize what vim could do out of the box and what additional features I wanted. I also got to choose all the key bindings since I started from scratch!

Next came [Powerline](http://powerline.readthedocs.io/en/master/), to which was really fun! But it was a bit hard to setup, and have running. Had lots of potential though! Wrote a nice little python script to hit a personal project I run, it was really easy to hack together and I think I got it working in an afternoon. Some people on the security team at work weren't super happy with my Powerline setup at work :laughing:. Since it ran in a seperate process, and ecspecially once I stared having it ping my personal project. The security software they have installed was sending alerts about my setup on a pretty regular basis. So to stop making their alerts noisy I decided to stop using Powerline. So I had to drop Powerline for [Airline](https://github.com/vim-airline/vim-airline). Its a project with a similiar ambition! The cool part is it's written in 100% vimscript so much easier to set up! Hooked that up with [tmuxline.vim](https://github.com/edkolev/tmuxline.vim) which gives tmux the same them as my vim line. It's not as feature rich, but I think I can write command line apps that function similarly to the python script I was using before! I haven't gotten around to actually doing this, but maybe I'll blog about it when I decide to do it! I considered using Powerline at home and Airline at work, but I decided I'd rather have consistency so switch to using Airline in both envs.

# General Tools

- Bash, I'm not a zsh guy. Plain ol' bash works for me. I experimented with Fish a long time ago but it never stuck. And now I know enough Bash and everything that I don't think switching would make much sense.
- [tmux](https://github.com/tmux/tmux). I was using iTerm for it's tabs and splits and all that, but switched to tmux semi-recently. At the time the reason I switched was really to use Tmuxinator. I really like having different projects that I can set up and reproduce easily. It also lets me easily context switch to a new app/project without losing where I currently am. For example I currently have both my `dotfiles` and `coreyja-blog` sessions open and I can toggle back and forth between the two. Each of those sessions has their own windows and splits and all the jazz. It helps make context switching faster, since I can pick up exactly where I left off!
- vim/[neovim](https://github.com/neovim/neovim). This is one of the more recent additions to my toolbelt! Before I was using RubyMine, which is am amazing IDE (all of Jetbrains stuff is really good in my opinion) but I wanted something more customizable and light weight, so VIM is where I ended up. My and a buddy at work picked it up at around the same time and that helped us both learn new tricks from each other. I'm actually using nvim which is a newer fork of vim, but essentially functions the same.
  - [https://github.com/coreyja/dotfiles/blob/main/.vimrc](https://github.com/coreyja/dotfiles/blob/main/.vimrc)
- [Airline](https://github.com/vim-airline/vim-airline). This very recently replaced Powerline in my setup. It's way lighter weight and easier to set up, but I did lose a few features in the transition.
- [fzf](https://github.com/junegunn/fzf)
  - fzf is a fuzzy finder that I absolutely love! I use it in the command line in Bash as well as in vim. It provides super fast, and amazing accurate fuzzy finding. It's one of my favorite tools that I couldn't live without. I love that it integrates into the other tools I use so well too.

# Ruby

- rbenv - Different projects needs different versions of Ruby and rbenv gets the job done perfectly!
  - There are some rbenv plugins that I also use too to help out:
    - rbenv/ruby-build
      - Download and build new versions of Ruby
    - rbenv/rbenv-default-gems
      - Install a set of gems when you install a new version of ruby
      - [https://github.com/coreyja/dotfiles/blob/main/.rbenv/default-gems](https://github.com/coreyja/dotfiles/blob/main/.rbenv/default-gems)
    - ianheggie/rbenv-binstubs
      - Never have to run `bundle exec` again!
    - tpope/rbenv-ctags
      - Automatically generate ctags files for installed Ruby versions. More on ctags in a bit!
    - rkh/rbenv-update
      - Automatically update rbenv plugins when `rbenv update` is run

# Ctags

Ctags are the reason I was able to leave RubyMine for vim! I LOVE the "Jump to Implementation" feature in RubyMine and ctags basically replicate that for me in VIM! I will admit they aren't _quite_ as good as I remember the "Jump to Implementation" feature being but its good enough for me! So basically ctags work by creating `tags` files which tell it where all the different classes and methods are defined. Once you have these made for your project (and your dependencies) vim can search through them to quickly jump to the definitions of functions. But getting all your ctags generated can be slightly challenging! I pretty much followed [this amazing guide](http://tbaggery.com/2011/08/08/effortless-ctags-with-git.html) by [Tim Pope (tpope)](https://github.com/tpope), so I won't go into that too much!

There is one thing I have started doing different than that guide is started using [ripper-tags](https://github.com/tmm1/ripper-tags) instead of ctags for my Ruby projects, which is most of my projects at the moment. It provides better indexing for Ruby files since it is specialized just for Ruby. I modified tpope's `ctags` bash script to use ripper tags for Ruby files. [Here](https://github.com/coreyja/dotfiles/blob/main/.git_template/hooks/ctags) is my modified version of that script.

I am also using [gem-ripper-tags](https://github.com/lzap/gem-ripper-tags) to automatically generate ctags using [ripper-tags](https://github.com/tmm1/ripper-tags) for all my installed gems, this is on my list of default gems so that it gets installed for each version of Ruby automatically.

To really take advantage of ctags in VIM I paried them with fzf, for that fuzzy finding goodness! Here is my remap so that I can hit Control-P and get a fzf window off all the tags that match what is under my cursor, and then jump to the one I pick. This is the most similiar to Rubymines, Command-Click behavior.

# Wrapping up

There is obviously alot more in my dotfiles than I outlined here! There are probably some more things in there I want to write more posts about eventually. One in particular is my setup with GPG and my Yubikey for Github commit signing, and SSH auth. But like I said, we'll save that for another post!

Thanks for making it all the way to the end! This is my first post of 2018, but I'm going to try to make it more of a habit. I'm gonna try to get 1 posted every week this year! We'll see how it goes!

Edit (2017-01-07): Today I found [gem-ripper-tags](https://github.com/lzap/gem-ripper-tags) which I am now using instead of [gem-ctags](https://github.com/tpope/gem-ctags).
