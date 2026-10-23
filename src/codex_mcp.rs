// SPDX-License-Identifier: GPL-3.0-only
//! The codex user MCP leg (SL-286) — what doctrine installs into a *user's*
//! `$CODEX_HOME/config.toml`, as opposed to the project-scoped entry `boot`
//! owns. Command tier: it may reach leaf and engine modules, never `boot` or
//! `install` (the dependency runs `boot` → here), and every fact about the
//! machine — where codex lives, what `$CODEX_HOME` is, whether a CLI is on
//! `PATH` — arrives as a supplied input rather than being read here.
//!
//! This phase carries the shared vocabulary: the codex config/table/server
//! constants, the wrapper-line normaliser, and [`is_wrapper_form`] — the single
//! recognition rule for "an entry whose command and args are a wrapper line
//! doctrine emitted", used by [`crate::boot`]'s project-entry classifier until
//! the user-entry planner takes it over.

/// The project-root codex config file — the codex MCP leg's write target
/// (SL-271). The project layer is loaded by codex for TRUSTED projects only, so
/// a written entry is never claimed active. Distinct from
/// `crate::boot::CODE_HOOKS_REL`: codex hooks and MCP servers live in different
/// files.
pub(crate) const CODEX_CONFIG_REL: &str = ".codex/config.toml";

/// The codex TOML table doctrine registers its MCP server under.
pub(crate) const CODEX_MCP_TABLE: &str = "mcp_servers";

/// The server key inside [`CODEX_MCP_TABLE`]. Pinned == `crate::boot::MCP_SERVER_KEY`
/// (one server, two harnesses) rather than shared by construction; the agreement
/// test holds them together (STD-001).
pub(crate) const CODEX_MCP_SERVER_KEY: &str = "doctrine";

/// The shell that reads `DOCTRINE_BIN` for the codex entry — codex execs
/// `command` literally (no `${VAR:-default}` interpolation), so the override
/// must run through a shell (DEC-323). That POSIX-shell dependency is declared
/// per POL-002 facet 3 (README).
pub(crate) const CODEX_MCP_SHELL: &str = "sh";
/// The shell flag carrying the command string.
pub(crate) const CODEX_MCP_SHELL_FLAG: &str = "-c";
/// The doctrine serve args shared by both harnesses' entries.
pub(crate) const CODEX_MCP_SERVE_ARGS: &str = "serve --mcp";
/// The wrapper line doctrine emits as the codex `args[1]` — the portable env
/// literal executed through [`CODEX_MCP_SHELL`]. `const` cannot compose, so this
/// holds its own literal; the agreement test pins it to `PORTABLE_EXEC`.
pub(crate) const CODEX_MCP_WRAPPER: &str = "exec \"${DOCTRINE_BIN:-doctrine}\" serve --mcp";
/// Every wrapper line doctrine has ever emitted, newest first (today: one). A
/// wording change APPENDS its predecessor, so a previously-emitted wrapper stays
/// ours and refreshes instead of becoming foreign and nagging forever.
pub(crate) const CODEX_MCP_WRAPPER_FORMS: &[&str] = &[CODEX_MCP_WRAPPER];
/// The codex env forwarding seam — codex gives an MCP child a FILTERED
/// environment, so `DOCTRINE_BIN` survives only if whitelisted.
pub(crate) const CODEX_MCP_ENV: &str = "DOCTRINE_BIN";

