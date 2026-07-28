//! WHATWG URLSearchParams.

use core::fmt;

/// An ordered list of URL query name/value pairs.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct UrlSearchParams {
    pairs: Vec<(String, String)>,
}

impl UrlSearchParams {
    /// Parses an `application/x-www-form-urlencoded` query.
    #[must_use]
    pub fn new(input: &str) -> Self {
        if input.len() > crate::url::get_max_input_length() as usize {
            return Self::default();
        }
        let input = input.strip_prefix('?').unwrap_or(input);
        let pairs = servo_url::form_urlencoded::parse(input.as_bytes())
            .map(|(name, value)| (name.into_owned(), value.into_owned()))
            .collect();
        Self { pairs }
    }

    /// Returns the number of pairs.
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.pairs.len()
    }

    /// Returns whether there are no pairs.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.pairs.is_empty()
    }

    /// Appends a pair.
    pub fn append(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.pairs.push((name.into(), value.into()));
    }

    /// Removes every pair with `name`.
    pub fn delete(&mut self, name: &str) {
        self.pairs.retain(|(key, _)| key != name);
    }

    /// Removes every pair matching both `name` and `value`.
    pub fn delete_value(&mut self, name: &str, value: &str) {
        self.pairs
            .retain(|(key, candidate)| key != name || candidate != value);
    }

    /// Returns the first value associated with `name`.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&str> {
        self.pairs
            .iter()
            .find_map(|(key, value)| (key == name).then_some(value.as_str()))
    }

    /// Returns all values associated with `name`.
    pub fn get_all<'a>(&'a self, name: &'a str) -> impl Iterator<Item = &'a str> + 'a {
        self.pairs
            .iter()
            .filter_map(move |(key, value)| (key == name).then_some(value.as_str()))
    }

    /// Returns whether at least one pair has `name`.
    #[must_use]
    pub fn has(&self, name: &str) -> bool {
        self.pairs.iter().any(|(key, _)| key == name)
    }

    /// Returns whether a pair matches both `name` and `value`.
    #[must_use]
    pub fn has_value(&self, name: &str, value: &str) -> bool {
        self.pairs
            .iter()
            .any(|(key, candidate)| key == name && candidate == value)
    }

    /// Replaces the first value for `name` and removes later duplicates.
    pub fn set(&mut self, name: impl Into<String>, value: impl Into<String>) {
        let name = name.into();
        let value = value.into();
        let mut found = false;
        self.pairs.retain_mut(|(key, candidate)| {
            if key != &name {
                return true;
            }
            if found {
                return false;
            }
            *candidate = value.clone();
            found = true;
            true
        });
        if !found {
            self.pairs.push((name, value));
        }
    }

    /// Sorts pairs stably by name using UTF-16 code units.
    pub fn sort(&mut self) {
        self.pairs
            .sort_by(|(left, _), (right, _)| left.encode_utf16().cmp(right.encode_utf16()));
    }

    /// Iterates over pairs in insertion order.
    pub fn iter(&self) -> impl ExactSizeIterator<Item = (&str, &str)> {
        self.pairs
            .iter()
            .map(|(name, value)| (name.as_str(), value.as_str()))
    }

    /// Iterates over names in insertion order.
    pub fn keys(&self) -> impl ExactSizeIterator<Item = &str> {
        self.pairs.iter().map(|(name, _)| name.as_str())
    }

    /// Iterates over values in insertion order.
    pub fn values(&self) -> impl ExactSizeIterator<Item = &str> {
        self.pairs.iter().map(|(_, value)| value.as_str())
    }

    /// Returns the first pair.
    #[must_use]
    pub fn front(&self) -> Option<(&str, &str)> {
        self.pairs
            .first()
            .map(|(name, value)| (name.as_str(), value.as_str()))
    }

    /// Returns the last pair.
    #[must_use]
    pub fn back(&self) -> Option<(&str, &str)> {
        self.pairs
            .last()
            .map(|(name, value)| (name.as_str(), value.as_str()))
    }

    /// Returns a pair by insertion-order index.
    #[must_use]
    pub fn get_index(&self, index: usize) -> Option<(&str, &str)> {
        self.pairs
            .get(index)
            .map(|(name, value)| (name.as_str(), value.as_str()))
    }

    /// Replaces all pairs by parsing a new query.
    ///
    /// As in Ada, an input above the configured maximum length clears the
    /// collection.
    pub fn reset(&mut self, input: &str) {
        *self = Self::new(input);
    }
}

impl fmt::Display for UrlSearchParams {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut serializer = servo_url::form_urlencoded::Serializer::new(String::new());
        serializer.extend_pairs(self.iter());
        formatter.write_str(&serializer.finish())
    }
}

impl core::str::FromStr for UrlSearchParams {
    type Err = core::convert::Infallible;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        Ok(Self::new(input))
    }
}

impl<'a> IntoIterator for &'a UrlSearchParams {
    type Item = (&'a str, &'a str);
    type IntoIter = core::iter::Map<
        core::slice::Iter<'a, (String, String)>,
        fn(&(String, String)) -> (&str, &str),
    >;

    fn into_iter(self) -> Self::IntoIter {
        fn as_pair(pair: &(String, String)) -> (&str, &str) {
            (&pair.0, &pair.1)
        }
        self.pairs.iter().map(as_pair)
    }
}

impl IntoIterator for UrlSearchParams {
    type Item = (String, String);
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.pairs.into_iter()
    }
}

impl<K, V> FromIterator<(K, V)> for UrlSearchParams
where
    K: Into<String>,
    V: Into<String>,
{
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        Self {
            pairs: iter
                .into_iter()
                .map(|(name, value)| (name.into(), value.into()))
                .collect(),
        }
    }
}

impl<K, V> Extend<(K, V)> for UrlSearchParams
where
    K: Into<String>,
    V: Into<String>,
{
    fn extend<T: IntoIterator<Item = (K, V)>>(&mut self, iter: T) {
        self.pairs.extend(
            iter.into_iter()
                .map(|(name, value)| (name.into(), value.into())),
        );
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for UrlSearchParams {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for UrlSearchParams {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let input = <String as serde::Deserialize>::deserialize(deserializer)?;
        Ok(Self::new(&input))
    }
}

#[cfg(test)]
mod tests {
    use super::UrlSearchParams;

    #[test]
    fn parses_and_serializes() {
        let mut params = UrlSearchParams::new("?a=b+c&a=d&empty");
        assert_eq!(params.get("a"), Some("b c"));
        assert_eq!(params.get_all("a").collect::<Vec<_>>(), ["b c", "d"]);
        params.set("a", "x");
        params.append("snow", "☃");
        assert_eq!(params.to_string(), "a=x&empty=&snow=%E2%98%83");
    }

    #[test]
    fn sort_is_stable() {
        let mut params = UrlSearchParams::new("z=1&a=first&a=second");
        params.sort();
        assert_eq!(params.to_string(), "a=first&a=second&z=1");
    }
}
