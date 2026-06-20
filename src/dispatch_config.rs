// SPDX-License-Identifier: GPL-3.0-only
//! `dispatch_config` — the `[dispatch]` section of `doctrine.toml` (IMP-101,
//! SL-108 design D3, SL-117).
//!
//! Declares the project's preferred subprocess harness for dispatch workers
//! and whether to force subprocess dispatch even when native subagents are
//! available. Purely advisory — the dispatch orchestrator (LLM) reads this to
//! choose the spawn arm; the config is also available programmatically for
//! validation and display.

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

const DEFAULT_DELIVER_TO: &str = "refs/heads/main";
fn default_deliver_to() -> String {
    DEFAULT_DELIVER_TO.to_string()
}

/// The `[dispatch]` table from `doctrine.toml`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case", default)]
pub(crate) struct DispatchConfig {
    /// Preferred subprocess harness for dispatch workers. Defaults to `codex`
    /// unless explicitly set to `pi`.
    #[serde(default)]
    pub(crate) preferred_subprocess_harness: SubprocessHarness,
    /// Force Claude orchestrators to use the subprocess dispatch arm
    /// (codex/pi) even though the native `Agent` subagent tool is available.
    /// Defaults to `false` (use native subagents where available).
    /// Inert on non-Claude orchestrators.
    #[serde(default)]
    pub(crate) claude_force_subprocess_dispatch: bool,
    /// The trunk delivery ref dispatch advances to / the close-integration
    /// gate checks against (IMP-124). The same value becomes the PR *base*
    /// under a future delivery-mode key. NOT the fork-base resolver
    /// (ADR-006 D3 `DOCTRINE_TRUNK_REF` / ladder), which resolves a
    /// commit-ish to fork *from*.
    #[serde(default = "default_deliver_to")]
    pub(crate) deliver_to: String,
}

impl Default for DispatchConfig {
    fn default() -> Self {
        Self {
            preferred_subprocess_harness: SubprocessHarness::default(),
            claude_force_subprocess_dispatch: false,
            deliver_to: default_deliver_to(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_config_defaults_to_codex() {
        // Documented invariant: an absent or empty config yields codex for
        // backward compatibility. Both the Rust Default derive and the TOML
        // deserialize default must agree.
        assert_eq!(
            DispatchConfig::default().preferred_subprocess_harness,
            SubprocessHarness::Codex
        );
        let doc: DispatchConfig = toml::from_str("").unwrap();
        assert_eq!(doc.preferred_subprocess_harness, SubprocessHarness::Codex);
    }

    #[test]
    fn parse_prefers_pi() {
        let doc: DispatchConfig =
            toml::from_str("preferred-subprocess-harness = \"pi\"\n").unwrap();
        assert_eq!(doc.preferred_subprocess_harness, SubprocessHarness::Pi);
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

    // --- claude-force-subprocess-dispatch (SL-117) ---

    #[test]
    fn claude_force_defaults_false() {
        // Both the Rust Default and the serde absent-key path must yield false.
        assert!(!DispatchConfig::default().claude_force_subprocess_dispatch);
        let doc: DispatchConfig = toml::from_str("").unwrap();
        assert!(!doc.claude_force_subprocess_dispatch);
        // [dispatch] present but key absent → false
        let doc: DispatchConfig =
            toml::from_str("preferred-subprocess-harness = \"pi\"\n").unwrap();
        assert!(!doc.claude_force_subprocess_dispatch);
    }

    #[test]
    fn parse_claude_force_true() {
        let doc: DispatchConfig =
            toml::from_str("claude-force-subprocess-dispatch = true\n").unwrap();
        assert!(doc.claude_force_subprocess_dispatch);
    }

    #[test]
    fn parse_claude_force_false() {
        let doc: DispatchConfig =
            toml::from_str("claude-force-subprocess-dispatch = false\n").unwrap();
        assert!(!doc.claude_force_subprocess_dispatch);
    }

    #[test]
    fn parse_combined_keys() {
        let doc: DispatchConfig = toml::from_str(
            "preferred-subprocess-harness = \"pi\"\nclaude-force-subprocess-dispatch = true\n",
        )
        .unwrap();
        assert_eq!(doc.preferred_subprocess_harness, SubprocessHarness::Pi);
        assert!(doc.claude_force_subprocess_dispatch);
    }

    #[test]
    fn deliver_to_defaults_to_main() {
        assert_eq!(DispatchConfig::default().deliver_to, "refs/heads/main");
        let doc: DispatchConfig = toml::from_str("").unwrap();
        assert_eq!(doc.deliver_to, "refs/heads/main");
    }

    #[test]
    fn parse_deliver_to_override() {
        let doc: DispatchConfig = toml::from_str("deliver-to = \"refs/heads/release\"\n").unwrap();
        assert_eq!(doc.deliver_to, "refs/heads/release");
    }

    #[test]
    fn deliver_to_default_matches_serde_absent() {
        let absent: DispatchConfig = toml::from_str("").unwrap();
        assert_eq!(DispatchConfig::default().deliver_to, absent.deliver_to);
    }
}
