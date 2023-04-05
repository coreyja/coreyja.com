---
title: Dotfiles - Put your home directory under `git`
author: Corey Alexander
date: 2020-12-11
color: red
tags:
  - tooling
  - dotfiles
  - terminal
---

Silicon Valley Season 2 ruined it for me. Richard had worked so hard on Pied Piper, but Gavin is almost able to steal it from him. Why? Cause Richard pulled down the code he had written on his work device and ran the test suite. How could something so trivial almost cost him so much! That season embedded itself in me, and now I try to keep my personal and professional software as separate as can be. And one thing this requires is having a personal machine, on top of the computer provided by my employer.

As anyone who's done development on different machines can speak to, I found it frustrating at first when settings from one device didn't transfer to the other. For a while, I used Google Drive to manage settings for a few different apps, including the text editor I used at the time. But eventually, I started doing more things in my terminal, and I wanted to keep a copy of the configuration files for all my terminal programs and tools, somewhere that I could share between machines. I did what seemed to me to be the norm and created a `dotfiles` repo on Github. And used some tool to symlink or copy between the actual 'live' files in my home directory `~`, and the repo files living in their project directory, something like `~/Projects/dotfiles`. I did this for years and thought of it as a solved problem. That isn't to say this setup didn't have issues. I would often experiment with something and edit the 'live' configuration files for a quicker feedback loop and forget to update the 'project' files. Or once I switched to symlinks, I would add a new file to a directory 'live', and it would never end up in the `git` repo at all. I had half-solutions to both of these issues, though, and I was happy.

One day, I talked to a co-worker, and they told me something that would change my dotfiles forever.

**You can make your home directory a git repo**

This thought immediately struck me. Why should I keep a separate directory with a copy of these files or deal with symlinking across the directories? It made so much sense! With a single set of files, I would eliminate a whole suite of small annoyances I had with my dotfiles.

However, I was worried about this kind of setup. I have my projects nested under my home directory, so would that break things? Was I going to commit these projects in my dotfiles? Was I finally going to learn what git sub-modules were?

Luckily my co-worker also gave me the magic sauce to make this happen, and I've adopted it in my dotfiles ever since! The 'trick' is to change your `.gitignore` file to an 'allow list' instead of a 'block list'. The syntax of `.gitignore` makes this pretty easy!

The `.gitignore` file is processed from top to bottom, so to start this trick, we add `*` as the first line in our `gitignore`. Now, `git` is actually going to ignore every file. We can go back in, and one by one, allow different files and directories with the `!` prefix operator! For example, a very minimalistic `.gitignore` for a dotfiles repo like this might be like the following.

```gitignore
*

!.bash_profile
```

 With this setup, you can easily share your `bash_profile` between machines in your dotfiles repo and be confident this is the only file shared. Only the things you explicitly allow in the `.gitignore` file will be included in the repo! I find this works really well for a dotfiles repo, as I have many more files in my home directory that I want git to ignore, it would be tedious and error-prone to try to block each of them individually.

Luckily having 'nested' git repo's like this doesn't really affect the 'inner' repos at all. By that, I mean that my project repos are none the wiser to my dotfiles setup, and no special config is needed for them. Git tools simply look upwards for the closest `.git` directory, so having one farther up the directory tree isn't an issue.

The one caveat to be aware of if you dive in with me and take this approach is that you are almost _always_ in a git repo. Even if you are simply in a git ignored directory. Does this matter? In practice, I find it really doesn't a ton. My `bash_prompt` always shows me I'm in a git repo, but nothing else is really affected.

I've been doing my dotfiles like this for a while, and I can't imagine going back! Maybe give it a shot and let me know what you think.

If you want a more detailed explanation of how to set this up, keep reading with me, and we'll walk through it together.

## Explanation

The actual how of doing it is slightly more complicated but only as a one-time setup. Let's walk through how you might set up a new dotfiles repo directly in your home directory and share it across multiple machines!

### Creating a repo on your first machine
So to start out, let's assume you have some files in your home directory that you want to put in your dotfiles repo.

Warning: If your dotfiles are symlinks to the 'real' files elsewhere, you will want to replace those symlinks with actual copies of the files before we get started. If you follow these steps with symlinks, git will only pick up the symlink destination and not the actual file contents.

Ok, let's get started! I'm gonna hop into a VM so that we can start fresh together.
I have three files I want to share between machines: `.bash_profile` and `.bash_prompt` for all my 100% necessary bash customizations. And then `.gnupg/gpg-agent.conf`, which contains some setup that I needed to make my Yubikey work. But even in a brand new VM, my home directory is full of way more than that.

```shell
$ pwd
/home/coreyja
$ ls -a
.	    Downloads .sudo_as_admin_successful
..	    .gnupg	 Templates
.bash_history gpg.pub	 .vboxclient-clipboard.pid
.bash_logout  .local	 .vboxclient-display-svga-x11.pid
.bash_prompt  Music	 .vboxclient-draganddrop.pid
.cache	    Pictures  .vboxclient-hostversion.pid
.config    .profile  .vboxclient-seamless.pid
Desktop    Public	 Videos
Documents   .ssh
```

