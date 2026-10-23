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

use std::path::{Path, PathBuf};

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

/// The user-config file name inside a `$CODEX_HOME` directory — the same file
/// [`CODEX_CONFIG_REL`] reaches through `$HOME`, so one name, two anchors.
pub(crate) const CODEX_CONFIG_FILE: &str = "config.toml";

/// Why the leg cannot propose a write (design sec-leg § Plan). Each variant is a
/// distinct user-facing remedy, so the planner never has to re-derive the cause
/// from a path that happens to be unusable.
///
/// `expect` rather than the `cfg_attr(not(test), …)` form: the variants arrive
/// with the planners that construct them (PHASE-04), so several are unbuilt in
/// *every* build until then. The expectation is a tripwire — it must go once the
/// last variant has a constructor.
#[expect(
    dead_code,
    reason = "variants arrive with their constructing planner in SL-286 PHASE-04"
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Degraded {
    /// `$CODEX_HOME` is set but empty, relative, or names nothing on disk.
    BadCodexHome,
    /// `$CODEX_HOME` is unset and `$HOME` is too — no anchor for the user file.
    NoHome,
    /// The destination exists but could not be read.
    UserUnreadable,
    /// The destination is not parseable TOML.
    UserUnparseable,
    /// The destination parses but the entry's shape is not interpretable.
    UserMalformed,
    /// The running binary is not the `doctrine` on `PATH`, so a baked path would
    /// not be the binary the user actually runs.
    ExecNotOnPath,
    /// The running binary's own path could not be resolved at all.
    ExecUnresolved,
    /// A user write is pending and no `codex` CLI is on `PATH` to honour it.
    NoCodexCli,
}

/// Env facts the shell gathers (never read here): the leg is pure, so every
/// environment question arrives as a supplied value.
#[cfg_attr(not(test), expect(dead_code, reason = "wired in SL-286 PHASE-04"))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CodexEnv {
    /// `$CODEX_HOME` as set — `None` means unset (empty is a distinct, bad value).
    pub(crate) codex_home: Option<std::ffi::OsString>,
    /// The shell's probe of that path: codex only honours an existing directory.
    pub(crate) codex_home_exists: bool,
    /// `$HOME` — the fallback anchor for the user config file.
    pub(crate) home: Option<PathBuf>,
    /// Whether a `codex` CLI is on `PATH` (the shell's `fsutil::find_on_path`).
    pub(crate) codex_on_path: bool,
}

/// Bake facts the shell gathers, already canonicalised where the field says so:
/// deciding whether the running binary *is* the `PATH` entry is a comparison of
/// canonical paths, and that resolution is IO the pure layer must not do.
#[cfg_attr(not(test), expect(dead_code, reason = "wired in SL-286 PHASE-04"))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BakeFacts {
    /// Canonical path of the running binary; `None` = unresolved.
    pub(crate) exec: Option<PathBuf>,
    /// The first `doctrine` on `PATH`, as found — the path to bake.
    pub(crate) path_doctrine: Option<PathBuf>,
    /// That entry's canonical form — the comparison key against `exec`.
    pub(crate) path_doctrine_canonical: Option<PathBuf>,
}

/// Where the user config file lives: `$CODEX_HOME/config.toml` when codex's own
/// variable names an existing absolute directory, else `$HOME/.codex/config.toml`.
/// A `$CODEX_HOME` that is empty, relative, or absent on disk is refused rather
/// than silently falling back to `$HOME` — codex would not read the file we wrote
/// there, and a *wrong* write is worse than none.
#[cfg_attr(not(test), expect(dead_code, reason = "wired in SL-286 PHASE-04"))]
pub(crate) fn destination(env: &CodexEnv) -> Result<PathBuf, Degraded> {
    match &env.codex_home {
        Some(dir) => {
            let dir = Path::new(dir);
            // An empty value is not absolute either — both spell a bad CODEX_HOME.
            if !dir.is_absolute() || !env.codex_home_exists {
                return Err(Degraded::BadCodexHome);
            }
            Ok(dir.join(CODEX_CONFIG_FILE))
        }
        None => env
            .home
            .as_ref()
            .map(|home| home.join(CODEX_CONFIG_REL))
            .ok_or(Degraded::NoHome),
    }
}

