#![no_main]

use ada_rs::{Url, UrlSearchParams};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(input) = std::str::from_utf8(data) else {
        return;
    };
    if let Ok(url) = Url::parse(input, None) {
        assert!(url.validate());
        assert_eq!(Url::parse(url.href(), None).unwrap(), url);
        let params = url.search_params();
        let serialized = params.to_string();
        assert_eq!(UrlSearchParams::new(&serialized), params);
    }
});
