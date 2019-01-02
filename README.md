# rust-smaz

[![Build Status](https://travis-ci.org/silentsokolov/rust-smaz.svg?branch=master)](https://travis-ci.org/silentsokolov/rust-smaz)
[![Crate](https://img.shields.io/crates/v/smaz.svg)](https://crates.io/crates/smaz)
[![Docs](https://docs.rs/rand/badge.svg)](https://docs.rs/smaz)

rust-smaz is a pure Rust implementation of smaz - algorithm for compressing very short strings. See original [C implementation smaz by antirez](http://github.com/antirez/smaz) for information on smaz and the algorithm itself.


## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
smaz = "0.1.0"
```

## Quick start

```rust
extern crate smaz;

use smaz::{compress,decompress};

fn main() {
    let s = "string";

    let compressed = compress(&s.as_bytes());
    println!("compress bytes: {:?}", &compressed);

    let decompressed = decompress(&compressed).unwrap();
    let origin = str::from_utf8(&decompressed).unwrap();
    assert_eq!(s, origin);
}
```
