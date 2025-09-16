---
title: "Introducing backup.blue: Back Up Your AtProto Account to Your iPhone (or Mac)"
author: Corey Alexander
date: 2025-09-16
is_newsletter: true
bsky_url: https://bsky.app/profile/coreyja.com/post/3lyxgzno2ps2n
---

Hey Team! Been awhile, sorry for the lack of updates. I’ve been working on lots of random projects, but nothing that’s really stuck. But that’s changing! I’ve got a new project I’m excited about and plan on shipping, on a reasonable timeframe!

So with that, I’m excited to introduce Backup.blue, my iOS app that lets you back up your AtProto/Bluesky account right to your phone — and have that backup saved to iCloud for safekeeping. Eventually the plan is to also let you upload that backup to a new PDS, create and manage your signing key, and monitor the PLC log for any un-authorized changes to your DID document.

## Why back up your AtProto account?

The whole promise of AtProto is that you actually own your data — your posts, your follows, your entire social graph. But that ownership only matters if you can actually access and move your data.

Right now, your data lives on whatever PDS you're using. If that server goes down, decides to shut down, or loses your trust, you need a backup to make the move.
Without one, you're stuck. With one? You can pack up your entire social presence and move it anywhere that speaks AtProto.
Plus, having your own backup means you can experiment with the data yourself — build tools, analyze your posting patterns, or just have peace of mind knowing your content is truly yours. That's the power of an open protocol!

## What Backup.blue does today

It doesn’t do a _ton_ but I got a speed boost right from the beginning, because the awesome [@baileytownsend.dev](https://bsky.app/profile/baileytownsend.dev) open sourced his [AT Toolbox](https://tangled.sh/@baileytownsend.dev/at_toolbox) iOS app that provides an assortment of Shortcuts to make it easy to automate any part of your AT Proto account!
This had code for downloading a repo’s CAR file, and then iterating through all the blobs to download and save them.
And with this my first version of backup.blue was born!

Here is a demo video I made after getting an experimental Live Island support working!
https://bsky.app/profile/coreyja.com/post/3lxhq6ynltc2q

Right now backup.blue:

- Backs up your BlueSky account to your device, by downloading the CAR file and all blobs
- Saves the backup to iCloud so you have an off-device copy.

## What’s Next

Up next is getting multi-device syncing working well. I looked at some Apple libraries to do this like their CloudKit. But from my research devs weren’t always happy with this syncing, and it seemed like a great time to learn something new, CRDTs!

Now I’m working on implementing a relatively simple CRDT (Conflict Resolving Data Type), and sync system so that any changes you make on one device are reflected on all your other Apple Devices (oh ya this app is going to work on your Mac too!)

The primitives we have to work with are SQLite and regular files.
The plan is to have one ‘main’ DB which will act as a “materialized view” of the data. We won’t write edits to it directly, instead we will create `Operations`. We’ll store these twice: first in an `operations.sqlite` DB for easy access and also in an immutable `op-1234-4566.json` type file, where each ‘bundle’ of operations will get a unique JSON file.
When a device goes to do a sync, it will first look for any `op-*.json` files that it doesn’t have in `operations.sqlite` and imports those operations. If new operations were loaded into the op DB, then we will truncate the materialized view DB and re-create it from the Operation Log! And the magic of the CRDT, is that even if the operations are processed out of order we should end up in a consistent end state across all devices.

## Other News

One roadblock that I identified early in this process is the App Store. Initially I thought I wanted to use CloudKit, and to do that you need a Business Apple Developer Account. A personal account wasn’t going to cut it.

And this was the jolt I needed to finally file that LLC paperwork I’ve been thinking about for awhile. Now `COREYJA LLC` is officially licensed with the State of NJ, and my Apple Dev account is being transitioned over as I type.

Was worried at first that I was going to have a completed app before all the paperwork was done, but with the LLC all created and the app not ready for Alpha testing yet it looks like I mis-calculated there lol. So good news is that this won’t block us, and as soon as the app is ready for more eyes we can submit an Alpha release to the store!

Thanks for reading — stay tuned for demos, status updates, and the App Store launch! I’m also planning on creating a dedicated account for backup.blue, keep your eyes peeled!
