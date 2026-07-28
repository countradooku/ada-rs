#![no_main]

use ada_rs::Url;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(input) = std::str::from_utf8(data) else {
        return;
    };
    let Ok(mut url) = Url::parse("https://user:pass@example.com:8443/a?b#c", None) else {
        unreachable!();
    };

    for setter in [
        Url::set_protocol,
        Url::set_username,
        Url::set_password,
        Url::set_host,
        Url::set_hostname,
        Url::set_port,
        Url::set_pathname,
        Url::set_search,
        Url::set_hash,
    ] {
        let before = url.clone();
        if setter(&mut url, input).is_err() {
            assert_eq!(url, before);
        }
        assert!(url.validate());
    }
});
