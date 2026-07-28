//! UTS #46 domain conversion helpers.

/// Converts a Unicode domain to its ASCII form using UTS #46 processing.
pub fn domain_to_ascii(domain: &str) -> Result<String, idna::Errors> {
    let converted = idna::domain_to_ascii_cow(domain.as_bytes(), idna::AsciiDenyList::URL)
        .map(|domain| domain.into_owned());
    if converted.is_err() && domain.is_ascii() && !domain.bytes().any(is_forbidden_domain_byte) {
        // The URL Standard deliberately preserves some ASCII labels that fail
        // strict UTS #46 validity checks (for example an invalid A-label).
        // Mapping is still required, so ASCII case is folded.
        return Ok(domain.to_ascii_lowercase());
    }
    converted
}

/// Converts an ASCII/Punycode domain to Unicode.
pub fn domain_to_unicode(domain: &str) -> (String, Result<(), idna::Errors>) {
    idna::domain_to_unicode(domain)
}

fn is_forbidden_domain_byte(byte: u8) -> bool {
    matches!(
        byte,
        0x00..=0x20
            | 0x7f
            | b'#'
            | b'%'
            | b'/'
            | b':'
            | b'<'
            | b'>'
            | b'?'
            | b'@'
            | b'['
            | b'\\'
            | b']'
            | b'^'
            | b'|'
    )
}
