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
- Optional brotli and gzip compression (see below).
- HTTP parsing is hand-written. No HTTP framework or parser crate.

What you get for a path depends on whether the served directory has an `index.html`:

- **With `index.html`** (a website or single page app): `/` serves it, and any
  path that doesn't exist also falls back to it, so client-side routing works.
- **Without `index.html`** (a plain folder of files): you get a directory browser
  instead. `/` and any subfolder show a list of folders and files with a back link.

The browser page is rendered from `directory.html`, which is embedded into the
binary at compile time. So the Docker image is still just one file.

## Compression

Off by default. Turn it on with `-c` or `--enable-compression`.

When it is on, a response is compressed only if all of these are true:

- The client sends `Accept-Encoding` with `br` or `gzip`. Brotli is preferred.
- The body is larger than 1 KB.
- The content is text-like: HTML, CSS, JS, JSON, XML, SVG, wasm, or TTF/OTF fonts.
  Images, video and other binaries are sent as-is.

Compression runs on every request, in memory. Nothing is cached. So it trades
CPU for bandwidth. On a local network the network is rarely the bottleneck, so it
can make things slower there. Over a slow link it helps. Pick based on where it runs.

## Usage

```
basic-http-rs [-a <address>] [-p <port>] [-c] [<path>]
```

| Flag | Default | Meaning |
|---|---|---|
| `-a`, `--address` | `0.0.0.0` | Address to bind |
| `-p`, `--port` | `8080` | Port to bind |
| `-c`, `--enable-compression` | off | Compress responses with brotli or gzip |
| `<path>` | `.` | Directory to serve |

Run it natively:

```sh
cargo build --release
./target/release/basic-http-rs -p 3000 ./dist
```

## Docker

The image is built from `scratch` and contains only the static musl binary,
about 2 MB (roughly 1 MB to pull). It runs as `nobody`. There is no shell
inside, so use `docker logs` for debugging.

```sh
docker build -t basic-http-rs .
docker run --rm -p 8080:80 -v ./dist:/srv:ro basic-http-rs
```

The container listens on port 80 and serves whatever is mounted at `/srv`.
Compression is enabled in the image. To run without it, override the entrypoint:

```sh
docker run --rm -p 8080:80 -v ./dist:/srv:ro --entrypoint /basic-http-rs basic-http-rs -p 80
```

## Limitations

- HTTP/1.1 only, no HTTPS.
- No caching headers.
- The whole file is read into memory before it is sent.

These are fine for serving a small site on a local network, which is all this
is meant for.
