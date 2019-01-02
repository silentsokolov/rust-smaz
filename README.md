# rust-smaz

rust-smaz is a pure Rust implementation of smaz - algorithm for compressing very short strings. See original [C implementation smaz by antirez](http://github.com/antirez/smaz) for information on smaz and the algorithm itself.


## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
smaz = "0.1.0"
```


## F.A.Q.

- Why HashMap?

Benchmark match statement and HashMap:

```
$ cargo bench

     Finished release [optimized] target(s) in 0.04s
     Running target/release/deps/smaz-07673d33e2751e17

running 2 tests
test tests::lookup_bench ... bench:          22 ns/iter (+/- 7)
test tests::map_bench    ... bench:          80 ns/iter (+/- 13)

test result: ok. 0 passed; 0 failed; 0 ignored; 2 measured; 0 filtered out
```
