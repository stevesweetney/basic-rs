# basic

Simplifying images

### Installation

[Precompiled binaries are available for linux and macOS](https://github.com/stevesweetney/basic-rs/releases)

### Command-line Usage

| Flag    | Default      | Description                            |
| ------- | ------------ | -------------------------------------- |
| `i`     | n/a          | input file                             |
| `o`     | 'output.png' | output file                            |
| `iters` | 1024         | number of times the algorithm will run |
| `g`     | false        | create a gif of the process            |
| `p`     | false        | add padding to the resulting quadrants |

#### (Build from source)

First, [install Rust](https://www.rust-lang.org/en-US/install.html) if you don't have it.

Next, clone this repository and navigate to the new directory

run

```
    cargo build --release
    .\target\release\basic --help
```

You can also run `cargo install --path .`
which will build a binary in release mode and place it in your
~/.cargo/bin folder
