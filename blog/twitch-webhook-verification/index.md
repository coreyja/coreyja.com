---
title: Verifying Twitch Webhooks in Rails
author: Corey Alexander
date: 2019-05-04
tags:
  - twitch
  - rails
  - livestreaming
  - webhooks
  - ruby
color: orange
---

I was recently working on adding some Twitch embedding support to dev.to! Part of this was adding webhook support.

One of the things that Twitch recommends doing, and I wanted to do, was verify that the webhooks came from Twitch and not a malicious third party. This works by passing a secret when you register for the webhook and using that and the contents of the body to create a hash and verify the payload came from Twitch.
The bottom of their Webhook Guide mentions verifying payloads but doesn't give a lot of context of how to actually do that, [https://dev.twitch.tv/docs/api/webhooks-guide/](https://dev.twitch.tv/docs/api/webhooks-guide/)

At first I thought I needed to do a 'normal' SHA256 with the secret and payload data concatenated together. But this was not correct I needed to make a SHA256 HMAC using the secret as the HMAC secret, and the payload as the entirety of the body. I figured this out by reading some issues online, but they were all in JS so I had to figure out the right Ruby methods to use and luckily that wasn't too hard.

Here is the long for way to do this in Ruby using the OpenSSL Library

```ruby
digest = OpenSSL::Digest::SHA256.new
secret = "SECRET_SENT_TO_TWITCH_AT_WEBHOOK_REGISTRATION"
hmac = OpenSSL::HMAC.new(secret, digest)
hmac << "BODY_OF_PAYLOAD_HERE"
hmac.hexdigest
```

There is also a short cut method that looks like this!

```ruby
secret = "SECRET_SENT_TO_TWITCH_AT_WEBHOOK_REGISTRATION"
OpenSSL::HMAC.hexdigest("SHA256", secret, "BODY_OF_PAYLOAD_HERE")
```

What this actually looked like the in Rails project I was working on was something like this to determine if the request was valid. This assumes the Twitch Secret is stored in an ENV variable.

```ruby
def webhook_verified?
  twitch_sha = request.headers["x-hub-signature"]
  digest = OpenSSL::HMAC.hexdigest("SHA256", ENV["TWITCH_WEBHOOK_SECRET"], request.raw_post)

  twitch_sha == "sha256=#{digest}"
end
```

I used this method in my controller action to determine what action to take in response to the webhook! In case the webhook is not verified I decided to simply reply with a 204 No Content, and make no local changes. This way if some malicious actor was trying to impersonate Twitch I would not immediately alert them of their requests failing.
