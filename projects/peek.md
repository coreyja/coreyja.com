---
title: Peek
repo: https://github.com/coreyja/peek
status: on-ice
---

Peek - News and Weather
This is an app to show news and weather for team members of remote teams. You'll be able to see the weather and local news for all your teammates in one convenient place

Crates
server
This is where the main server code lives.

The web side of things is powered by Axum: <https://github.com/tokio-rs/axum> We are using maud for templates: <https://maud.lambda.xyz/> We are using sqlx: <https://github.com/launchbadge/sqlx> With SqLite And Litestream: <https://litestream.io/>

frontend
This is compiled with WASM and loaded as JS into the server to run on the clients

E2E Testing
We have end to end testing via Cypress: <https://www.cypress.io/>

Code lives in the `./browser-tests` directory
