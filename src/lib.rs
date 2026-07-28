//! A memory-safe, high-performance WHATWG URL parser.
//!
//! The primary type is [`Url`]. It stores one normalized serialization and
//! compact byte offsets, so component access is allocation-free.

#![forbid(unsafe_code)]

mod components;
mod encoding;
mod error;
mod fast_path;
mod idna;
mod search_params;
mod url;
#[cfg(feature = "url-pattern")]
mod url_pattern;

pub use components::{Components, HostType, SchemeType};
pub use encoding::{PercentEncodeSet, percent_decode, percent_encode};
pub use error::{ParseError, ParseErrorKind};
pub use idna::{domain_to_ascii, domain_to_unicode};
pub use search_params::UrlSearchParams;
#[cfg(feature = "std")]
pub use url::href_from_file;
pub use url::{Url, can_parse, get_max_input_length, parse, set_max_input_length};
#[cfg(feature = "url-pattern")]
pub use url_pattern::{
    RegexSyntax, UrlPattern, UrlPatternComponentResult, UrlPatternError, UrlPatternInit,
    UrlPatternMatchInput, UrlPatternOptions, UrlPatternResult,
};