/// Trim a wrapper line and collapse internal whitespace runs to one space — the
/// canonicalising comparison behind [`CODEX_MCP_WRAPPER_FORMS`] membership, so a
/// re-spaced variant of an emitted wording still reads as ours.
pub(crate) fn normalise(line: &str) -> String {
    line.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Whether this entry's `command`/`args` pair is a wrapper line doctrine
/// emitted: `command` is [`CODEX_MCP_SHELL`] and `args` is exactly
/// `[CODEX_MCP_SHELL_FLAG, line]` with `line` in [`CODEX_MCP_WRAPPER_FORMS`]
/// once [`normalise`]d. The one recognition rule — the project-entry classifier
/// and the user-entry planner both ask this, so "ours" cannot mean two things.
/// Every other key on the table is ignored: ownership of the *shape* is what
/// this answers, not an opinion on extra keys.
pub(crate) fn is_wrapper_form(table: &dyn toml_edit::TableLike) -> bool {
    let Some(args) = table.get("args").and_then(|i| i.as_array()) else {
        return false;
    };
    table.get("command").and_then(|i| i.as_str()) == Some(CODEX_MCP_SHELL)
        && args.len() == 2
        && args.get(0).and_then(|v| v.as_str()) == Some(CODEX_MCP_SHELL_FLAG)
        && args
            .get(1)
            .and_then(|v| v.as_str())
            .is_some_and(|line| CODEX_MCP_WRAPPER_FORMS.contains(&normalise(line).as_str()))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Parse an entry body as the `[server]` table and ask whether it is a
    /// wrapper form. Single-quoted TOML literals keep the `$`/`"` of a wrapper
    /// line readable here.
    fn is_wrapper(body: &str) -> bool {
        let doc: toml_edit::DocumentMut = format!("[server]\n{body}").parse().unwrap();
        doc.get("server")
            .and_then(toml_edit::Item::as_table_like)
            .is_some_and(is_wrapper_form)
    }

    const EMITTED_ARGS: &str = r#"args = ["-c", 'exec "${DOCTRINE_BIN:-doctrine}" serve --mcp']"#;

    #[test]
    fn is_wrapper_form_accepts_the_emitted_wrapper() {
        assert!(is_wrapper(&format!("command = \"sh\"\n{EMITTED_ARGS}")));
    }

    #[test]
    fn is_wrapper_form_accepts_a_respaced_line() {
        // normalise collapses the runs, so a hand-mangled wrapper is still ours.
        assert!(is_wrapper(
            "command = \"sh\"\nargs = [\"-c\", 'exec    \"${DOCTRINE_BIN:-doctrine}\"   serve  --mcp']"
        ));
    }

    #[test]
    fn is_wrapper_form_ignores_extra_keys() {
        assert!(is_wrapper(&format!(
            "command = \"sh\"\n{EMITTED_ARGS}\nenv_vars = [\"DOCTRINE_BIN\"]\nenabled = true"
        )));
    }

    #[test]
    fn is_wrapper_form_rejects_a_wrong_command() {
        assert!(!is_wrapper(&format!("command = \"bash\"\n{EMITTED_ARGS}")));
        assert!(!is_wrapper(&format!("{EMITTED_ARGS}")), "command is absent");
        assert!(!is_wrapper(&format!("command = 1\n{EMITTED_ARGS}")));
    }

    #[test]
    fn is_wrapper_form_rejects_a_wrong_shell_flag() {
        assert!(!is_wrapper(
            "command = \"sh\"\nargs = [\"-x\", 'exec \"${DOCTRINE_BIN:-doctrine}\" serve --mcp']"
        ));
    }

    #[test]
    fn is_wrapper_form_requires_exactly_two_args() {
        assert!(!is_wrapper("command = \"sh\"\nargs = [\"-c\"]"));
        assert!(!is_wrapper(
            "command = \"sh\"\nargs = [\"-c\", 'exec \"${DOCTRINE_BIN:-doctrine}\" serve --mcp', \"extra\"]"
        ));
    }

    #[test]
    fn is_wrapper_form_rejects_an_unknown_line() {
        // Right shell and flag, wording doctrine never emitted.
        assert!(!is_wrapper(
            "command = \"sh\"\nargs = [\"-c\", \"exec doctrine serve --mcp\"]"
        ));
    }

    #[test]
    fn is_wrapper_form_rejects_non_string_args() {
        assert!(!is_wrapper("command = \"sh\"\nargs = [1, 2]"));
        assert!(!is_wrapper("command = \"sh\"\nargs = \"-c\""));
    }
}