/// The `doctrine` path to bake into the user entry: the `PATH` entry's own path
/// when the running binary resolves to the same file, so a self-update through
/// that entry keeps working (`Ok(path_doctrine)`, never the canonical target).
/// A different file means the running binary is not what `doctrine` on `PATH`
/// runs — a dev build, typically — and baking either path would be a lie.
#[cfg_attr(not(test), expect(dead_code, reason = "wired in SL-286 PHASE-04"))]
pub(crate) fn bake(facts: &BakeFacts) -> Result<PathBuf, Degraded> {
    let Some(exec) = &facts.exec else {
        return Err(Degraded::ExecUnresolved);
    };
    match (&facts.path_doctrine, &facts.path_doctrine_canonical) {
        (Some(path_doctrine), Some(canonical)) if canonical == exec => Ok(path_doctrine.clone()),
        _ => Err(Degraded::ExecNotOnPath),
    }
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

    // --- destination (design sec-leg § Plan) ---

    fn codex_env(
        codex_home: Option<&str>,
        codex_home_exists: bool,
        home: Option<&str>,
    ) -> CodexEnv {
        CodexEnv {
            codex_home: codex_home.map(std::ffi::OsString::from),
            codex_home_exists,
            home: home.map(PathBuf::from),
            codex_on_path: true,
        }
    }

    #[test]
    fn destination_uses_home_dot_codex_when_codex_home_is_unset() {
        let env = codex_env(None, false, Some("/home/u"));
        assert_eq!(
            destination(&env),
            Ok(PathBuf::from("/home/u/.codex/config.toml"))
        );
    }

    #[test]
    fn destination_no_home_when_neither_codex_home_nor_home_is_set() {
        assert_eq!(
            destination(&codex_env(None, false, None)),
            Err(Degraded::NoHome)
        );
    }

    #[test]
    fn destination_bad_codex_home_when_empty() {
        assert_eq!(
            destination(&codex_env(Some(""), true, Some("/home/u"))),
            Err(Degraded::BadCodexHome),
            "an empty value names no directory"
        );
    }

    #[test]
    fn destination_bad_codex_home_when_relative() {
        assert_eq!(
            destination(&codex_env(Some(".codex"), true, Some("/home/u"))),
            Err(Degraded::BadCodexHome),
            "a relative value is CWD-dependent, so it is not a resolved path"
        );
    }

    #[test]
    fn destination_bad_codex_home_when_missing() {
        assert_eq!(
            destination(&codex_env(Some("/home/u/.codex"), false, Some("/home/u"))),
            Err(Degraded::BadCodexHome),
            "the shell's probe says nothing is there"
        );
    }

    #[test]
    fn destination_uses_an_absolute_existing_codex_home() {
        let env = codex_env(Some("/opt/codex"), true, Some("/home/u"));
        assert_eq!(
            destination(&env),
            Ok(PathBuf::from("/opt/codex/config.toml")),
            "CODEX_HOME wins over HOME"
        );
    }

    // --- bake (design sec-leg § Plan) ---

    fn bake_facts(
        exec: Option<&str>,
        path_doctrine: Option<&str>,
        path_doctrine_canonical: Option<&str>,
    ) -> BakeFacts {
        BakeFacts {
            exec: exec.map(PathBuf::from),
            path_doctrine: path_doctrine.map(PathBuf::from),
            path_doctrine_canonical: path_doctrine_canonical.map(PathBuf::from),
        }
    }

    #[test]
    fn bake_returns_the_path_entry_not_its_canonical_form() {
        // A nix/home-manager install: the running binary is the store copy, the
        // PATH entry is the profile symlink that survives a rebuild.
        const CANON: &str = "/nix/store/x/bin/doctrine";
        const ON_PATH: &str = "/home/u/.nix-profile/bin/doctrine";

        let facts = bake_facts(Some(CANON), Some(ON_PATH), Some(CANON));
        assert_eq!(
            bake(&facts),
            Ok(PathBuf::from(ON_PATH)),
            "the PATH entry's own path — the canonical store path would pin a rebuild"
        );
    }

    #[test]
    fn bake_exec_not_on_path_for_a_dev_build() {
        let facts = bake_facts(
            Some("/home/u/src/doctrine/target/debug/doctrine"),
            Some("/home/u/.nix-profile/bin/doctrine"),
            Some("/nix/store/x/bin/doctrine"),
        );
        assert_eq!(bake(&facts), Err(Degraded::ExecNotOnPath));
    }

    #[test]
    fn bake_exec_not_on_path_when_doctrine_is_absent_from_path() {
        let facts = bake_facts(Some("/bin/doctrine"), None, None);
        assert_eq!(bake(&facts), Err(Degraded::ExecNotOnPath));
        // a canonical fact with no entry it came from cannot bake either.
        let orphan = bake_facts(Some("/bin/doctrine"), None, Some("/bin/doctrine"));
        assert_eq!(bake(&orphan), Err(Degraded::ExecNotOnPath));
    }

    #[test]
    fn bake_exec_unresolved_when_the_running_binary_is_unknown() {
        let facts = bake_facts(None, Some("/bin/doctrine"), Some("/bin/doctrine"));
        assert_eq!(bake(&facts), Err(Degraded::ExecUnresolved));
    }
}
