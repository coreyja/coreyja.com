---
title: caje
subtitle: caje is a caching reverse proxy developed live on stream
repo: https://github.com/coreyja/caje
youtube_playlist: PL0FtqJaYsqZ2v0FezJa15ynwBpo7KE8Xa
status: active
---

caje is a caching reverse proxy developed live on stream by me, coreyja

You can find a YouTube playlist containing all the previous streams at <https://www.youtube.com/playlist?list=PL0FtqJaYsqZ2v0FezJa15ynwBpo7KE8Xa> And you can catch my live streams on my Twitch at <https://twitch.tv/coreyja>

## Overview

caje is a reverse proxy CDN. It sits in between the potentially slow origin server, and the users. It caches the responses from the origin server, and serves them to the users. caje respects CacheControl headers and only caches requests that contain caching headers.

caje plays middleman for all requests to the origin server, including those that are not cached. You point the DNS for your domain to caje, and caje will forward the requests to the origin server.

caje is designed to be run in multiple regions around the world. When one node gets a request for a resource, it saves this information to a manifest that is shared between all nodes.

Currently there is an admin endpoint at `_caje/populate` that looks at this manifest and caches locally any files that are known to other nodes but not saved locally. In this way we can make sure all the nodes have all the cached content, so that requests from any region can be fast. In the future this functionality will be moved to a background process that runs periodically, so the admin endpoint is no longer needed.
