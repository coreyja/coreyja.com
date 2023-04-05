---
title: Migrating From Heroku to Dokku
author: Corey Alexander
date: 2018-10-27
tags:
  - heroku
  - dokku
  - self-hosting
color: purple
---

## Heroku

I've hosted on a number of different providers over the year, but I have always kept coming back to Heroku. They make hosting so simple, a deploy is always only a git push away! With their Github Integration I don't even push to Heroku manually anymore, merges to main auto-deploy.

With that said there was one big drawback to Heroku. The cost. I want HTTPS for all my side-projects, and I also want to use a custom domain for most of them. This right away knocks me out of the free tier. Most of my side projects require a worker and a web process, so on Heroku that's $14 a month per project, minimum. When I started this migration I had 2 side projects on Heroku, and was also wanting to start hosting an [Octobox](https://octobox.io/) instance. This would have cost me about $32 a month on Heroku.

## Finding Dokku

One day while I was looking for a platform to help make self hosting easier I ran into [Dokku](https://github.com/dokku/dokku)! It advertises itself as a `Docker powered mini-Heroku` and that sounded exactly what I wanted. I had minimal experience in Docker, but I like the idea of containers and wanted to dive in. The idea was that after getting Dokku setup, I would be able to replicate my own little Heroku environment.

## Server

I decided to go with Digital Ocean. At first I wanted to move my package tracker side project over and set up Octobox. So I went with a 2GB 1 vCPU box for \$10 a month. I was thinking that this would be about the same as 4 Heroku dynos. A web and a worker, each for the two projects I was wanting to host.

I installed Dokku with the Digital Ocean one click installer without any difficulties and pointed a subdomain at my newly created droplet.
The first setup step with Dokku is through a web installer. This is accessed by going to the ip or domain of the Droplet and following the wizard. It will have you add an SSH key for the Dokku user, as well as some default Virtualhost settings. After the wizard is finished, the Dokku box itself no longer will respond to web requests, it has no web GUI.

Since there is no GUI you interact with the Dokku box over SSH. While you can SSH into the box as root, and run the `dokku` command locally. It is recommended to SSH as the dokku user. Since this user is limited to only run the `dokku` command, you have to use remote commands to send the cli options you want to pass to the `dokku` command. For example to check the version of Dokku you may run a command like

```bash
ssh -t dokku@DOKKU_HOST version
```

## Setting up a Rails App

This mostly follows the [official docs](http://dokku.viewdocs.io/dokku/deployment/application-deployment/#deploying-to-dokku)

First step was the create the first app we want to deploy

```bash
ssh -t dokku@DOKKU_HOST apps:create APP_NAME
```

Next was adding the Postgres plugin for my DB, and linking it to the app. To add Postgres you need to run as sudo, so lets SSH into the box as root.

```bash
ssh root@DOKKU_HOST
sudo dokku plugin:install https://github.com/dokku/dokku-postgres.git
exit
```

Now we need to create the DB and link it to our app. App and database names to NOT need to be unique. Since I was planning on only running a single env of an app, I decided naming my DB and app the same would be easier to keep track of.

```bash
ssh -t dokku@DOKKU_HOST postgres:create APP_NAME
ssh -t dokku@DOKKU_HOST postgres:link APP_NAME APP_NAME
```

### Migrating existing DB

If this was a brand new app, we could skip this step. But this app has an existing database on Heroku that we need to migrate over.

Heroku makes it easy to export a backup of our DB and download it using the following commands

```bash
heroku pg:backups:capture
heroku pg:backups:download
```

The Dokku postgres plugin also provides an easy way to import this backup. The only hoop we have to jump through is uploading our DB backup up to our Dokku server so we can import it. This is a job for SCP

```bash
scp database.dump root@DOKKU_HOST:/tmp
ssh -t dokku@DOKKU_HOST postgres:import APP_NAME < /tmp/database.dump
```

### Environment Variables

Copying the ENV vars from Heroku was pretty simple.

```bash
heroku config
```

This will output the ENV vars for an existing Heroku app. Then I simply added each to Dokku using the following command

```bash
ssh -t dokku@DOKKU_HOST config:set APP_NAME KEY1=VALUE1 KEY2=VALUE2
```

The one ENV variable you do NOT want to copy down is the `DATABASE_URL`. This is the URL of our Heroku database, and when we linked our new Dokku database and app earlier, we already set a `DATABASE_URL` ENV var that will be available to your app.

### Domains

By default, if you set up a host name during the initial web GUI setup, your app will be available at a sub-domain of your DOKKU_HOST. The sub-domain will be your app name. So for a HOST_NAME of `dokku.somecoolsite.com` and an APP_NAME of `AwesomeApp` the app url would be `awesomeapp.dokku.somecoolsite.com`

You can add custom URLs though. Simply set the domain's DNS to point to the IP of your Dokku host. Then run a command similar to the following

```bash
ssh -t dokku@DOKKU_HOST domains:add APP_NAME CUSTOM_DOMAIN
```

We can now cleanup the default if we want to. We can do that using the `domains:remove` command

```bash
ssh -t dokku@DOKKU_HOST domains:remove APP_NAME APP_NAME.DOKKU_HOST
```

### Dockerfile

The package tracker side project is a Rails app, and I found this great article about setting up a Rails app on Dokku, using a Docker ["Optimize Dokku Deployment Speed for Ruby on Rails with Dockerfile"](https://pawelurbanek.com/optimize-dokku-deployment-speed). This article was more focused on transitioning from a Dokku buildpack based build, to a Dockerfile based one. I wanted to go right to a Dockerfile based deploy, but I found this article to be a great resource anyways.

Following along in that article I got a `Dockerfile` all setup, and added the pre and post deploy tasks to `app.json`.

Now we are just was a push away from deploying our app!

```bash
git remote add dokku dokku@DOKKU_HOST
git push dokku main
```

This will look similar to a Heroku deploy, in that we will see the deploy output in your git push. Here you can see it building your Dockerfile, and when it is done your app will be available!

### Scaling Processes

Similar to dynos in Heroku, we can spin up multiple of each different type of process if the apps requires it for horizontal scaling. This is a side project app that has little traffic so we only need one web and one worker process.

```bash
ssh -t dokku@DOKKU_HOST ps:scale APP_NAME web=1 worker=1
```

## Limitations of Dokku

One of the big limitations of Dokku is that everything runs on a single server. If your app needs to scale more than a single server, Dokku isn't going to be the right tool for the job.

## What's next?

While non of the my apps currently need the scale provided by being able to scale horizontally to multiple servers, I still don't like that being a limiting factor in my setup. I'm investigating Kubernetes to manage my 'fleet' of personal projects and hosted apps, and am thinking that will be my next step.
