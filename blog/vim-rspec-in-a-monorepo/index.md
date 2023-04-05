---

title: vim-rspec in a Monorepo
author: Corey Alexander
date: 2018-04-15
color: green
tags:
 - vim
 - vim-rspec
 - monorepo

---

## Monorepos

At work we often use monorepos, where the root of the git repo contains subdirs where each one contains a different project. A single repo could contain multiple Ruby projects, or a single Ruby project and a docs repo for instance. When I work in these monorepos I typically prefer to open VIM at the root level of the git repo. This has the unfortunate side effect of making it hard to interact with [vim-rspec](https://github.com/thoughtbot/vim-rspec)[^1] plugin. vim-rspec tries to run rspec from the git root which is outside of any of the projects, and as such fails.

## Investigating Plugin Configuration

My first thought was maybe there was some settings or configuration options in vim-rpsec that I could expand.

The first option I looked into was the ability to set a custom command to run instead of simply `rspec`. This can be done by a like such as this one in your `.vimrc` [^2]

```vim
let g:rspec_command = "Dispatch rspec {spec}"
```

From here I thought about writing a command that could parse the `{spec}` and `cd` to the correct directory. Sometimes the `spec` will simply be a path, but sometimes it will also contain a `:` and a number at the end to indicate a line number for instance `some/path/to/spec.rb:15`.
It would be possible to separate the line number from the path, but then I realized the beginning of the path would also contain the dir I needed to cd into. This was helpful, but also meant I would have to modify the `{spec}`, as well as just parse it. I decided to look further and see if I could find a simpler solution that lived outside the configuration offered within `vim-rspec`.

## Looking outside `vim-rspec`

After thinking about parsing the path after `vim-rspec` was done with it, also meant redoing some of the work that `vim-rspec` already did in preparing the `{spec}`. It would make more sense to get `vim-rspec` to run as if we were in the project root, instead of the git root. So what I wanted to do was `cd` to the correct directory, run the `vim-rspec` command and then `cd` back to where I started. But I didn't know how to determine where or when to `cd` to a different directory. For instance, sometimes the projects I work on are not in monorepos, so I don't need any of this extra behavior.

One thing that most rspec spec files have in common, is that they are within a project that contain a `Gemfile`. All my monorepos will contain a Gemfile in the root on the project if it is a Ruby project that is using rspec, so this will work for my use-case and I imagine many others.

Since we are operating outside the context of `vim-rspec` we are acting in the context of the current file in VIM. From the file we need to find the nearest `Gemfile` if one exists. Luckily the `findfile` function in VIM will do exactly what I want. From there it was simply a matter of cding to that directory before running `vim-rspec`, and the returning the original directory. I decided to (probably overly 😉) deconstruct into a few functions that I added to my `.vimrc`.

## Solution

Here is an excerpt from my `.vimrc` with the relevant bits of my full solution

```vim
fun! SafeCD(dir)
  execute 'cd' fnameescape(a:dir)
endfun
fun! RunFromDir(dir, function)
  let current_dir = getcwd()
  if !(a:dir ==? '')
    call SafeCD(a:dir)
    call a:function()
    call SafeCD(current_dir)
  else
    call a:function()
  endif
endfun
fun! RunFromGemfileDir(function)
  let gemfile_dir = fnamemodify(findfile("Gemfile"), ':p:h')
  call RunFromDir(gemfile_dir, a:function)
endfun

" RSpec.vim mappings
map <Leader>t :call RunFromGemfileDir(function('RunCurrentSpecFile'))<CR>
map <Leader>s :call RunFromGemfileDir(function('RunNearestSpec'))<CR>
map <Leader>l :call RunFromGemfileDir(function('RunLastSpec'))<CR>
map <Leader>a :call RunFromGemfileDir(function('RunAllSpecs'))<CR>
```

This includes a few fun bits that I didn't know about vimscript before starting on this project. The first was how to modify and work with [paths](http://learnvimscriptthehardway.stevelosh.com/chapters/40.html), and the second was how to pass around vim [functions](http://learnvimscriptthehardway.stevelosh.com/chapters/39.html#functions-as-variables). [Learn Vimscript the Hard Way](http://learnvimscriptthehardway.stevelosh.com/) was an excellent resource for both.


[^1]: The awesome people over at ThoughtBot maintain the [vim-rspec](https://github.com/thoughtbot/vim-rspec) gem, that I really enjoy, which allows you to run your Rspec specs from within VIM!
[^2]: This is the actual rspec command I use to integrate with the [vim-dispatch](https://github.com/tpope/vim-dispatch) plugin
