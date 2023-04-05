---
title: VIM Spelling Suggestions with fzf
author: Corey Alexander
date: 2018-11-10
tags:
  - vim
  - fzf
  - spell-check
color: orange
---

## TlDr

 Use fzf to show VIM spelling suggestions, and override the built in `z=` shortcut

```vim
function! FzfSpellSink(word)
  exe 'normal! "_ciw'.a:word
endfunction
function! FzfSpell()
  let suggestions = spellsuggest(expand("<cword>"))
  return fzf#run({'source': suggestions, 'sink': function("FzfSpellSink"), 'down': 10 })
endfunction
nnoremap z= :call FzfSpell()<CR>
```

## Background

Recently I was looking to add spell-checking to VIM and came across this great ThoughtBot article, [Vim Spell-Checking](https://robots.thoughtbot.com/vim-spell-checking) that got me started.
This article showed me how to turn on spellchecking, and also to turn it on for specific files. Here are the snippets from the `.vimrc` that are directly inspired from this blog post.

```vim
autocmd BufRead,BufNewFile *.md setlocal spell spelllang=en_us
autocmd FileType gitcommit setlocal spell spelllang=en_us
set complete+=kspell
```

## The Problem

This served me well for quite awhile, but when I ran across misspelled words, it often left me looking for a better way to replace them.

VIM has a built in solution to this in the form of the `z=` keyboard shortcut. This takes the current word under cursor and shows spelling suggestions, where selecting a selection will replace the current word. This is the functionality I wanted to keep, but I didn't love the interface that the suggestions appeared in. They take up the entire VIM screen and force you to pick by entering the number corresponding to the word you would like. I am a big fan of [fzf](https://github.com/junegunn/fzf), and I wanted to use this for spelling suggestions!

## The Solution

The first thing I needed was a list of the spelling suggestions for the word under the current cursor. Getting the current word is simple enough with `expand('<cword>')` so now I just needed to get the spelling suggestions for it.
After a bit of digging [^1] I found the VIM function `spellsuggest`. This function takes the word we want suggestions for as its first arguments. It also takes an optional second and third argument, which we are not currently using. The second argument is the number of suggestions to return, the default is 25. The third argument is a flag for whether we should filter to only capitalized words.

```vim
spellsuggest(expand('<cword>'))
```

So I could now use the above command to get a VIM list of spelling suggestions. Next step was to get fzf to let me select an option from this list.

The fzf repo has a [readme](https://github.com/junegunn/fzf/blob/master/README-VIM.md#fzfrun) that details how to use fzf in VIM. I was mostly interested in how to use the `fzf#run` function, which is the main function for calling into fzf. This can take a VIM list as input, so it fits really nicely with the list of spelling suggestions we already generated. We pass this is as the `source` to `fzf#run`. The other important option is `sink` which tells fzf what to do after we have selected a suggestion. Now its time to replace the word under the cursor with our suggestion! One of the accepted types for `sink` is a VIM function reference so I needed another function to call as a callback, which will responsible for actually replacing the word under the cursor. So far we have the following

```vim
function! FzfSpell()
  let suggestions = spellsuggest(expand("<cword>"))
  return fzf#run({'source': suggestions, 'sink': function("FzfSpellSink"), 'down': 10 })
endfunction
```

For this we can use the VIM command `ciw` to change the current word to what we selected. To execute that from vimscript we use `exe` and `normal!` giving us the following. This uses the normal mode `ciw` and sends the old value to the black hole register `"_`, it then inserts the new word.

```vim
function! FzfSpellSink(word)
  exe 'normal! "_ciw'.a:word
endfunction
```

The last thing we need is a keyboard shortcut, so I can access this quickly. I want to use this instead of the default `z=` behavior so I decided to just remap that shortcut

```vim
nnoremap z= :call FzfSpell()<CR>
```

And finally putting it all together we have... :drumroll:

```vim
function! FzfSpellSink(word)
  exe 'normal! "_ciw'.a:word
endfunction
function! FzfSpell()
  let suggestions = spellsuggest(expand("<cword>"))
  return fzf#run({'source': suggestions, 'sink': function("FzfSpellSink"), 'down': 10 })
endfunction
nnoremap z= :call FzfSpell()<CR>
```

[^1]: I eventually found this function by digging into the source code of the [kopischke/unite-spell-suggest](https://github.com/kopischke/unite-spell-suggest/blob/master/autoload/unite/sources/spell_suggest.vim) repo. This plugin did a similar thing with Unite as the fuzzy finding tool.
