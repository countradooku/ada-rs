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

On an Apple M5 Pro, the nine-URL real-web comparison corpus measured a
1.041 µs median for `ada-rs`, 1.124 µs for C++ Ada, and 1.666 µs for `url`.
The complete malformed-heavy WPT stress corpus measured 124.5 µs for `ada-rs`
and 125.2 µs for C++ Ada. Results and the methodology are recorded in
[`docs/BENCHMARKS.md`](docs/BENCHMARKS.md).

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
