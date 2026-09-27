# basic-http-rs

A tiny static file server written in Rust.

## Why

Two reasons:

- To get familiar with Rust. This is a learning project.
- Pulling in a whole nginx container just to serve a folder of HTML, CSS and JS
  always felt like too much. This does the one thing I need and nothing else.

Not a big deal. Don't use it for anything serious.

## What it does

- Serves files from a directory over plain HTTP.
- Guesses `Content-Type` from the file extension.
- Rejects any path containing `..` with a 400.
- Each connection is handled in its own tokio task.

What you get for a path depends on whether the served directory has an `index.html`:

- **With `index.html`** (a website or single page app): `/` serves it, and any
  path that doesn't exist also falls back to it, so client-side routing works.
- **Without `index.html`** (a plain folder of files): you get a directory browser
  instead. `/` and any subfolder show a list of folders and files with a back link.

The browser page is rendered from `directory.html`, which is embedded into the
binary at compile time. So the Docker image is still just one file.

## Usage

```
basic-http-rs [-a <address>] [-p <port>] [<path>]
```

| Flag | Default | Meaning |
|---|---|---|
| `-a`, `--address` | `0.0.0.0` | Address to bind |
| `-p`, `--port` | `8080` | Port to bind |
| `<path>` | `.` | Directory to serve |

Run it natively:

```sh
cargo build --release
./target/release/basic-http-rs -p 3000 ./dist
```

## Docker

The image is built from `scratch` and contains only the static musl binary.
It is about 1.2 MB and runs as `nobody`. There is no shell inside, so use
`docker logs` for debugging.

```sh
docker build -t basic-http-rs .
docker run --rm -p 8080:80 -v ./dist:/srv:ro basic-http-rs
```

The container listens on port 80 and serves whatever is mounted at `/srv`.

## Limitations

- HTTP/1.1 only, no HTTPS.
- No compression, no caching headers.
- The whole file is read into memory before it is sent.

These are fine for serving a small site on a local network, which is all this
is meant for.
