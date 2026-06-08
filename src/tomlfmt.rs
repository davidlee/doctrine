// SPDX-License-Identifier: GPL-3.0-only
//! Pure TOML-literal escaping leaf (SL-024, design §5). The single render-escape
//! seam: user free-text (`title`, an explicit `slug`) is emitted through these
//! fns instead of a raw `"{{v}}"` splice, so a `"`, `\`, newline, or `]` can
//! neither break the scaffolded `*.toml` document nor inject a key. Imports only
//! the `toml` crate — a leaf depended on by every command-tier renderer
//! (`adr`/`slice`/`spec`/`requirement`/`backlog`/`memory`), no cycle (ADR-001).
//!
//! The bodies are the byte-for-byte move of `memory.rs`'s original private
//! escaper (the SL-008 A-1 fix), so memory's output stays identical and the
//! other five renderers gain the same guarantee (design D1/D3).

/// Render `s` as a TOML basic-string literal — quoted and fully escaped by the
/// serializer (the read-path's own `toml` stack). The interpolated value lines
/// emit this in place of a raw `"{{v}}"` splice, so a `"`, newline, or `]` can
/// neither break the document nor inject a key.
pub(crate) fn toml_string(s: &str) -> String {
    toml::Value::String(s.to_owned()).to_string()
}

/// Render the *inner* of a TOML array literal — each element escaped through
/// `toml_string`, comma-joined (the template supplies the surrounding `[ ]`).
/// The single escaping seam for every scope array, so a hostile element cannot
/// break out of the array.
pub(crate) fn toml_array_inner(xs: &[String]) -> String {
    xs.iter()
        .map(|s| toml_string(s))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Shared adversarial fixtures for the per-renderer round-trip tests (SL-024):
/// the quoted-literal breakers a hostile `title`/`--slug` can carry. A basic
/// string only needs `"`, `\`, and control chars escaped — `]` is a breaker in
/// array context, not in a quoted literal, so it lives in the array test, not
/// here. One corpus, so the next breaker-class is added in one place.
#[cfg(test)]
pub(crate) const HOSTILE_TITLE: &str = "a\"b\\c\nd";
#[cfg(test)]
pub(crate) const HOSTILE_SLUG: &str = "p\"q";

#[cfg(test)]
mod tests {
    use super::*;

    /// Parse `v = <literal>` and return the round-tripped string value.
    fn round_trip_string(literal: &str) -> String {
        let doc = format!("v = {literal}");
        let parsed: toml::Value = toml::from_str(&doc).unwrap();
        parsed["v"].as_str().unwrap().to_owned()
    }

    /// Parse `v = [<inner>]` and return the round-tripped array of strings.
    fn round_trip_array(inner: &str) -> Vec<String> {
        let doc = format!("v = [{inner}]");
        let parsed: toml::Value = toml::from_str(&doc).unwrap();
        parsed["v"]
            .as_array()
            .unwrap()
            .iter()
            .map(|e| e.as_str().unwrap().to_owned())
            .collect()
    }

    #[test]
    fn toml_string_is_identity_on_safe_input() {
        // the safe-input guarantee: identical to the old raw `"{{title}}"` splice.
        assert_eq!(toml_string("Fast boot"), "\"Fast boot\"");
    }

    #[test]
    fn toml_string_escapes_quote_backslash_newline() {
        for hostile in ["a\"b", "a\\b", "a\nb", "a\"\\\nb", "ends-with-quote\""] {
            let literal = toml_string(hostile);
            assert_eq!(
                round_trip_string(&literal),
                hostile,
                "literal was {literal}"
            );
        }
    }

    #[test]
    fn toml_array_inner_escapes_string_and_array_breakers() {
        // `]` / `,` break the array case; `"` breaks the element. All must survive.
        let xs = vec![
            "plain".to_owned(),
            "has]bracket".to_owned(),
            "has,comma".to_owned(),
            "has\"quote".to_owned(),
            "has\nnewline".to_owned(),
        ];
        assert_eq!(round_trip_array(&toml_array_inner(&xs)), xs);
    }

    #[test]
    fn toml_array_inner_empty_is_empty() {
        assert_eq!(toml_array_inner(&[]), "");
        assert_eq!(
            round_trip_array(&toml_array_inner(&[])),
            Vec::<String>::new()
        );
    }
}
