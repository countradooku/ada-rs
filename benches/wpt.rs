//! Broad WPT corpus comparison with the C++ Ada implementation.

use std::hint::black_box;
use std::sync::LazyLock;

use ada_rs::Url;
use ada_url_ffi::Url as AdaCppUrl;
use serde_json::Value;

struct Case {
    input: String,
    base: Option<String>,
    failure: bool,
}

static CASES: LazyLock<Vec<Case>> = LazyLock::new(|| {
    let source = replace_unpaired_surrogate_escapes(include_str!("../tests/wpt/urltestdata.json"));
    serde_json::from_str::<Vec<Value>>(&source)
        .unwrap()
        .into_iter()
        .filter_map(|value| {
            let object = value.as_object()?;
            Some(Case {
                input: object.get("input")?.as_str()?.to_owned(),
                base: object
                    .get("base")
                    .and_then(Value::as_str)
                    .map(str::to_owned),
                failure: object
                    .get("failure")
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
            })
        })
        .collect()
});

fn main() {
    divan::main();
}

#[divan::bench]
fn ada_rs_wpt(bencher: divan::Bencher) {
    bencher.bench_local(|| {
        let mut consumed = 0;
        for case in CASES.iter() {
            let base = case
                .base
                .as_deref()
                .and_then(|base| Url::parse(base, None).ok());
            if let Ok(url) = Url::parse(black_box(&case.input), base.as_ref()) {
                consumed += black_box(url.href()).len();
            }
        }
        black_box(consumed)
    });
}

#[divan::bench]
fn ada_cpp_wpt(bencher: divan::Bencher) {
    bencher.bench_local(|| {
        let mut consumed = 0;
        for case in CASES.iter() {
            let base = case
                .base
                .as_deref()
                .and_then(|base| AdaCppUrl::parse(base, None).ok());
            if let Ok(url) = AdaCppUrl::parse(
                black_box(case.input.as_str()),
                base.as_ref().map(AdaCppUrl::href),
            ) {
                consumed += black_box(url.href()).len();
            }
        }
        black_box(consumed)
    });
}

#[divan::bench]
fn ada_rs_wpt_valid(bencher: divan::Bencher) {
    bench_rust(bencher, |case| !case.failure);
}

#[divan::bench]
fn ada_cpp_wpt_valid(bencher: divan::Bencher) {
    bench_cpp(bencher, |case| !case.failure);
}

#[divan::bench]
fn ada_rs_wpt_invalid(bencher: divan::Bencher) {
    bench_rust(bencher, |case| case.failure);
}

#[divan::bench]
fn ada_cpp_wpt_invalid(bencher: divan::Bencher) {
    bench_cpp(bencher, |case| case.failure);
}

#[divan::bench]
fn ada_rs_wpt_with_base(bencher: divan::Bencher) {
    bench_rust(bencher, |case| case.base.is_some());
}

#[divan::bench]
fn ada_cpp_wpt_with_base(bencher: divan::Bencher) {
    bench_cpp(bencher, |case| case.base.is_some());
}

#[divan::bench]
fn ada_rs_wpt_without_base(bencher: divan::Bencher) {
    bench_rust(bencher, |case| case.base.is_none());
}

#[divan::bench]
fn ada_cpp_wpt_without_base(bencher: divan::Bencher) {
    bench_cpp(bencher, |case| case.base.is_none());
}

fn bench_rust(bencher: divan::Bencher, include: fn(&Case) -> bool) {
    bencher.bench_local(|| {
        let mut consumed = 0;
        for case in CASES.iter().filter(|case| include(case)) {
            let base = case
                .base
                .as_deref()
                .and_then(|base| Url::parse(base, None).ok());
            if let Ok(url) = Url::parse(black_box(&case.input), base.as_ref()) {
                consumed += black_box(url.href()).len();
            }
        }
        black_box(consumed)
    });
}

fn bench_cpp(bencher: divan::Bencher, include: fn(&Case) -> bool) {
    bencher.bench_local(|| {
        let mut consumed = 0;
        for case in CASES.iter().filter(|case| include(case)) {
            let base = case
                .base
                .as_deref()
                .and_then(|base| AdaCppUrl::parse(base, None).ok());
            if let Ok(url) = AdaCppUrl::parse(
                black_box(case.input.as_str()),
                base.as_ref().map(AdaCppUrl::href),
            ) {
                consumed += black_box(url.href()).len();
            }
        }
        black_box(consumed)
    });
}

fn replace_unpaired_surrogate_escapes(source: &str) -> String {
    let bytes = source.as_bytes();
    let mut output = String::with_capacity(source.len());
    let mut index = 0;
    while index < bytes.len() {
        let is_escape =
            index + 6 <= bytes.len() && bytes[index] == b'\\' && bytes[index + 1] == b'u';
        if !is_escape {
            let character = source[index..].chars().next().expect("valid UTF-8 fixture");
            output.push(character);
            index += character.len_utf8();
            continue;
        }
        let Some(code_unit) = parse_hex_u16(&bytes[index + 2..index + 6]) else {
            output.push('\\');
            index += 1;
            continue;
        };
        if (0xd800..=0xdbff).contains(&code_unit) {
            let paired = index + 12 <= bytes.len()
                && bytes[index + 6] == b'\\'
                && bytes[index + 7] == b'u'
                && parse_hex_u16(&bytes[index + 8..index + 12])
                    .is_some_and(|low| (0xdc00..=0xdfff).contains(&low));
            if paired {
                output.push_str(&source[index..index + 12]);
                index += 12;
            } else {
                output.push_str("\\uFFFD");
                index += 6;
            }
        } else if (0xdc00..=0xdfff).contains(&code_unit) {
            output.push_str("\\uFFFD");
            index += 6;
        } else {
            output.push_str(&source[index..index + 6]);
            index += 6;
        }
    }
    output
}

fn parse_hex_u16(input: &[u8]) -> Option<u16> {
    input.iter().try_fold(0_u16, |value, byte| {
        let digit = match byte {
            b'0'..=b'9' => u16::from(byte - b'0'),
            b'a'..=b'f' => u16::from(byte - b'a' + 10),
            b'A'..=b'F' => u16::from(byte - b'A' + 10),
            _ => return None,
        };
        Some(value * 16 + digit)
    })
}
