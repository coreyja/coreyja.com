---
title: Automating Screenshots and Github Actions
author: Corey Alexander
date: 2023-09-06
tags:
  - github-actions
  - oidc
  - automating
---

## Introduction

I’ve been using [`shot-scraper` ](https://github.com/simonw/shot-scraper?tab=readme-ov-file#shot-scraper) by (Simin Willson)[https://simonwillison.net/] to take screenshots of my blog everyday, and commit them back to my blogs git repo. This makes it easy to check what my site looks like currently on a few screen sizes, and gives me a way to see how my blog has changed over time!
For example [here is a quick video](https://coreyja.com/til/video-from-screenshot-history) I made about a year ago showing how my Blog had changed since I launched its latest iteration.

This is all setup using an [scheduled Github Action](https://github.com/coreyja/coreyja.com/blob/3b913cb09fa1a2ad44fe47b3a284a78c25b13fdb/.github/workflows/shots.yml). Every day it would dutifully kick off and take screenshots and commit them back to the main branch.

This was working great! Until, all of a sudden, it wasn’t :sadnerd:

## The Problem

Last week me and my wife drove up to “The Cottage”, my families lake house get away. And while she was relaxing and enjoying the views, I cracked open my laptop get some blog work done.

And that’s when I noticed it for the first time.

The dreaded Red X of Github Actions failing.

Screenshot 2024-08-31 at 4.06.10 PM.png
Title: Red X of Github Actions
Alt: Screenshot from my repo’s Github Action, which is failing and displaying with the big red X. There is a single job in the workflow called `shot-scraper` which is failing, and reporting an exit code of 1

Looking into this more, it turned out to be my own fault. My blog is mostly a ‘single player’ repo; it’s almost always my fault when things break.

At least this time the error message made it clear what was going on.

Screenshot 2024-08-31 at 4.13.19 PM.png
Title: Github Actions Error Message
Alt: Error message screenshot from the Logs of the previous Actions. The relevant error line is `Changes must be made through a pull request.`

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

Setting up a GitHub Action to:
Create a PR automatically from the scheduled action.
Approve and auto-merge the PR once tests pass.
Encountering GitHub's infinite loop prevention:
Actions cannot trigger other actions if they create a PR.
Actions cannot approve their own PRs due to GitHub's credentials restrictions.

## Exploring Alternatives

Consideration of alternatives:
Using a personal access token to overcome GitHub’s restriction.
Challenges with personal access tokens: expiration and secret management.

## Switching to OpenID Connect (OIDC)

Discovery and exploration of OpenID Connect (OIDC) as a solution:
Explanation of OIDC and its benefits over personal access tokens.
Plan to use OIDC for authenticating GitHub Actions with a custom server.
Realization of how to implement the OIDC solution:
Setting up a server to verify OIDC tokens and handle PR creation and management.

## Final Solution Plan

Overview of the final solution:
GitHub Action takes screenshots and makes a branch.
Action sends branch details to a new automation server.
Automation server creates PR and sets it to auto-merge after tests pass.

## Implementation Steps

Detailed steps for the new solution:
Setting up the server and GitHub App for handling automation.
Integrating Safe’s OIDC crate for verification and authentication.
Using the new system to streamline screenshot automation.
Next Steps and Considerations
Potential improvements:
Evaluating whether to disable certain CI steps for automated PRs.

## Future plans:

Live-streaming the implementation process.
Publishing the OIDC crate and using it in the new automation setup.

## Conclusion

Reflecting on the challenges and the journey to the solution.
Emphasizing the value of iterative problem-solving in development.
Encouraging readers to experiment with GitHub Actions and OIDC for their automation needs.

## Call to Action

Inviting readers to join the live stream or follow the progress on the project.
Providing links or resources for further reading on GitHub Actions, OIDC, and related tools.
