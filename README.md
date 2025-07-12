# US Distances RS

A Rust implementation of the Clojure US Distances program for calculating the
farthest distance from any point of US soil.

## Building

The following will build an optimized binary for the local architecture:

```bash

RUSTFLAGS='-C target-cpu=native' cargo build --release
```

## Benchmarking

```bash

hyperfine --warmup 30 ./target/release/usdist
```

## Notes

`src/lib.rs` uses `f32` by default, as it is a bit faster. Other versions
many not be able to do 32-bit floating point math, e.g. Go's std-lib uses
float64 by default for many `math` package functions.
If you want to more directly compare the two, simply `sed` `f32` for `f64`,
e.g. `sed -i '' 's/f32/f64/g' src/lib.rs` (on macOS).