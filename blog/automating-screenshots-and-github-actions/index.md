---
title: Automating Screenshots and Github Actions
author: Corey Alexander
date: 2024-09-18
tags:
  - github-actions
  - oidc
  - automating
---

### TLDR

While tinkering at my family's lake house, I discovered my automated screenshots were broken. Instead of using a Personal Access Token (blegh), I roped in a friend for some "research" that turned into launching a whole new crate. Now I've got a custom OIDC server handling my GitHub automations, and I can say goodbye to PAT headaches!

## Introduction

I’ve been using [`shot-scraper`](https://github.com/simonw/shot-scraper?tab=readme-ov-file#shot-scraper) by [Simin Willson](https://simonwillison.net/) to take screenshots of my blog everyday, and commit them back to my blogs git repo. This makes it easy to check what my site looks like currently on a few screen sizes, and gives me a way to see how my blog has changed over time!

For example [here is a quick video](https://coreyja.com/til/video-from-screenshot-history) I made about a year ago showing how my Blog had changed since I launched its latest iteration.

This is all setup using an [scheduled Github Action](https://github.com/coreyja/coreyja.com/blob/3b913cb09fa1a2ad44fe47b3a284a78c25b13fdb/.github/workflows/shots.yml). Every day it would dutifully kick off and take screenshots and commit them back to the main branch.

This was working great! Until, all of a sudden, it wasn’t :sadnerd:

## The Problem

Last week me and my wife drove up to “The Cottage”, my families lake house get away. And while she was relaxing and enjoying the views, I cracked open my laptop get some blog work done.

And that’s when I noticed it for the first time.

The dreaded Red X of Github Actions failing.

![Screenshot from my repo’s Github Action, which is failing and displaying with the big red X. There is a single job in the workflow called `shot-scraper` which is failing, and reporting an exit code of 1](./red-x.png "Red X of Github Actions")

Looking into this more, it turned out to be my own fault. My blog is mostly a ‘single player’ repo; it’s almost always my fault when things break.

At least this time the error message made it clear what was going on.

![Error message screenshot from the Logs of the previous Actions. The relevant error line is `Changes must be made through a pull request.`](./error-message.png "Github Actions Error Message")

Ahh!

I had recently setup a [`Ruleset`](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-rulesets/about-rulesets) on this repo.

Rulesets are the new version of [`Branch Protection Rules`](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-protected-branches/managing-a-branch-protection-rule)

Both of which allow you to setup rules for when Pull Requests can be merged into the Repo. I use them to make sure that tests pass, and PRs have reviews before they get merged into main.

In single player repos like my blog I set these up as a matter of best practices. I have Admin rights to the repo, meaning I can override the rules no matter what. But in multi-player repos, like all the ones at work, these protection rules can be great for aligning the whole team on PR practices and standards!

Aside: The new Rulesets fix one of my longest standing gripes with Branch Protection Rules. It’s now possible to stop Admins from pushing to `main` while continuing to let them force merge PRs when needed! :tada:

But anyways, back to our story.

I had recently setup Rulesets on my blog, which made sure that all changes went through Pull Requests AND that all PRs had at least one approving review.

And these rules were breaking my automated screenshots! Before my Github Action had no problem pushing directly to `main`, but now all we got was that red x.

## First Attempted Solution: Automating Pull Requests

But I knew how to fix that! Let’s have our Action make a pull request!

With the Github CLI it was really pretty easy. I’ll save you the YML but the cli commands for that looked like this

```bash
branch_name="update-screenshots-${current_date}"
git checkout -b $branch_name
timestamp=$(date -u)
git commit -m "Screenshots Updated at ${timestamp}" || exit 0
git push origin $branch_name

gh pr create --title "Update Daily Screenshot ${current_date}" --body "Automated PR to update screenshots" --base main
gh pr merge --auto --squash
```

Breaking this down, we have two sections. One about getting the commits and branch into Github. And a second set of commands to open a PR and mark it to auto merge.

For the first set we use standard `git` cli commands.

For the second set we are using `gh`, [Github’s CLI tool](https://cli.github.com/).
Here we tell Github to open a PR for us, and base it off the `main` branch. Then we request that the PR auto-merges once all the tests pass, and specify that we want to use ‘Squash and Merge’ instead of making a merge commit.

With this I kicked off a run of the Screenshot workflow, from the action UI and gave it a minute to spin up its Chromium instance and take my screenshots.

And after about a minute I had my PR!

But something was wrong...
The tests weren’t running. I gave it a couple minutes to make sure I wasn’t being impatient. But the Github Actions I have setup for PRs never kicked off.

The Pull Request was marked to auto merge, but only once all the requirements were satisfied. And that meant I needed those tests to run.

After internet sleuthing for a bit I found the problem. Github doesn’t run actions on Pull Requests that were opened by a Github Action. Or more specifically, when you are using the Github Token that is supplied to Github Actions it won’t trigger the actions on any PRs you open.

Now from Github’s point of view I can see this making sense. They are trying to prevent a run-away action making a PR which kicks off more actions, makes a new PR and starts an infinite loop leaving a trail of PRs in its wake.

But from my point of view, this was an annoying hurdle to get over.

## Exploring Alternatives

There was however an easy workaround! I could use a Github Personal Access Token instead of the Github token that was automatically given to my actions. This would make it look like the PR was coming from my user account, and the actions would run fine!

But I’ve been burned by Personal Access Tokens (abbreviated PATs from here on out) before. Recently, for a different part of my blog, I ran into an issue where my PAT expired and my site silently stopped being able to fetch my Github Sponsor list. The silently part is due to bad error handling on my end, but I digress.

Adding a PAT to the Github Action was something I wanted to avoid. Besides the expiration the PAT would have access to more than this repo, and wasn’t sparking joy to me. And remember this is a personal project, with no purpose beyond my own enjoyment, so being happy with the result is important!

Another option I quickly discarded, was to wire up my Actions different and kick off the tests with `workflow_dispatch` (or maybe even `workflow_call`). This didn’t sound ideal either, cause I wasn’t looking to adjust how my tests ran, and it meant coupling this Action to my test runs. If I added another workflow that ran on PRs, I’d have to remember to wire up the dispatch for that workflow too. No fun.

The PAT approach was looking better than the dispatch version, but I wanted to avoid putting secrets into my Github Actions.

And that’s when I dawned on me. I knew a system that allowed Github Actions to ‘authenticate’ with third parties without issues explicit secrets, OpenID Connect or OIDC!

## Switching to OpenID Connect (OIDC)

I’d used OIDC before to simplify deploying apps from GitHub Actions to AWS. Instead of needing to put IAM credentials into GitHub Actions, and remember to rotate them, I could use OIDC to fetch a temporary AWS token with the permissions I needed! This required telling AWS to let the Github Action “become” a certain role I had defined in AWS, but allowed me to avoid adding any secrets to my GitHub Actions. GitHub would sign the OIDC token, and give AWS the context of where the request came from and AWS could verify this key and confirm the specific repo and action was allowed to use the AWS Role.

So the question was if I could use OIDC to talk to a custom server, which could make and merge the Pull Requests for me! This custom server would of course still need to have permission to make the Pull Requests and merge them, but I could create a GitHub App and centralize its secrets to this one custom server. Now anytime I had a project that needed to interface with the GitHub API, and needed ‘elevated’ permissions I could use my new “Workflow Automation” GitHub app.

And OIDC is the glue that makes it so each repo I want to automated didn’t need its own set of credentials to manage! And as it turns out, OIDC the protocol was easier than I feared it might be.

I’d been working with a friend, [Seif](https://github.com/Seif-Mamdouh), and I asked him if he thought we could write our own OIDC implementing server, hoping he might do a bit of research for me. But instead, he handed me a brand new Rust crate, capable of verifying an OIDC token ready to be plugged into a custom server, [github-oidc](https://github.com/the-cafe/github-oidc).

OIDC works by GitHub generating a JSON Web Token, and signing it with one of their Private Keys and giving everyone access to the Public Keys, so that we can verify that these tokens came from GitHub.
The JSON Web Tokens contain all the standard JWT bits like expiration timestamps, and issuer information. They also [contain a whole slew of information](https://docs.github.com/en/actions/security-for-github-actions/security-hardening-your-deployments/about-security-hardening-with-openid-connect#understanding-the-oidc-token) about the Repo and Action that initiated the OIDC request!

All of this info can be used by my custom server to make sure that only my repos can interact with my OIDC server. This prevents anyone on GitHub from being able to open a pull request to my blog and have it automatically be merged to main.

`GitHub-OIDC` the crate, helps make this process even simpler by automating fetching all of GitHub’s Public Keys and verifying the token was signed by one of them. I didn’t have to find the URL where GitHub serves its [JWKs (JSON Web Keys)](https://token.actions.githubusercontent.com/.well-known/jwks), and I didn’t have to futz with verifying the JWT manually. I’m able to specify an Organization and Repo that is permitted to send requests to my server, and the crate returns we the full decoded body of the token for permitted requests, and an error explaining what failed to verify for invalid tokens! I was able to focus on the business logic of my custom server, and not about the authentication pieces! This is _exactly_ what I was shooting for.

## Final Solution Plan

If you’d rather watch me build the project from scratch checkout the [stream recording for this project here](https://youtu.be/jbIVJIQOJ9I)

Let’s quickly summarize the final solution that I settled on!

I have a [Github Action](https://github.com/coreyja/coreyja.com/blob/main/.github/workflows/shots.yml) running in my [blog repo](https://github.com/coreyja/coreyja.com). This is scheduled to run daily.
It uses the [`shot-scraper`](https://github.com/simonw/shot-scraper) tool to make screenshots of various pages of my site, at a few different resolutions. It cleans up the PNGs with [`oxipng`](https://github.com/shssoichiro/oxipng), and then creates a new branch commits the changes (if there are any) and pushes it’s new branch to Github.

This was almost identical to what I was doing before we started this journey.

But now instead of pushing directly to `main` it creates a new branch, such as `update-screenshot-2024-09-18`.

And now for the OIDC portion!

In the Github Action, we generate an OIDC token using [a touch of Javascript and Github’s official Actions](https://github.com/coreyja/coreyja.com/blob/main/.github/workflows/shots.yml#L67-L79).

We then use a `curl` command to `POST` this token and other info about the PR to open, including the PR Title and Body to my custom [`workflow_automation`](https://github.com/coreyja/workflow-automation) server!

This custom server uses [`axum`](https://github.com/tokio-rs/axum) to listen for POST requests. When we get a request we first decode and validate the JWT, using the [`github-oidc`](https://crates.io/crates/github-oidc) crate.

After we’ve validated that the action came from my blog repo, we proceed to create the PR, and mark it to auto-merge when all the tests pass! To access the Github API I created a private GitHub app, and installed it for my blog repo. Now `workflow-automation` can make GitHub API calls for my repo, but not act as my User Account. All requests come authorized as the GitHub app!

The API calls to create the PR and set it to auto-merge weren’t as straightforward as I had hoped. If you check out the stream you’ll see that I spent most of my time getting those API calls right, and almost no time on the actual OIDC validation. It’s funny how that happens!

One reason the API calls were more complicated is that there is no REST API to mark a PR to auto merge, which means I had to use the GraphQL api for that operation. I’ve used GraphQL client’s before but this time we just stuffed a GraphQL query into a string and POST’ed to it GitHub manually. Not as nice of a developer experience, but hey, it’s working!
You can checkout all the code to [interact with GitHub here](https://github.com/coreyja/workflow-automation/blob/main/src/github.rs#L102).

And with that we have a working solution! And my automated screenshots are back in action!

[Here is an example PR](https://github.com/coreyja/coreyja.com/pull/141) that was created opened and merged all without any interaction on my part!

## Future plans

One of my favorite things about this project isn’t the end result of bring back my automated screenshots. It’s about the tools that we built along the way!

Both my `workflow-automation` server and github-oidc` crate open up a ton of doors for future projects!

I have a [domain manager](https://github.com/coreyja/domains) project that I started awhile ago, and that I’ve been wanting to extend. I want to be able to add a file to new projects specifying DNS settings, and then have `domains` talk to the various DNS providers I use to setup the DNS for me, and ideally register it with my web host for that particular project.

And with `github-oidc` I should be able to create a Github Action that talks to this `domains` service without dealing with more credentials!

And I’m sure I’ll find more use cases for `workflow-automation` in the future. I’m a junky for tools and automation, so I can’t imagine it will be too long!

## Wrapping Up and What's Next

That was a journey!

What started as a simple screenshot automation hiccup turned into a deep dive into OIDC, custom servers, and the joys (and occasional headaches) of GitHub's API. But you know what? I wouldn't have it any other way. This is exactly the kind of tinkering that keeps me excited about coding.
I'm already itching to upgrade my domain manager with this OIDC magic.

If you're as much of an automation junkie as I am, I encourage you to play around with GitHub Actions and OIDC. Once you feel the power of temporary credentials with OIDC you’ll never want to add manual secrets again! I know I don’t 😂

And if you want to see how this all came together in real-time (complete with my inevitable "why isn't this working?!" moments), check out the stream recording: <https://youtu.be/jbIVJIQOJ9I>

Got any cool automation ideas or stories of your own? I'd love to hear them! Drop me a line or, better yet, why not turn it into a blog post of your own? After all, the best part of building cool stuff is sharing it with others.

Now, if you'll excuse me, I've got some domains and DNS to automate...
