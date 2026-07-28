# ada-rs

`ada-rs` is a memory-safe Rust port of [Ada](https://github.com/ada-url/ada),
the WHATWG URL parser used by Node.js. It targets three properties
simultaneously:

- WHATWG and UTS #46 conformance;
- memory safety, with unsafe code forbidden in the library;
- parse throughput and allocation behavior at least as good as C++ Ada.

The crate provides:

- a single-buffer `Url` with compact `u32` offsets and allocation-free
  component access;
- WHATWG parsing, base resolution, file/opaque URLs, IPv4/IPv6, UTS #46 IDNA,
  origins, transactional setters, and configurable length limits;
- `UrlSearchParams`, percent-encoding helpers, optional serde support, and
  URLPattern behind the `url-pattern` feature;
- runtime-dispatched SIMD delimiter scans through `memchr`, with conservative
  native fast paths and a memory-safe Rust correctness fallback.

```rust
use ada_rs::Url;

let url = Url::parse("https://www.7‑Eleven.com/Home/Privacy/Montréal", None)?;
assert_eq!(
    url.href(),
    "https://www.xn--7eleven-506c.com/Home/Privacy/Montr%C3%A9al"
);
# Ok::<(), ada_rs::ParseError>(())
```

## Benchmarks

Measured on an Apple M5 Pro running macOS 26.5 (arm64), using Rust 1.97.1,
LLVM 22.1.8, `opt-level=3`, one codegen unit, and fat LTO. Each parser receives
the same preloaded inputs and the benchmark consumes the normalized
serialization with `black_box`.

The focused corpus contains five already-normalized web URLs and four
normalization-heavy URLs covering credentials, an opaque authority URL, IPv4,
IPv6, and a Unicode domain/path.

| Parser | All 9 | Normalized 5 | Complex 4 |
| --- | ---: | ---: | ---: |
| **ada-rs** | **1.041 µs** | **244.2 ns** | **790.4 ns** |
| C++ Ada 4.0.0 | 1.124 µs | 311.9 ns | 790.4 ns |
| Servo `url` | 1.666 µs | 708.4 ns | 916.4 ns |

Over the full corpus, `ada-rs` was 7.4% faster than C++ Ada and 37.5% faster
than Servo `url`. It was 21.7% faster than C++ Ada on the normalized subset and
tied at the timer's median resolution on the complex subset.

The broader benchmark repeatedly parses all 891 cases from the pinned WHATWG
URL fixture, including malformed inputs designed to exercise error paths:

| WPT subset | ada-rs | C++ Ada | Difference |
| --- | ---: | ---: | ---: |
| All cases | **124.5 µs** | 125.2 µs | 0.6% faster |
| Valid cases | **106.1 µs** | 109.6 µs | 3.2% faster |
| Invalid cases | **18.20 µs** | 24.66 µs | 26.2% faster |
| With a base URL | **67.74 µs** | 68.16 µs | 0.6% faster |
| Without a base URL | 57.70 µs | **57.45 µs** | 0.4% slower |

Reproduce the measurements with:

```console
cargo bench --features bench-ada --bench parse -- \
  --min-time 3 --sample-count 100
cargo bench --features bench-ada --bench wpt -- \
  --min-time 3 --sample-count 100
```

These are local measurements, not universal performance claims. Results should
be remeasured on each target CPU and toolchain. Full methodology and raw
environment details are in [`docs/BENCHMARKS.md`](docs/BENCHMARKS.md).

## Development

```console
cargo test --all-features
cargo clippy --all-targets --all-features -- -D warnings
cargo bench --features bench-ada --bench parse
cargo check --manifest-path fuzz/Cargo.toml
```

The pinned upstream corpus currently executes 4,350 URL, setter, IDNA,
percent-encoding, and URLPattern cases. Deterministic differential tests add
10,000 URL cases and 5,000 SearchParams mutation sequences against C++ Ada.

See [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) for invariants, fallback
boundaries, and the optimization policy.

## Upstream attribution

This project is an independent Rust port inspired by
[`ada-url/ada`](https://github.com/ada-url/ada). Conformance fixtures are pinned
to a specific upstream revision and recorded in
[`tests/wpt/UPSTREAM`](tests/wpt/UPSTREAM).

## Features

- `std` (default): enables filesystem-path to `file:` URL conversion.
- `serde`: normalized string serialization for `Url` and `UrlSearchParams`.
- `url-pattern`: WHATWG URLPattern with a linear-time Rust regex provider.
- `bench-ada`: builds the C++ Ada wrapper for differential tests and benches.

## License

MIT or Apache-2.0, at your option.
