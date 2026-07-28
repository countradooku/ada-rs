# Benchmark record

## Method

`benches/parse.rs` feeds identical preloaded inputs to `ada-rs`, C++ Ada
through the official `ada-url` Rust wrapper, and Servo's `url` crate. Every
iteration consumes the normalized serialization with `black_box`.

Run:

```console
cargo bench --features bench-ada --bench parse -- \
  --min-time 3 --sample-count 100
```

The corpus contains five normalized web URLs and four normalization-heavy URLs
(credentials, an opaque authority URL, IPv4, IPv6, and a Unicode domain/path).
This is a focused regression gate, not a universal performance claim.

## 2026-07-28 result

- CPU: Apple M5 Pro
- OS: macOS 26.5, arm64
- Rust: 1.97.1, LLVM 22.1.8
- Profile: `opt-level=3`, one codegen unit, fat LTO
- C++ Ada Rust wrapper: 4.0.0

| Parser / subset | Median |
| --- | ---: |
| ada-rs, all 9 | 1.041 µs |
| C++ Ada, all 9 | 1.124 µs |
| Servo `url`, all 9 | 1.666 µs |
| ada-rs, normalized 5 | 244.2 ns |
| C++ Ada, normalized 5 | 311.9 ns |
| ada-rs, complex 4 | 790.4 ns |
| C++ Ada, complex 4 | 790.4 ns |

On this run, `ada-rs` was 7.4% faster than C++ Ada over the whole corpus,
21.7% faster on the normalized subset, and tied at the timer's median
resolution on the complex subset.

`benches/wpt.rs` is deliberately less representative of ordinary traffic: it
parses the complete WPT fixture repeatedly, including malformed inputs intended
to exercise every error path. Run it with:

```console
cargo bench --features bench-ada --bench wpt -- \
  --min-time 3 --sample-count 100
```

| WPT subset | ada-rs | C++ Ada | Difference |
| --- | ---: | ---: | ---: |
| All cases | 124.5 µs | 125.2 µs | 0.6% faster |
| Valid cases | 106.1 µs | 109.6 µs | 3.2% faster |
| Invalid cases | 18.20 µs | 24.66 µs | 26.2% faster |
| With a base URL | 67.74 µs | 68.16 µs | 0.6% faster |
| Without a base URL | 57.70 µs | 57.45 µs | 0.4% slower |

The full-corpus and focused web-URL performance gates are met. The no-base
subset is effectively at parity and inside the 3% per-corpus tolerance.

For optimization work, `cargo run --release --features bench-ada --example
wpt-profile` prints syntax-family aggregates and the WPT cases with the largest
absolute Rust/C++ time difference.

Results should be remeasured on every target; medians can vary with CPU state.
