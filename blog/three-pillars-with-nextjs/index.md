---
title: Three Pillars with Next.js
author: Corey Alexander
date: 2025-02-07
---

The basic idea here is that we want to create a [3 Pillars](/posts/three-pillars/) architecture for our app.

The Three Pillars are: web, worker and cron.

The `web` pillar is what we are getting from `next.js`! This handles web requests and renders the app.

The two that we need to make are `worker` and `cron`. The `worker` will be responsible for doing background work. And `cron` will add new background work to the queue at certain times or intervals.

We are going to have three separate ‘processes’ now instead of the single next.js process.

## Procfile.dev

Since we will have multiple processes/commands that we need to run at the same time for local development to work.

Let’s install [`overmind`](https://github.com/DarthSim/overmind) to help manage this process.

Then we’ll make a new file in the base of the repo called `Procfile.dev`. Procfile is a ‘standardized’ format for a file that defines multiple processes that we need to run. The `.dev` postfix is just to remind us that this is for local development, NOT production. We likely won’t use a Procfile for production here, but often I do so I like adding the `.dev` for clarity.

For now we just have the web process from `next.js`, so we’ll put that in the `Procfile.dev` for now. That will look something like

```Procfile
web: pnpm dev
```

This will run the already defined `dev` script from the `package.json`

We will add to this Procfile.dev as we create the other processes.

## `worker` process

Ok so next up is creating a Worker process. Basically what this needs to do is sit in an infinite loop, and query the DB for new jobs to run. If there is a job it should ‘pick them up’ and run the specific job in question. Once it’s done with that job it should return to its loop.

Our first step is setting up the DB table where we will store jobs that need to be worked on. Here is the schema I am using for my Rust site

```sql
create table jobs
(
    job_id     uuid                                   not null primary key,
    name       text                                   not null,
    payload    jsonb                                  not null,
    priority   integer                                not null,
    run_at     timestamp with time zone default now() not null,
    created_at timestamp with time zone default now() not null,
    locked_at  timestamp with time zone,
    locked_by  text,
    context    text                                   not null
);

CREATE INDEX jobs_priority ON jobs (priority, run_at);
```

Once we have the table setup we can work on the ‘worker’ process/code.

We want to create a new file that will be the “entry point” for our worker process. Something like ‘src/worker.ts’. For now just put a console.log line there so we can make sure it works when we call it from the command line. This file will behave like a “script”, it will just execute its contents when we run it from the command line.

Let’s add a script entry to the package.json now. Call it worker and have it run ‘node src/worker.ts’.
Now we should be able to run ‘pnpm run worker’ and see our console message!

From here we have a home for our worker code!

Now let’s setup the job loop!

We want this worker script to have an infinite loop where it looks for new jobs, runs one if it exists and sleeps for a second if it doesn’t.

For pulling a job we want to use a sql query like this to efficiently pull things off the queue. This is from my rust project

```sql
‌UPDATE jobs
SET LOCKED_BY = $1, LOCKED_AT = NOW()
WHERE job_id = (
    SELECT job_id
    FROM jobs
    WHERE run_at <= NOW() AND locked_by IS NULL
    ORDER BY priority DESC, created_at ASC
    LIMIT 1
    FOR UPDATE SKIP LOCKED
)
RETURNING job_id, name, payload, priority, run_at, created_at, context;
```

This will lock a job and say it’s being worked on by our specific worker! (We will deal with jobs that are locked and don’t actually run later, but good to start with this)

Then we can use the name to find the job function in TS to run. I think we should basically have a big object that maps string name to job function. These functions should take an object of properties they need to run, which you will get from the JSON in payload. There is more stuff we can add to this post to make it fancy/type safe but we will leave the off for now!

One you finish a job function you want to delete the job row from the table so mark it complete!

Then we start the loop over! Here is my rust code for this worker

https://github.com/coreyja/coreyja.com/blob/main/cja/src/jobs/worker.rs

We also want to add a line to our Procfile that will run this script along side the web process now! This way if we enqueue a job from the web it will run on the worker nicely locally!

## Cron

Same idea as worker. Let’s add a new file called cron.ts and wire it up as a script in package.json

In this we want to define things that need to happen at certain times or at an interval.
And we loop and sleep until it’s time to enqueue some jobs from here!
Can expand on this section b it typing it from mobile lol

https://github.com/coreyja/coreyja.com/blob/main/cja/src/cron/worker.rs

## Deploy steps

We can worry about this after all the local stuff is working!

### fly.io Multiple Processes

We want to use "Process Groups" for this. This way each 'service' will run on its own machine, providing more isolation between services. If we were looking to save a few bucks, we might look into the "Process Manager" approach they mention, using something like `overmind`.

https://fly.io/docs/app-guides/multiple-processes/#process-groups

### Run migrations as `release` commands

https://fly.io/docs/launch/deploy/#release-commands
