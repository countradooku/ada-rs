//! Apples-to-apples parsing throughput comparison.

use std::hint::black_box;

use ada_rs::Url;
use ada_url_ffi::Url as AdaCppUrl;
use servo_url::Url as ServoUrl;

const URLS: &[&str] = &[
    "https://www.google.com/webhp?hl=en&source=hp",
    "https://en.wikipedia.org/wiki/Dog#Roles_with_humans",
    "https://www.tiktok.com/@aguyandagolden/video/7133277734310038830",
    "https://images-na.ssl-images-amazon.com/images/I/41Gc3C8UysL.css",
    "https://www.reddit.com/?after=t3_zvz1ze",
    "postgresql://other:9818274x1!!@localhost:5432/otherdb?connect_timeout=10",
    "http://192.168.1.1",
    "http://[2606:4700:4700::1111]",
    "https://www.7‑Eleven.com/Home/Privacy/Montréal",
];

const FAST_URLS: &[&str] = &[
    "https://www.google.com/webhp?hl=en&source=hp",
    "https://en.wikipedia.org/wiki/Dog#Roles_with_humans",
    "https://www.tiktok.com/@aguyandagolden/video/7133277734310038830",
    "https://images-na.ssl-images-amazon.com/images/I/41Gc3C8UysL.css",
    "https://www.reddit.com/?after=t3_zvz1ze",
];

const COMPLEX_URLS: &[&str] = &[
    "postgresql://other:9818274x1!!@localhost:5432/otherdb?connect_timeout=10",
    "http://192.168.1.1",
    "http://[2606:4700:4700::1111]",
    "https://www.7‑Eleven.com/Home/Privacy/Montréal",
];

fn main() {
    divan::main();
}

#[divan::bench]
fn ada_rs_parse(bencher: divan::Bencher) {
    bencher.bench_local(|| {
        for input in URLS {
            let parsed = Url::parse(black_box(input), None).unwrap();
            black_box(parsed.href());
        }
    });
}

#[divan::bench]
fn ada_cpp_parse(bencher: divan::Bencher) {
    bencher.bench_local(|| {
        for input in URLS {
            let parsed = AdaCppUrl::parse(black_box(input), None).unwrap();
            black_box(parsed.href());
        }
    });
}

#[divan::bench]
fn servo_url_parse(bencher: divan::Bencher) {
    bencher.bench_local(|| {
        for input in URLS {
            let parsed = ServoUrl::parse(black_box(input)).unwrap();
            black_box(parsed.as_str());
        }
    });
}

#[divan::bench]
fn ada_rs_fast(bencher: divan::Bencher) {
    bencher.bench_local(|| {
        for input in FAST_URLS {
            let parsed = Url::parse(black_box(input), None).unwrap();
            black_box(parsed.href());
        }
    });
}

#[divan::bench]
fn ada_cpp_fast(bencher: divan::Bencher) {
    bencher.bench_local(|| {
        for input in FAST_URLS {
            let parsed = AdaCppUrl::parse(black_box(input), None).unwrap();
            black_box(parsed.href());
        }
    });
}

#[divan::bench]
fn servo_url_fast(bencher: divan::Bencher) {
    bencher.bench_local(|| {
        for input in FAST_URLS {
            let parsed = ServoUrl::parse(black_box(input)).unwrap();
            black_box(parsed.as_str());
        }
    });
}

#[divan::bench]
fn ada_rs_complex(bencher: divan::Bencher) {
    bencher.bench_local(|| {
        for input in COMPLEX_URLS {
            let parsed = Url::parse(black_box(input), None).unwrap();
            black_box(parsed.href());
        }
    });
}

#[divan::bench]
fn ada_cpp_complex(bencher: divan::Bencher) {
    bencher.bench_local(|| {
        for input in COMPLEX_URLS {
            let parsed = AdaCppUrl::parse(black_box(input), None).unwrap();
            black_box(parsed.href());
        }
    });
}

#[divan::bench]
fn servo_url_complex(bencher: divan::Bencher) {
    bencher.bench_local(|| {
        for input in COMPLEX_URLS {
            let parsed = ServoUrl::parse(black_box(input)).unwrap();
            black_box(parsed.as_str());
        }
    });
}
