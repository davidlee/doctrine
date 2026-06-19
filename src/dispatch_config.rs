// SPDX-License-Identifier: GPL-3.0-only
//! `dispatch_config` — the `[dispatch]` section of `doctrine.toml` (IMP-101,
//! SL-108 design D3).
//!
//! Declares the project's preferred subprocess harness for dispatch workers.
//! Purely advisory — the dispatch orchestrator (LLM) reads this to choose the
//! spawn arm; the config is also available programmatically for validation and
//! display.

use serde::Deserialize;

/// The subprocess harness for dispatch workers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum SubprocessHarness {
    /// `codex exec` spawn arm (the default for backward compatibility).
    #[default]
    Codex,
    /// pi RPC mode spawn arm (SL-108).
    Pi,
}

/// The `[dispatch]` table from `doctrine.toml`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case", default)]
pub(crate) struct DispatchConfig {
    /// Preferred subprocess harness for dispatch workers. Defaults to `codex`
    /// unless explicitly set to `pi`.
    #[serde(default)]
    pub(crate) preferred_subprocess_harness: SubprocessHarness,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_codex() {
        let cfg = DispatchConfig::default();
        assert_eq!(cfg.preferred_subprocess_harness, SubprocessHarness::Codex);
    }

    #[test]
    fn parse_prefers_pi() {
        let doc: DispatchConfig =
            toml::from_str("preferred-subprocess-harness = \"pi\"\n").unwrap();
        assert_eq!(doc.preferred_subprocess_harness, SubprocessHarness::Pi);
    }

    #[test]
    fn absent_key_defaults_to_codex() {
        let doc: DispatchConfig = toml::from_str("").unwrap();
        assert_eq!(doc.preferred_subprocess_harness, SubprocessHarness::Codex);
    }

    #[test]
    fn unknown_harness_is_error() {
        let err = toml::from_str::<DispatchConfig>("preferred-subprocess-harness = \"cursor\"\n")
            .unwrap_err();
        assert!(
            err.to_string().contains("preferred-subprocess-harness"),
            "expected error to mention the key: {err}"
        );
    }
}
