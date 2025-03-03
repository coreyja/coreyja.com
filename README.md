# coreyja.com

New and Hopefully Improved Personal Site

## Development

### Zero-Downtime Reloading

This project uses `systemfd` and `cargo-watch` to enable zero-downtime reloading during development. This allows the server to keep serving requests while recompiling code in the background.

Install the required tools:

```bash
cargo install systemfd cargo-watch
```

Run the development server with:

```bash
foreman start
```

This setup allows:
1. The socket to remain open during code changes
2. The old version to continue serving requests while the new version compiles
3. A smooth transition to the new version once compiled

This works automatically in development with `systemfd`, and in production it falls back to normal socket binding with `cargo run`.

## Screenshots

There are generated from `shot-scraper` on the 'live' site

<details><summary>iPhone</summary>

![Screenshot at iPhone](screenshots/iphone.png)
</details>

<details><summary>4K</summary>

  ![Screenshot at 4k](screenshots/4k.png)
</details>

<details><summary>1080p</summary>

![Screenshot at 1080p](screenshots/desktop.png)
</details>

<details><summary>iPad</summary>

![Screenshot at iPad](screenshots/ipad.png)
</details>
