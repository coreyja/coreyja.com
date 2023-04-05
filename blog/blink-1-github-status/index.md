---
title: blink(1) Github Status
author: Corey Alexander
date: 2018-04-29T18:28:05-04:00
color: red
tags:
  - blink1
  - github
  - graphql
  - ruby
---

A little while back I got a [blink(1)](https://blink1.thingm.com/), which is a cool little USB light that is fully programable. While it's been really fun to play with but I haven't really used it for much. Recently I was talking with some coworkers and realized that a great use for it would be as a Github status indicator, telling me the status of a specific Github branch. I'll have it set up to track our main branch and report on the CI status.

Right now it turns the light red to indicate failure, green to indicate success and purple to indicate failure. If the status is pending it will flash yellow and then show the color of the previous status, dimmed out by how long the status has been pending.

Before we get too far, here is a link to the repo: [https://github.com/coreyja/blink1-github-status](https://github.com/coreyja/blink1-github-status)

# Github Status API

The first version used the v3 Github API to check the statuses endpoint. This worked well, but couldn't support a use case I had in mind. If the current status was pending, I wanted to show the status of the last build, but dim it depending on how long the pending build has been pending. The more recent it is the brighter it is. This means that a light green light would indicate that the tests have been running for awhile and the last run was a success.

With this requirement we needed to also look the history of commits and their status. From here I decided I wanted to use the v4 GraphQL API. I definitely could have got what I wanted from the v3 API, but I wanted to give GraphQL a try.

# GraphQL

GraphQL is a new way to write APIs, where instead of providing multiple endpoints you provide a single endpoint and consumers right queries to get the data they want from your API. So for this example, instead of finding the correct APIs to get the history of a branches status, I needed to write a GraphQL query to get that data. The most indispensable tool while working on this was Github's [Explorer](https://developer.github.com/v4/explorer/) which is a usage of the [GraphiQL](https://github.com/graphql/graphiql). This provides a real-time way to write a query and see what data Github will return. This is how I played with my query, to get it right, before I integrated it into my program.

Eventually I found a query that would work for the information I was looking for. It would look up the designated branch (ex: 'main') for the given repo. From there it would look at last N commits, and include any status checks that exist for the commit. It includes the overall status of the commit, as well as the as the individual status `contexts`. The following is the query I am using:

```graphql
query(
  $owner: String!
  $name: String!
  $branch: String!
  $pageSize: Int
  $cursor: String
) {
  repository(owner: $owner, name: $name) {
    ref(qualifiedName: $branch) {
      target {
        ... on Commit {
          history(first: $pageSize, after: $cursor) {
            pageInfo {
              hasNextPage
              endCursor
            }
            nodes {
              oid
              status {
                state
                contexts {
                  state
                  context
                  createdAt
                }
              }
            }
          }
        }
      }
    }
  }
}
```

One thing that I have the basics for in the query that I haven't implemented yet is pagination. The current version just fetches a single page of result, but the basics of pagination are already built into the above query.

## blink(1)

I have the blink(1) m2 which has 2 leds that can be controlled independently. The ruby gem for interacting with the blink(1) is [rb-blink1](http://ngs.github.io/rb-blink1/). Unfortunately it currently doesn't support setting the two leds independently. It is a wrapper around the official C API, which also powers the CLI tool. The CLI tool does support setting each LED, and the gem hasn't been updated recently. So the gem could be updated to support both LEDs, which is a project I'm interested in working on in the future!

## The Code and Crontab

Unsurprisingly I decided to implement this in Ruby. It's currently a Ruby script, with a few classes extracted, but I want to make it a gem with an executable in the future. There is definitely some more extraction and modeling that needs to be done, as the script currently does most of the heavy lifting. I didn't want to implement this as any kind of service that I had to keep alive, so it is a simple script that I am currently running through cron. I may look into running it through launchd in the future, but cron got me up and running faster.

One quirk with cron is that it does not run commands with the PATH and other env variables set to what you would expect from an interactive terminal. Because of this full paths to executables are frequently required. Since this is simply a ruby script and not a standalone executable I had to jump through a small hoop to get it set up. I needed to run something like `bundle exec ./exe/run_light.rb coreyja/glassy-collections main` from cron.

The issue I ran into was that `bundle` was not in the limited PATH that cron ran in. My solution was probably a bit of overkill, but its getting the job done until I make a proper executable. I use `bash -c` to create a sub-shell. Inside this I source my bash_profile, cd to the project directory, and then from there I was able to run my `bundle exec` command. I also included the `BLINK1_GITHUB_TOKEN` directly in the crontab entry since it isn't in the bash_profile.

Here is my current entry in my crontab. I currently have it set to run every 5 minutes. I also pipe everything to /dev/null to that it doesn't send me Terminal mail every time it runs. Ideally it should probably only output on an error so I can not capture all the output and let the mail indicate that there was an issue.

```crontab
*/5 * * * * bash -c "source ~/.bash_profile && cd ~/Projects/blink1-github-status && BLINK1_GITHUB_TOKEN=FAKE_TOKEN bundle exec ./exe/set_light.rb coreyja/glassy-collections main" &>/dev/null
```

## The Future

Along with making this a gem, I want to look into creating a Homebrew tap for distribution. There are some homebrew gem projects that I found quickly that looked pretty interesting. Since I use RBenv to manage multiple ruby versions it can be hard to install a gem as a global executable. Some options include installing the gem for each version of ruby you use, or some clever bash aliasing, neither of which I am a huge fan of. Using homebrew to install global executables written in Ruby sounds like a better solution to the problem to me, so I am interested in giving it a shot.

Another thing I want to iterate on is adding pagination to the GraphQL query. I added the building blocks into the query I wrote, and now I want to take advantage of them to make sure I am fetching all the commits I need. Right now if there aren't any statuses found in 1 page of commits, the light will go purple to indicate that no status was found. In the future I would like it to be able to continue to ask for more commits to find the most recent one with a status. Hopefully in this use case you won't have more than a handful of untest commits on main ;-) but I still would like to implement pagination.
