//! Pattern compilation: turn a user string into a compiled regular expression.

use regex::{Regex, RegexBuilder};

/// Compile `pattern` into a [`Regex`], optionally case-insensitive.
///
/// The resulting regex is intended to be built **once** and reused for every
/// file in a search run.
pub fn compile_pattern(pattern: &str, ignore_case: bool) -> Result<Regex, regex::Error> {
    let mut builder = RegexBuilder::new(pattern);
    builder.case_insensitive(ignore_case);
    builder.build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compiles_literal() {
        let re = compile_pattern("hello", false).unwrap();
        assert!(re.is_match("say hello"));
        assert!(!re.is_match("HELLO"));
    }

    #[test]
    fn compiles_case_insensitive() {
        let re = compile_pattern("hello", true).unwrap();
        assert!(re.is_match("HELLO"));
        assert!(re.is_match("HeLLo"));
    }

    #[test]
    fn rejects_invalid_regex() {
        assert!(compile_pattern("(", false).is_err());
    }

    #[test]
    fn supports_metacharacters() {
        let re = compile_pattern(r"^foo\d+$", false).unwrap();
        assert!(re.is_match("foo42"));
        assert!(!re.is_match("xfoo42"));
    }
}
