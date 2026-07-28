//! Reports the WPT cases with the largest parsing-time difference from C++ Ada.

use std::{
    cmp::Reverse,
    collections::BTreeMap,
    hint::black_box,
    sync::LazyLock,
    time::{Duration, Instant},
};

use ada_rs::Url;
use ada_url_ffi::Url as AdaCppUrl;
use serde_json::Value;

const ITERATIONS: u32 = 2_000;
const DISPLAY_LIMIT: usize = 40;

struct Case {
    index: usize,
    input: String,
    base: Option<String>,
    failure: bool,
}

struct Measurement {
    index: usize,
    rust: Duration,
    cpp: Duration,
    input: String,
    base: bool,
    failure: bool,
    category: String,
}

#[derive(Default)]
struct Aggregate {
    count: usize,
    rust: Duration,
    cpp: Duration,
}

static CASES: LazyLock<Vec<Case>> = LazyLock::new(|| {
    let source = replace_unpaired_surrogate_escapes(include_str!("../tests/wpt/urltestdata.json"));
    serde_json::from_str::<Vec<Value>>(&source)
        .expect("valid WPT fixture")
        .into_iter()
        .enumerate()
        .filter_map(|(index, value)| {
            let object = value.as_object()?;
            Some(Case {
                index,
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
    let mut measurements = CASES
        .iter()
        .map(|case| Measurement {
            index: case.index,
            rust: measure_rust(case),
            cpp: measure_cpp(case),
            input: case.input.clone(),
            base: case.base.is_some(),
            failure: case.failure,
            category: category(case),
        })
        .collect::<Vec<_>>();

    let total_rust: Duration = measurements.iter().map(|item| item.rust).sum();
    let total_cpp: Duration = measurements.iter().map(|item| item.cpp).sum();
    println!(
        "iterations={ITERATIONS} cases={} rust={:.1}ns/case cpp={:.1}ns/case ratio={:.3}",
        measurements.len(),
        nanos_per_case(total_rust, measurements.len()),
        nanos_per_case(total_cpp, measurements.len()),
        total_rust.as_secs_f64() / total_cpp.as_secs_f64()
    );
    let mut aggregates = BTreeMap::<String, Aggregate>::new();
    for item in &measurements {
        let aggregate = aggregates.entry(item.category.clone()).or_default();
        aggregate.count += 1;
        aggregate.rust += item.rust;
        aggregate.cpp += item.cpp;
    }
    println!("\ncategory aggregates:");
    for (category, aggregate) in aggregates {
        println!(
            "{category:35} count={:>3} rust={:>7.1}ns cpp={:>7.1}ns ratio={:.3}",
            aggregate.count,
            nanos_per_case(aggregate.rust, aggregate.count),
            nanos_per_case(aggregate.cpp, aggregate.count),
            aggregate.rust.as_secs_f64() / aggregate.cpp.as_secs_f64(),
        );
    }
    println!("\nlargest absolute deltas:");
    measurements.sort_unstable_by_key(|item| Reverse(item.rust.saturating_sub(item.cpp)));
    for item in measurements.iter().take(DISPLAY_LIMIT) {
        let rust = nanos_per_iteration(item.rust);
        let cpp = nanos_per_iteration(item.cpp);
        println!(
            "#{:<4} rust={:>8.1}ns cpp={:>8.1}ns delta={:>8.1}ns {:4} {:7} {:?}",
            item.index,
            rust,
            cpp,
            rust - cpp,
            if item.base { "base" } else { "none" },
            if item.failure { "failure" } else { "success" },
            abbreviate(&item.input),
        );
    }
}

fn category(case: &Case) -> String {
    let input = case
        .input
        .trim_matches(|character: char| character <= '\u{20}');
    let base = case.base.as_deref().map_or("no-base", |base| {
        if base
            .trim_start_matches(|character: char| character <= '\u{20}')
            .to_ascii_lowercase()
            .starts_with("file:")
        {
            "file-base"
        } else {
            "web-base"
        }
    });
    let syntax = if input
        .get(..5)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("file:"))
    {
        "file-absolute"
    } else if ["http:", "https:", "ws:", "wss:", "ftp:"]
        .iter()
        .any(|scheme| {
            input
                .get(..scheme.len())
                .is_some_and(|prefix| prefix.eq_ignore_ascii_case(scheme))
        })
    {
        "special-absolute"
    } else if input.find(':').is_some_and(|colon| {
        input[..colon]
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_alphabetic())
    }) {
        "other-absolute"
    } else if input.starts_with("//") || input.starts_with("\\\\") {
        "authority-reference"
    } else if input.starts_with('/') || input.starts_with('\\') {
        "absolute-path"
    } else if input.starts_with('?') {
        "query"
    } else if input.starts_with('#') {
        "fragment"
    } else if input.is_empty() {
        "empty"
    } else {
        "relative"
    };
    format!(
        "{}|{base}|{syntax}",
        if case.failure { "failure" } else { "success" }
    )
}

fn measure_rust(case: &Case) -> Duration {
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let base = case
            .base
            .as_deref()
            .and_then(|base| Url::parse(black_box(base), None).ok());
        if let Ok(url) = Url::parse(black_box(&case.input), base.as_ref()) {
            black_box(url.href());
        }
    }
    start.elapsed()
}

fn measure_cpp(case: &Case) -> Duration {
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let base = case
            .base
            .as_deref()
            .and_then(|base| AdaCppUrl::parse(black_box(base), None).ok());
        if let Ok(url) = AdaCppUrl::parse(
            black_box(case.input.as_str()),
            base.as_ref().map(AdaCppUrl::href),
        ) {
            black_box(url.href());
        }
    }
    start.elapsed()
}

fn nanos_per_iteration(duration: Duration) -> f64 {
    duration.as_nanos() as f64 / f64::from(ITERATIONS)
}

fn nanos_per_case(duration: Duration, cases: usize) -> f64 {
    duration.as_nanos() as f64 / f64::from(ITERATIONS) / cases as f64
}

fn abbreviate(input: &str) -> String {
    const LIMIT: usize = 100;
    if input.chars().count() <= LIMIT {
        return input.escape_debug().to_string();
    }
    let prefix = input.chars().take(LIMIT).collect::<String>();
    format!("{}…", prefix.escape_debug())
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