First, we need to create a new git repo. We can just run `git init .`

```shell
$ git init .
Initialized empty Git repository in /home/coreyja/.git/
```

As an optional step, we will also switch to a `main` branch that we will use here as the default branch.

```shell
$ git checkout -b main
Switched to a new branch 'main'
```

Tip: If you have `git` version 2.28.0 or newer, you can do this from your git init with the `--initial-branch` flag. Ex: `git init . --initial-branch=main`

Right now, we have an empty git repo, but it 'wants' to track everything in my home directory. Nothing is tracked because we haven't `git add`ed anything, but the entire home directory is coming up as untracked.

```shell
$ git status
On branch main

No commits yet

Untracked files:
 (use "git add <file>..." to include in what will be committed)
	.bash_history
	.bash_logout
	.bash_prompt
	.cache/
	.config/
	.gnupg/
	.local/
	.profile
	.ssh/
	.sudo_as_admin_successful
	.vboxclient-clipboard.pid
	.vboxclient-display-svga-x11.pid
	.vboxclient-draganddrop.pid
	.vboxclient-seamless.pid
	gpg.pub

nothing added to commit but untracked files present (use "git add" to track)
```

Now, let's add a `.gitignore` file that ignores everything.

```shell
$ echo '*' > .gitignore
$ git status
On branch main

No commits yet

nothing to commit (create/copy files and use "git add" to track)
```

Now, `git status` shows that there is nothing to commit because we ignored everything. But we actually DO want to track the `.gitignore` file we made. That is because `.gitignore` basically becomes the 'manifest' of files to commit to the repo, and as such, does need to be present on each machine. So let's add to our `.gitignore` file to tell it we DO want to track the `.gitignore` file itself, how meta.

```shell
$ echo '!.gitignore' >> .gitignore
$ git status
On branch main

No commits yet

Untracked files:
 (use "git add <file>..." to include in what will be committed)
	.gitignore

nothing added to commit but untracked files present (use "git add" to track)
```

The syntax used was to put a `!` before the file name, which acts like a NOT ignore; tracking the file!

Now that git shows some untracked changes, we can add the `.gitignore` file and commit.

```shell
$ git add --all
$ git commit -m "The bare bones of our dotfiles setup. Currently just a gitignore file that ignores everything besides itself"
[main (root-commit) 664ab8d] The bare bones of our dotfiles setup. Currently just a gitignore file that ignores everything besides itself
 1 file changed, 2 insertions(+)
 create mode 100644 .gitignore
```

And from here, we can just repeat what we did for `.gitignore` and add the rest of the files we want to track! Lets start with the `.bash_profile` and `.bash_prompt`

```shell
$ echo '!.bash_profile' >> .gitignore
$ echo '!.bash_prompt' >> .gitignore
$ git status
On branch main
Changes not staged for commit:
 (use "git add <file>..." to update what will be committed)
 (use "git restore <file>..." to discard changes in working directory)
	modified:  .gitignore

Untracked files:
 (use "git add <file>..." to include in what will be committed)
	.bash_profile
	.bash_prompt

no changes added to commit (use "git add" and/or "git commit -a")
$ git add --all
$ git commit -m "Track the bash_profile and bash_prompt files"
[main fdd42e4] Track the bash_profile and bash_prompt files
 3 files changed, 4 insertions(+)
 create mode 100644 .bash_profile
 create mode 100644 .bash_prompt
```

Now let's do the for the `.gnupg/gpg-agent.conf` file

```shell
$ echo '!.gnupg/gpg-agent.conf' >> .gitignore
$ git status
On branch main
Changes not staged for commit:
 (use "git add <file>..." to update what will be committed)
 (use "git restore <file>..." to discard changes in working directory)
	modified:  .gitignore

no changes added to commit (use "git add" and/or "git commit -a")
```

Hmm, that didn't work quite as well... That's due to how gitignore works on directories. You need to allow BOTH the directory and the file. For things in sub-directories, you need the 'allow' each directory and the final file. Once we ignore the directory, we can go ahead and commit the `gpg-agent.conf` file.

```shell
$ echo '!.gnupg/' >> .gitignore
$ git status
On branch main
Changes not staged for commit:
 (use "git add <file>..." to update what will be committed)
 (use "git restore <file>..." to discard changes in working directory)
	modified:  .gitignore

Untracked files:
 (use "git add <file>..." to include in what will be committed)
	.gnupg/

no changes added to commit (use "git add" and/or "git commit -a")
$ git add -all
$ git status
On branch main
Changes to be committed:
 (use "git restore --staged <file>..." to unstage)
	modified:  .gitignore
	new file:  .gnupg/gpg-agent.conf
$ git commit -m "Add the gpg-agent file as well"
[main e228187] Add the gpg-agent file as well
 2 files changed, 5 insertions(+)
 create mode 100644 .gnupg/gpg-agent.conf

```

After all this, here is my `.gitignore` file (eagle-eyed readers will notice I reordered this, so the `.gnupg/` directory comes before the filename, this is optional, but I like how it looks better personally).

```shell
$ cat .gitignore
*
!.gitignore
!.bash_profile
!.bash_prompt
!.gnupg/
!.gnupg/gpg-agent.conf
```

