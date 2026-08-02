## Build

To build Constellation, you will need a new version of Rust using `rustup`. On Debian based distros at least these:
```sh
sudo apt-get update && sudo apt-get install -y pkg-config libxkbcommon-dev libx11-dev libxcb1-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libglib2.0-dev
```

```bash
cargo build --release
```

To run the application:

```bash
cargo run --release
```

## Issues & PR's

You're welcome to open feature request or report problems through Issues. PR are also welcome. Just have a human responsible for each one.
