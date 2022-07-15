# Developer Guide

## Prerequisite

- Rust toolchain installed via <https://rustup.rs/>. Tested with 1.62.0.
- Node.js. Tested with 16.x LTS and 18.x.
- PostgreSQL. Tested with 14. On macOS with Homebrew installed, simply `brew install postgresql`, and follow the
  instructions in the installation caveats.

## Build from Source

```bash
git clone https://github.com/caizixian/notegraf.git
cd notegraf/notegraf-web
npm install
cargo build
```

## Setup Development Server

First, you need a PostgreSQL server running on the same host.
On macOS with Homebrew installed, simply `brew install postgresql`,

Under `notegraf/notegraf-web`, create the following two files.

```
notegraf
├── ...
└── notegraf-web
    ├── ...
    ├── .proxyrc.js
    └── configuration.yml
```

```javascript
// .proxyrc.js
const {createProxyMiddleware} = require("http-proxy-middleware");

module.exports = function (app) {
    app.use(
        createProxyMiddleware(["/**", "!/", "!**/*.html", "!**/*.js", "!*.css", "!**/*.css", "!**/*.map", "!**/*.ttf", "!**/*.woff", "!**/*.woff2"], {
            // localhost on macOS can also resolve to ::1
            // python3 -c 'import socket; print(socket.getaddrinfo("localhost", 8000))'
            // https://stackoverflow.com/questions/15227154/inexplicable-node-js-http-throwing-connect-econnrefused-ipv6
            target: "http://localhost:8000/",
        })
    );
};
```

```yaml
# configuration.yml
port: 8000
notestoretype: "PostgreSQL"
debug: true
database:
  host: localhost
  port: 5432
  name: notegraf
```

Finally, open two terminal windows.
In the first window, run `cargo run` under `notegraf/notegraf-web`, and in the other window, run `npm start`.
Your browser should automatically navigate to <http://localhost:1234>.

## Project Structure

The repo is set up as a cargo workspace with two crates.

```
notegraf
├── ...
├── notegraf     <- Notegraf core data types and persistence logics
└── notegraf-web <- Notegraf HTTP frontend and web UI 
```

## Pre-commit

`cargo check && cargo test && cargo clippy && cargo fmt`.
