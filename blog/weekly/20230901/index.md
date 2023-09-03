---
title: Sept 1st Update - Lockfiles
author: Corey Alexander
date: 2023-09-01
is_newsletter: true
---

Hey Team! Its September now, so how was everyones August?

Today I want to talk a little about lock files, especially since the Cargo Team just updated their guidance about them in this blog post: [Change in Guidance on Committing Lockfiles](https://blog.rust-lang.org/2023/08/29/committing-lockfiles.html)

## This Week

We had two good stream this week!

### Sunday, Aug 27th

Link: <https://youtu.be/EBmDL9ZYI_Y>

On Sunday we worked more on our Bot Byte! We fixed some issues I had introduced since the previous stream, and got Byte able to listen to my audio again nicely!

After that we made use of our new neon.tech powered DB to save and store chatters preferred nicknames. We also use this to customize how Byte pronounces your name! For instance I set my nickname to `Corey Jay Aye` so that it would pronounce my username correctly.

I'm really excited to keep improving Byte and using them as I stream!
Let me know what else we should teach Byte to do on upcoming streams!

### Wednesday Aug 30th

Link: <https://youtu.be/oTS7LB2ChK8>

On Wednesday we got back to my most requested topic recently, Building a CDN in Rust!

Previously we had gotten our server stood up to Proxy requests to our origin server, and cache each request without checking its cache ability,
And on Wesnesday we fixed that! Now we check the `Cache-Control` and other relevant headers to determine if a resource is cachable. If it isn't we always make the live request from the origin server!

While streaming we found out that Chromium in developer mode always adds `Max-Age=0` as a header, which was forcibly 'shutting down' my caching proxy! We switched over to Firefox and were better able to test the caching behavior.

The brains of our header checking were done by the[`http-cache-semantics` crate](https://docs.rs/http-cache-semantics/latest/http_cache_semantics/).

We wrapped up this stream by making a plan for next time! We still want to add a 'manifest' of cached pages, and a way to share this manifest between nodes so that different nodes can cache the content without it being requested through them!

## coreyja's Ramblings

### Lockfiles

Today I want to talk about package manager lockfiles! In particular I'm going to focus on the two languages and package managers I have the most experience with Ruby and Rust.

Ruby with Bundler and Rust with Cargo have very similar philosophies. And that makes sense knowing that Cargo was written by some of the same people who developed Bundler for Ruby! Here is the announcement post for [Cargo (on archived version)](https://archive.ph/Cox1H) which mentions the dev Bundler roots.

Lockfiles are an important part of the Cargo and Bundler designs. In Ruby we have the `Gemfile.lock` and in Rust we have the `Cargo.lock` file. These files specify the exact version of your dependencies that you have installed.

When you add a dependency to Cargo or Bundler you specify a range of versions that your code can work with. In Bundler you might add some a range like this `~> 1.1.0`. This range means that we need AT LEAST version `1.1.0`, and that we also allow any version where the last digit is higher. So `~> 1.1.0` allows version `1.1.1` or even `1.1.99`. Cargo does a very similar thing, but you don't need the `~>` operator as this is Cargo's default behavior!

These version ranges live in either the `Cargo.toml` or `Gemfile`, and when you install the dependencies for the first time a lock file is created, that lists the exact version of the dependency that was installed. So if your version range was `~> 1.1.0` the lock file might specify `1.1.99` if that was the latest matching version at the time you installed.

If you committed your lock file to version control, any other developers on your project would get the EXACT same version of the dependencies. Even if `1.1.100` comes out, without updating the lock file everyone will still get `1.1.99` when they install dependencies.

And this can be really powerful! It allows all the developers to have the same version of every dependency so as to avoid issues cause by differences in versions. Or cases where something works for one developer but not for another. Having and committing a lock file can eliminate a whole class of issues in dependency management.

And when you want to start using a new version, you can have your tool of choice update the lock file to a new compatible version! And every developer will install that new version when they pull that lock file from version control.

What I described above is considered the "best practice" for applications, where the final result is an application to run and not a library.

Libraries are a bit different.

See in a library you don't have as much control over what versions of dependencies are installed. This is because these package managers **do not take lockfiles into account when installing a library**. This is often something that trips people up at first when they haven't run into it before.
But libraries _only_ use the version ranges in your `Cargo.toml` or `Bundler` file to decide what version of dependencies to install. This is because both of these package managers try to install dependency versions that work for your whole tree of dependencies. This is more exaggerated in Ruby and Bundler, where you can only have a single version of any dependency.

Let's say we have 3 gems (what Ruby calls packages). `base_library` has no dependencies. `lib_a` depends on `base_library ~> 1.0` and `lib_b` depends on `base_library <= 1.2`. Since we can only have one version of `base_libary` installed we have to find one that matches both `~> 1.0` and `<= 1.20`. This might be `1.2.0` which satisfies both ranges. If thought `lib_b` depended on `base_library < 1.0` we'd have a problem. There isn't any version that can match both `~> 1.0` and `< 1.0` at the same time. Here Bundler will blow up, saying it couldn't find matching dependencies.

Cargo is a little more lenient here as it allows multiple versions of the same dependency to be installed. Though you might run into issues later if you try to pass Structs from one version to methods from a different version.

Since libraries don't use the lock file for resolving dependency trees, there is debate on what you should do with lockfiles in libraries.

Until recently the `Cargo` teams advice has been to commit lockfiles for applications, but NOT for libraries.

This is so that when CI runs, it will always pull the latest possible version of your dependencies. This lets you know what a user of you package might get if they installed it for the first time. It can help find incompatibilities and bugs with the newest versions of your deps.

But of course, this does mean that different developers working on the same project won't have the same exact versions of things. Each individual will have their own local lock file.

And Cargo recently amended their stance to say that it's ok to commit the lock file for libraries too, and to consider it for your project.
This is a very reasonable answer because each project is different. On some projects keeping all the developers on the same version might outweigh the benefits of potentially finding issues earlier in CI. The "don't commit" for libraries advice also kinda implies that you have a working CI that could potentially catch issues early. If you don't then some of the benefits of not committing your lock file are lost.
And on the flip side it's possible to get the benefits of not committing your lock file in other ways. I've recently heard of a project that commits their lock file, but has an automation that periodically updated the lock file with the latest releases. Or a different project that has two CI jobs setup, one that uses the committed lock file and one that does NOT and grabs the latest compatible versions.

I like this new advice in that it acknowledges that there are tradeoffs that might not apply to everyone. However I do like that the old advice as a bit more universal. Universal standards are nice in that they can reduce the number of things someone starting out has to think about.

And I'm really glad to hear from different people around the internet, and hear their tips and tricks for dealing with dependency versioning!

I was firmly in the "don't commit for libraries" camp, and think I still lean that way. But I am very intrigued by the idea of committing the lock file and having automation to update it on some cadence. I think I'll play around with this more sometime!

And finally there is one piece to this puzzle that I didn't get today and that is Cargo Workspaces. These allow you to bundle a bunch of crates together into a workspace, and have a single lock file for the whole workspace! This is really nice and has quickly become my default for Rust projects. But it is very interesting as it relates to this topic, cause now you have a single lock file potentially for both apps AND libraries. And I think in this case the tradeoffs are different. So far I've always committed lockfiles in workspaces, and I think that makes the most sense. Since there are applications in my workspaces, it makes sense to lock the deps for those versions. This works great for me because most of the library style crates in my workspaces aren't published externally so their versioning isn't super important. But I think I want to expand on this topic focusing on workspaces for an upcoming newsletter, so be on the lookout for that!

And with that I'm going to wrap up for the day. Thanks for reading team, and see ya online this week!
