//! Public API integration tests.

use ada_rs::{HostType, SchemeType, Url, UrlSearchParams, can_parse, domain_to_ascii};

#[test]
fn readme_example() {
    let url = Url::parse("https://www.7‑Eleven.com/Home/Privacy/Montréal", None).unwrap();
    assert_eq!(
        url.href(),
        "https://www.xn--7eleven-506c.com/Home/Privacy/Montr%C3%A9al"
    );
    assert_eq!(url.scheme_type(), SchemeType::Https);
    assert_eq!(url.host_type(), HostType::Domain);
}

#[test]
fn validation_and_idna() {
    assert!(can_parse("https://example.com/", None));
    assert!(can_parse("/relative", Some("https://example.com/base")));
    assert!(!can_parse("/relative", None));
    assert_eq!(
        domain_to_ascii("www.7‑Eleven.com").unwrap(),
        "www.xn--7eleven-506c.com"
    );
}

#[test]
fn search_params_api() {
    let mut params = UrlSearchParams::new("a=1&a=2");
    params.append("b", "hello world");
    assert_eq!(params.to_string(), "a=1&a=2&b=hello+world");
}

#[test]
fn url_and_search_params_synchronize_explicitly() {
    let mut url = Url::parse("https://example.com/?a=1&a=2", None).unwrap();
    let mut params = url.search_params();
    params.set("a", "3");
    params.append("snow", "☃");
    url.set_search_params(&params).unwrap();
    assert_eq!(url.search(), "?a=3&snow=%E2%98%83");
}

#[cfg(feature = "serde")]
#[test]
fn serde_uses_normalized_strings() {
    let url = Url::parse("https://EXAMPLE.com/a b", None).unwrap();
    let json = serde_json::to_string(&url).unwrap();
    assert_eq!(json, "\"https://example.com/a%20b\"");
    assert_eq!(serde_json::from_str::<Url>(&json).unwrap(), url);

    let params = UrlSearchParams::new("a=b+c");
    let json = serde_json::to_string(&params).unwrap();
    assert_eq!(
        serde_json::from_str::<UrlSearchParams>(&json).unwrap(),
        params
    );
}
