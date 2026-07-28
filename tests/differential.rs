//! Deterministic differential testing against the pinned C++ Ada wrapper.

#![cfg(feature = "bench-ada")]

use ada_rs::{Url, UrlSearchParams};
use ada_url_ffi::{Url as AdaUrl, UrlSearchParams as AdaSearchParams};

const BASES: &[Option<&str>] = &[
    None,
    Some("https://base.example/a/b?old#fragment"),
    Some("file:///C:/work/project/"),
    Some("foo://user@example.test/root/"),
];

#[test]
fn generated_inputs_match_cpp_ada() {
    let mut random = Lcg(0xada5_5afe_cafe_f00d);
    for case in 0..10_000 {
        let input = generate_input(&mut random);
        let base = BASES[random.index(BASES.len())];
        if base.is_none() && !has_scheme(&input) {
            continue;
        }

        let rust_base = base.map(|base| Url::parse(base, None).unwrap());
        let rust = Url::parse(&input, rust_base.as_ref());
        let cpp = AdaUrl::parse(input.as_str(), base);
        assert_eq!(
            rust.is_ok(),
            cpp.is_ok(),
            "case {case}: validity differs for input={input:?}, base={base:?}"
        );
        if let (Ok(rust), Ok(cpp)) = (rust, cpp) {
            assert_eq!(
                rust.href(),
                cpp.href(),
                "case {case}: serialization differs for input={input:?}, base={base:?}"
            );
            assert!(rust.validate(), "case {case}: invalid internal offsets");
        }
    }
}

#[test]
fn generated_search_params_match_cpp_ada() {
    let mut random = Lcg(0x5ea2_cafe_1234_5678);
    for case in 0..5_000 {
        let input = format!(
            "{}={}&{}={}&{}",
            segment(&mut random),
            segment(&mut random),
            segment(&mut random),
            segment(&mut random),
            segment(&mut random)
        );
        let mut rust = UrlSearchParams::new(&input);
        let mut cpp = AdaSearchParams::parse(&input).unwrap();
        assert_eq!(
            rust.to_string(),
            cpp.to_string(),
            "case {case}: initial serialization for {input:?}"
        );

        let key = segment(&mut random);
        let value = segment(&mut random);
        rust.append(key.clone(), value.clone());
        cpp.append(&key, &value);
        rust.set("shared", value.clone());
        cpp.set("shared", &value);
        if case % 2 == 0 {
            rust.delete_value(&key, &value);
            cpp.remove(&key, &value);
        }
        rust.sort();
        cpp.sort();
        assert_eq!(
            rust.to_string(),
            cpp.to_string(),
            "case {case}: mutated serialization for {input:?}"
        );
        assert_eq!(rust.len(), cpp.len(), "case {case}: pair count");
    }
}

fn generate_input(random: &mut Lcg) -> String {
    if random.index(5) == 0 {
        return generate_reference(random);
    }
    let scheme = ["http", "https", "ftp", "ws", "wss", "foo", "file"][random.index(7)];
    if scheme == "file" {
        return format!("file:///C:/{}/{}", segment(random), segment(random));
    }
    let separator = if random.index(8) == 0 { r":\\" } else { "://" };
    let credentials = if random.index(4) == 0 {
        format!("{}:{}@", segment(random), segment(random))
    } else {
        String::new()
    };
    let host = [
        "example.com",
        "EXAMPLE.test",
        "127.0.0.1",
        "[2001:db8::1]",
        "münich.example",
        "localhost",
    ][random.index(6)];
    let port = if random.index(4) == 0 {
        format!(":{}", [80, 443, 8080, 5432][random.index(4)])
    } else {
        String::new()
    };
    let path = format!("/{}/{}", segment(random), segment(random));
    let query = (random.index(3) == 0).then(|| format!("?q={}&n={}", segment(random), random.0));
    let fragment = (random.index(4) == 0).then(|| format!("#{}", segment(random)));
    format!(
        "{scheme}{separator}{credentials}{host}{port}{path}{}{}",
        query.as_deref().unwrap_or_default(),
        fragment.as_deref().unwrap_or_default()
    )
}

fn generate_reference(random: &mut Lcg) -> String {
    match random.index(6) {
        0 => format!("../{}/{}", segment(random), segment(random)),
        1 => format!("/{}/./{}/../x", segment(random), segment(random)),
        2 => format!("?{}={}", segment(random), segment(random)),
        3 => format!("#{}", segment(random)),
        4 => format!("//example.com/{}/{}", segment(random), segment(random)),
        _ => segment(random),
    }
}

fn segment(random: &mut Lcg) -> String {
    const PIECES: &[&str] = &[
        "alpha",
        "b",
        "42",
        "-",
        "_",
        "~",
        "a b",
        "caf%C3%A9",
        "é",
        "%2e",
        "x+y",
    ];
    let mut output = String::new();
    for _ in 0..=random.index(3) {
        output.push_str(PIECES[random.index(PIECES.len())]);
    }
    output
}

fn has_scheme(input: &str) -> bool {
    input
        .find(':')
        .is_some_and(|colon| colon > 0 && !input[..colon].contains(['/', '?', '#']))
}

struct Lcg(u64);

impl Lcg {
    fn index(&mut self, upper: usize) -> usize {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (self.0 as usize) % upper
    }
}
