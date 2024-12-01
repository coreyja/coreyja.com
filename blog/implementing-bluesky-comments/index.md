---
title: Implementing Bluesky Comments
author: Corey Alexander
date: 2024-12-01
bsky_url: https://bsky.app/profile/coreyja.com/post/3lcbharoxea2n
---

Yesterday I did the not super original thing of adding Bluesky comments to my blog! Now when I post a new post (including this one!), you can leave comments by responding to my Bluesky post.

It was pretty easy to get working, but a large part of that is due to others doing the same thing recently and sharing their code as examples. It also helps that the Bluesky API is open and easy to work with.

Thanks to everyone who shared their implementation of this recently, including the following which inspired my version!

[@emilyliu.me](https://bsky.app/profile/emilyliu.me) works at Bluesky and recently [made a Gist](https://gist.github.com/emilyliu7321/19ac4e111588bdc0cb4e411c88d9c79a) implemented this idea in React with Tailwind. My site is written in Rust, without any client side code, which meant I couldn’t use the exact Gist. But I was able to reuse the same HTML structure and Tailwind styles, with the React re-written to fit in my Rust site.

I also used [Cory Zue’s](https://bsky.app/profile/coryzue.com) [Post](https://www.coryzue.com/writing/bluesky-comments/) to fill in any gaps, and specifically it helped me figure out how to format my At-Protocol url!

Thanks to both of you for writing up your solutions! Here’s hoping this write-up helps someone out too.

## The Inspiration and Research

After reading the code from Emily and Cory’s post I had a good idea what I wanted to do, but needed to decide how to implement it.

A comment on Cory’s post linked out to [another gist](https://bsky.app/profile/louee.bsky.social/post/3lbsizqjik22o), that had a simple Web Component version that I could have dropped into my site. But I decided to keep my site without client side code for the moment, which left me with doing this on the server. The only issue with this is that it will increase the load times for my blog, since I need to make an API call to fetch the comments before rendering the page. For now this was a fine trade-off, but something I might optimize more in the future

## Technical Implementation

I was pleasantly surprised at how easy it was to use the Bluesky API for this!

All it takes to fetch the full thread of replies is a single API call. The only slight hiccup was converting my `https://bsky.app` URL of the post into an `at://` url that the API would understand.

My full Rust code for doing the URL conversion and hitting the right API is this

```rust
pub async fn fetch_thread(post_url: &str) -> cja::Result<ThreadViewPost> {
    let re = Regex::new(r"/profile/([\w.:]+)/post/([\w]+)").unwrap();
    let caps = re.captures(post_url).unwrap();

    let did = caps.get(1).unwrap().as_str();
    let post_id = caps.get(2).unwrap().as_str();

    let at_proto_uri = format!("at://{did}/app.bsky.feed.post/{post_id}");
    let mut url = Url::parse("https://public.api.bsky.app/xrpc/app.bsky.feed.getPostThread")?;
    url.set_query(Some(&format!("uri={at_proto_uri}")));

    let res = reqwest::get(url).await?;
    let data = res.json::<GetPostThreadOutput>().await?;

    let ThreadViewPostEnum::ThreadViewPost(thread) = data.thread else {
        return Err(cja::color_eyre::eyre::eyre!("Expected thread view post"));
    };

    Ok(thread)
}
```

This uses the `https://public.api.bsky.app/xrpc/app.bsky.feed.getPostThread` API endpoint to fetch the thread details, after first constructing the `at://` URL.

One thing to note about the AT URL is that my code references the variable as the `DID` but in practice that’s actually using my handle of `coreyja.com` and this is working fine! This means I can copy the post URL directly from the web app without having to convert my handle to my DID.

We take the AT Protocol URL and pass it as a query parameter to the `getPostThread` endpoint and get a JSON response back.

Now getting the right JSON return type here was something I was expecting to be tedious. But I was in luck! I found the awesome [`rsky-lexicon` package](https://github.com/blacksky-algorithms/rsky/tree/main/rsky-lexicon) which included all the structs I needed already! This crate is made by [@rudyfraser.com](https://bsky.app/profile/rudyfraser.com) the author of [Blacksky](https://bsky.app/profile/did:plc:d2mkddsbmnrgr3domzg5qexf). This crate was great, and I’m thankful that I didn’t need to model all these objects manually!

After I had all the data, the only thing left was creating a view to show these comments.

For this part I 100% ripped off the HTML from [Emily’s](https://bsky.app/profile/emilyliu.me) [original gist](https://gist.github.com/emilyliu7321/19ac4e111588bdc0cb4e411c88d9c79a). Since I already use Tailwind CSS for my site, I made some simple functions that mirrored the React components and everything worked great!

## Current Limitations and Future Improvements

### Performance considerations

The biggest issue I have with my current approach is that I’m doing the fetch for comments, inline in each request to view my blog. And since this is all server rendered, you can’t see my post content at all until I’ve fetched all the comments. That’s one big benefit on the React version, since it fetches on the client side the rest of the post can render while we wait for the comment fetch to happen.

Currently the API is fast, and my blog is low traffic enough that this is an ok solution. But I’d love to optimize it a bit!

Currently I’m thinking about making a [ReactQuery](https://tanstack.com/query/v5/docs/framework/react/overview) style system in my Rust backend, that allows me to cache arbitrary ‘queries’ and persist them to by Postgres DB. That way when I go to render a blog post, I only need to fetch comments from Bluesky if my most recent fetch was a while ago.

I’d want to set the `stale_time` to something like a minute, so that if it’s been longer than a minute since I checked for new comments I redo the API call. This way the max amount of time between someone writing a comment and it showing up on my site would be 60 seconds. But since my blog gets way less traffic than a visitor every 60 seconds, I left that off for next time!

### Features to implement

There are two features from the original React gist that I haven’t yet copied over to my implementation

First, I didn’t sort the comments at all. Just rendering them in whatever order the API returns them in. The original React sorted by number of likes, which I think makes sense and I’ll likely steal eventually. But it didn’t make it into the v1.

I also left off the toggle to “Show More” comments. My version renders all the comments as a long list. This was easier for me since I don’t have any client side scripting to show a certain number of comments at first, and show more when requested.
Except by writing out this post I think I have a solution to solve that! I should be able to use an [HTML `details` element](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/details) to render all the comments, but hide some behind a click through without needing any JS! Looking forward to implementing this sometime soon, and will do a small write up about that when I do.

## Closing Thoughts

This is the first time I’ve had comments on my site, and I like how simple this solution is. Part of the simplicity comes at the cost of a bit of friction for commenters. You can’t comment directly on my site, you need to go to Bluesky. But for now that tradeoff seems worth it!
Who knows, maybe we’ll experiment with Bluesky OAuth eventually and let you make comment posts right from my site!

Thanks again to everyone who implemented this before me! 💜

Respond to this post on Bluesky to test it out! You should see your comment appear under this article!
