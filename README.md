# basic

Simplifying images

### Command-line Usage

#### (Build from source)
First, [install Rust](https://www.rust-lang.org/en-US/install.html).

Next, clone this repository and navigate to the new directory

run

    cargo build --release
    .\target\release\basic --help

| Flag | Default | Description |
| --- | --- | --- |
| `i` | n/a | input file |
| `o` | 'output.png' | output file |
| `iters` | 1024 | number of times the algorithm will run |