Now we can push this up to your Git host of choice; I'll use Github here to demo.

```shell
$ git remote add origin git@github.com:coreyja/example-dotfiles.git
$ git push --set-upstream origin main
Enumerating objects: 13, done.
Counting objects: 100% (13/13), done.
Compressing objects: 100% (6/6), done.
Writing objects: 100% (13/13), 1.07 KiB | 1.07 MiB/s, done.
Total 13 (delta 1), reused 0 (delta 0)
remote: Resolving deltas: 100% (1/1), done.
To github.com:coreyja/example-dotfiles.git
 * [new branch]   main -> main
Branch 'main' set up to track remote branch 'main' from 'origin'.
```

And there we go! You can expand on this to add any files you want in your dotfiles repo! You can edit the 'live' files in place and easily commit the results. You don't have to worry about accidentally leaking anything from your home directory since you have to manually allow files and directories.
### Cloning this repo on your second machine
Ok, so now let's move to machine two, and copy these dotfiles down. For this demo, let's also look at what happens when there is a conflict and your second machine's version doesn't match the version we had on our original machine.

On this machine, we have a slightly different bash prompt.

```shell
Machine2$ cat .bash_prompt
export PS1="Machine2$ "
```

If we try to clone the repo we created, we will get an error since we don't have an empty directory.

```shell
Machine2$ git clone git@github.com:coreyja/example-dotfiles.git .
fatal: destination path '.' already exists and is not an empty directory.
```

So what we need to do is create an empty git repo, and then wire up the origin and do a fetch.

```shell
Machine2$ git init
Initialized empty Git repository in /home/coreyja/.git/
Machine2$ git remote add origin git@github.com:coreyja/example-dotfiles.git
Machine2$ git fetch
remote: Enumerating objects: 13, done.
remote: Counting objects: 100% (13/13), done.
remote: Compressing objects: 100% (5/5), done.
remote: Total 13 (delta 1), reused 13 (delta 1), pack-reused 0
Unpacking objects: 100% (13/13), 1.05 KiB | 540.00 KiB/s, done.
From github.com:coreyja/example-dotfiles
 * [new branch]   main    -> origin/main
```

So now we have our remote all set up, let's set up a local branch.

```shell
Machine2$ git checkout -b main
Switched to a new branch 'main'
Machine2$ git reset --mixed origin/main
Unstaged changes after reset:
M	.bash_prompt
D	.gitignore
```

Here we used `git reset` to tell git that our `HEAD` is the same as the origin version, or we want to operate as if we were 'on' that commit. We use `--mixed` so that it keeps our local changes. We can then see if any of our local changes are things we want to keep or remove.
You might also notice it says we 'deleted' the `.gitignore` file. This is because this machine didn't have a `.gitignore`. We want to take this file from the version we already committed on the first machine. If we don't and run `git status`, you will see we aren't yet ignoring the rest of the home directory. So let us go ahead and checkout the origin copy.

```shell
Machine2$ git checkout origin/main -- .gitignore
Machine2$ git status
On branch main
Changes not staged for commit:
 (use "git add <file>..." to update what will be committed)
 (use "git restore <file>..." to discard changes in working directory)
	modified:  .bash_prompt

no changes added to commit (use "git add" and/or "git commit -a")
```

Now we can take a look at the diff between our local version and the origin.

```shell
Machine2$ git diff
diff --git a/.bash_prompt b/.bash_prompt
index 6d41176..fdbe383 100644
--- a/.bash_prompt
+++ b/.bash_prompt
@@ -1 +1 @@
-export PS1="$ "
+export PS1="Machine2$ "
```

In this case, I think I want to throw away both versions and create a new prompt to use on both machines!

```shell
Machine2$ echo 'export PS1="coreyja $ "' > .bash_prompt
Machine2$ source .bash_prompt
coreyja $ git add -all
coreyja $ git commit -m "Get second machine setup and create a more unified prompt"
[main 9449185] Get second machine setup and create a more unified prompt
 1 file changed, 1 insertion(+), 1 deletion(-)
coreyja $ git push --set-upstream origin main
Enumerating objects: 5, done.
Counting objects: 100% (5/5), done.
Compressing objects: 100% (2/2), done.
Writing objects: 100% (3/3), 311 bytes | 311.00 KiB/s, done.
Total 3 (delta 1), reused 0 (delta 0)
remote: Resolving deltas: 100% (1/1), completed with 1 local object.
To github.com:coreyja/example-dotfiles.git
  e228187..9449185 main -> main
Branch 'main' set up to track remote branch 'main' from 'origin'.
```

While we were doing those last steps, we used `git add --all`. When I've talked to people about this workflow before, they get worried that commands like this will accidentally commit more than they want. But as you can see, that isn't the case! `git add` follows the `.gitignore` file, so it will only add the files we explicitly allow.

And there you have it! Now we have a dotfiles repo setup on two machines, but even better, the repo lives directly in your home directory! No more copying or symlinking! Simply edit the files and create a commit, nice and simple and using all the tools you already know and love!

I hope more people give this technique a try and let me know what you think!

