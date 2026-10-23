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
/// `expect` is the `cfg_attr(not(test), …)` form because every variant now has a
/// constructor in the test build (`NoCodexCli` is built by [`plan`]), so only the
/// non-test build — where nothing calls the planner yet — can report it unused.
/// It lapses when PHASE-04 wires the planner into install.
#[cfg_attr(not(test), expect(dead_code, reason = "wired in SL-286 PHASE-04"))]
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
    /// The shell's probe of that path: doctrine only honours an entry that names
    /// an existing directory. Codex's own handling of a missing or relative
    /// `$CODEX_HOME` is unprobed (design RV-427 `F-9`), so doctrine refuses to
    /// guess rather than falling back to `$HOME` and writing where codex will not
    /// read.
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

/// A config file as the shell read it. Three states, not `Option<&str>`: "absent"
/// (create the file) and "unreadable" (refuse, and say why) are different
/// answers, and collapsing them would let a permission error look like a fresh
/// install — the defect ISS-495 records on the Claude read.
#[cfg_attr(not(test), expect(dead_code, reason = "wired in SL-286 PHASE-04"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FileRead<'a> {
    Absent,
    Unreadable,
    Text(&'a str),
}

/// The user's `[mcp_servers.doctrine]` entry against the entry doctrine would
/// write. `Replace` carries what the planner cannot derive once the table is
/// gone: the old invocation, as the report shows it, and every key being dropped
/// — the consent text has to name what the user loses.
#[cfg_attr(not(test), expect(dead_code, reason = "wired in SL-286 PHASE-04"))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum UserEntry {
    /// No entry to overwrite.
    Add,
    /// Already exactly the canonical entry — nothing to write.
    Current,
    Replace {
        /// The existing entry's own invocation: command then args, single-spaced.
        old: String,
        /// Every key the write removes, in file order.
        dropped: Vec<String>,
    },
}

/// Classify the user's `[mcp_servers.doctrine]` entry: absent → [`UserEntry::Add`],
/// already ours — command and serve args as baked, `enabled` not `false` →
/// [`UserEntry::Current`] (nothing is written, so nothing is dropped), a stdio
/// entry doctrine can interpret → [`UserEntry::Replace`]. `bake` is the path the
/// entry *should* carry ([`bake`]), so an entry pointing anywhere else is ours to
/// replace. Everything the classifier cannot interpret — a url transport, a
/// non-string `command`, `args` that are not an array of strings — is
/// `UserMalformed`: guessing would mean overwriting a server doctrine does not
/// understand. Every other key survives a `Current` and is named in `dropped`
/// when replacing.
#[cfg_attr(not(test), expect(dead_code, reason = "wired in SL-286 PHASE-04"))]
pub(crate) fn classify_user(read: FileRead<'_>, bake: &Path) -> Result<UserEntry, Degraded> {
    let text = match read {
        FileRead::Absent => return Ok(UserEntry::Add),
        FileRead::Unreadable => return Err(Degraded::UserUnreadable),
        FileRead::Text(text) => text,
    };
    let doc: toml_edit::DocumentMut = match text.parse() {
        Ok(doc) => doc,
        Err(_) => return Err(Degraded::UserUnparseable),
    };
    let table = match doctrine_entry(&doc) {
        EntryLookup::NoEntry => return Ok(UserEntry::Add),
        EntryLookup::NotATable => return Err(Degraded::UserMalformed),
        EntryLookup::Entry(table) => table,
    };
    let Some(command) = table.get("command").and_then(|i| i.as_str()) else {
        return Err(Degraded::UserMalformed);
    };
    // A url transport is a server doctrine cannot own — never silently replaced.
    if table.get("url").is_some() {
        return Err(Degraded::UserMalformed);
    }
    let Some(args) = string_args(table) else {
        return Err(Degraded::UserMalformed);
    };
    // `enabled` is the one key that can veto: codex honours a disabled server,
    // so a matching entry that is switched off is not the entry we want.
    let enabled_off = table.get("enabled").and_then(toml_edit::Item::as_bool) == Some(false);
    if !enabled_off && Path::new(command) == bake && args_are_serve(&args) {
        return Ok(UserEntry::Current);
    }
    let dropped: Vec<String> = table
        .iter()
        .filter(|(key, _)| !matches!(*key, "command" | "args"))
        .flat_map(|(key, item)| render_dropped(key, item))
        .collect();
    Ok(UserEntry::Replace {
        old: invocation(command, &args),
        dropped,
    })
}

/// The `[mcp_servers.doctrine]` entry inside a parsed config file — the three
/// questions both classifiers ask before they differ. `NoEntry` is "there is
/// nothing of ours here" (`Add` for the user layer, `Absent` for the project
/// one); `NotATable` is "something of ours is there but unreadable as a server"
/// (`Malformed` in both). Interpreting the table stays with each caller: the two
/// layers disagree about *what* an entry means, not about where it lives.
#[cfg_attr(not(test), expect(dead_code, reason = "wired in SL-286 PHASE-04"))]
enum EntryLookup<'d> {
    NoEntry,
    NotATable,
    Entry(&'d dyn toml_edit::TableLike),
}

/// Locate `[mcp_servers.doctrine]` in a parsed config file. Parsing itself stays
/// with the caller, because a parse failure means different things per layer
/// ([`Degraded::UserUnparseable`] vs [`ProjectEntry::Unreadable`]).
#[cfg_attr(not(test), expect(dead_code, reason = "wired in SL-286 PHASE-04"))]
fn doctrine_entry(doc: &toml_edit::DocumentMut) -> EntryLookup<'_> {
    let Some(servers) = doc.get(CODEX_MCP_TABLE) else {
        return EntryLookup::NoEntry;
    };
    let Some(servers) = servers.as_table_like() else {
        return EntryLookup::NotATable;
    };
    let Some(entry) = servers.get(CODEX_MCP_SERVER_KEY) else {
        return EntryLookup::NoEntry;
    };
    match entry.as_table_like() {
        Some(table) => EntryLookup::Entry(table),
        None => EntryLookup::NotATable,
    }
}

/// The project `.codex/config.toml` `[mcp_servers.doctrine]` entry, classified
/// by form rather than by name: the project layer is where a user, a previous
/// doctrine, and the codex CLI itself may all have written, so the planner has to
/// say which of them it is looking at before advising anything (design sec-leg
/// § Plan step 4). Only an owned wrapper is doctrine's to remove.
#[cfg_attr(not(test), expect(dead_code, reason = "wired in SL-286 PHASE-04"))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ProjectEntry {
    /// No entry, or no project file.
    Absent,
    /// A wrapper line doctrine emitted ([`is_wrapper_form`]) — removed with the
    /// user write. `extra_keys` is everything else it carries, as the report
    /// names it, so the consent text can state what the user loses.
    OwnedWrapper { extra_keys: Vec<String> },
    /// An explicit choice doctrine will not touch: a path command (absolute or
    /// relative), or a url transport — `target` is the command or the url.
    Override { target: String },
    /// A bare-name command, resolved through `PATH` at codex launch — a different
    /// `doctrine` may run. Untouched, but the install is incomplete.
    PathShadow { command: String },
    /// Not interpretable: empty command, wrong types, not a table.
    Malformed,
    /// The project file exists but could not be read or parsed.
    Unreadable,
}

/// Classify the project's `[mcp_servers.doctrine]` entry. The wrapper check
/// comes *before* the command checks because a wrapper's `command` is the bare
/// name `sh`: asking [`is_wrapper_form`] first is what keeps doctrine's own old
/// entry from being read as a `PATH` shadow of something else.
#[cfg_attr(not(test), expect(dead_code, reason = "wired in SL-286 PHASE-04"))]
pub(crate) fn classify_project(read: FileRead<'_>) -> ProjectEntry {
    let text = match read {
        FileRead::Absent => return ProjectEntry::Absent,
        FileRead::Unreadable => return ProjectEntry::Unreadable,
        FileRead::Text(text) => text,
    };
    let doc: toml_edit::DocumentMut = match text.parse() {
        Ok(doc) => doc,
        Err(_) => return ProjectEntry::Unreadable,
    };
    let table = match doctrine_entry(&doc) {
        EntryLookup::NoEntry => return ProjectEntry::Absent,
        EntryLookup::NotATable => return ProjectEntry::Malformed,
        EntryLookup::Entry(table) => table,
    };
    if is_wrapper_form(table) {
        let extra_keys = table
            .iter()
            .filter(|(key, _)| !matches!(*key, "command" | "args" | "env_vars"))
            .flat_map(|(key, item)| render_dropped(key, item))
            .collect();
        return ProjectEntry::OwnedWrapper { extra_keys };
    }
    // A url transport is a server doctrine cannot replace with a stdio entry —
    // an explicit choice, like a path command.
    if let Some(url) = table.get("url") {
        return match url.as_str() {
            Some(url) => ProjectEntry::Override {
                target: url.to_string(),
            },
            None => ProjectEntry::Malformed,
        };
    }
    let Some(command) = table.get("command").and_then(|i| i.as_str()) else {
        return ProjectEntry::Malformed;
    };
    if command.is_empty() {
        return ProjectEntry::Malformed;
    }
    // Absent `args` is fine here (a bare command is a complete entry); present
    // but unreadable is not.
    if table.get("args").is_some() && string_args(table).is_none() {
        return ProjectEntry::Malformed;
    }
    if command.contains('/') {
        return ProjectEntry::Override {
            target: command.to_string(),
        };
    }
    ProjectEntry::PathShadow {
        command: command.to_string(),
    }
}

/// The entry's `args` as strings, or `None` when `args` is absent or holds a
/// non-string.
fn string_args(table: &dyn toml_edit::TableLike) -> Option<Vec<String>> {
    table
        .get("args")
        .and_then(|i| i.as_array())
        .and_then(|a| a.iter().map(|v| v.as_str().map(String::from)).collect())
}

/// Whether `args` is exactly the serve invocation — derived from
/// [`CODEX_MCP_SERVE_ARGS`], never a second literal.
fn args_are_serve(args: &[String]) -> bool {
    args.iter()
        .map(String::as_str)
        .eq(CODEX_MCP_SERVE_ARGS.split_whitespace())
}

/// The entry's own invocation, as the report shows it: command then args,
/// single-spaced.
fn invocation(command: &str, args: &[String]) -> String {
    std::iter::once(command)
        .chain(args.iter().map(String::as_str))
        .collect::<Vec<_>>()
        .join(" ")
}

/// How one key about to be dropped is named in the report. A value keeps its
/// TOML spelling (`enabled = false`); a sub-table lists its keys (`env.FOO`),
/// because a nested value's spelling is not what the user is being asked about.
fn render_dropped(key: &str, item: &toml_edit::Item) -> Vec<String> {
    if let Some(table) = item.as_table_like() {
        return table
            .iter()
            .map(|(sub, _)| format!("{key}.{sub}"))
            .collect();
    }
    match item.as_value() {
        Some(value) => vec![format!("{key} = {}", value.to_string().trim())],
        None => vec![key.to_string()],
    }
}

/// The codex user MCP leg's whole decision (design sec-leg § Plan): where the
/// user file is, what to bake there, what each layer currently holds, and — if
/// anything — the one cause that stops the leg. Partial facts are kept: a
/// degradation at a later step still leaves the report able to name the file and
/// the command the earlier steps resolved.
#[cfg_attr(not(test), expect(dead_code, reason = "wired in SL-286 PHASE-04"))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CodexUserPlan {
    /// The user config file, when [`destination`] resolved it.
    pub(crate) user_config: Option<PathBuf>,
    /// The `doctrine` command to write, when [`bake`] resolved it.
    pub(crate) bake: Option<PathBuf>,
    /// The user entry, `None` when an earlier degradation stopped classification.
    pub(crate) user: Option<UserEntry>,
    /// The project entry — always classified, whatever the user leg did.
    pub(crate) project: ProjectEntry,
    /// The first cause, in step order; `None` means the leg can proceed.
    pub(crate) degraded: Option<Degraded>,
}

impl CodexUserPlan {
    /// Whether the leg has anything to do: a user write pending (`Add`/
    /// `Replace`) or doctrine's own wrapper sitting in the project file — and
    /// nothing degraded. A `PathShadow`, an `Override` or a `Current` alone is
    /// worth a report, not a write, so the consent prompt is not asked.
    #[cfg_attr(not(test), expect(dead_code, reason = "wired in SL-286 PHASE-04"))]
    pub(crate) fn proposes_change(&self) -> bool {
        self.degraded.is_none()
            && (matches!(self.user, Some(UserEntry::Add | UserEntry::Replace { .. }))
                || matches!(self.project, ProjectEntry::OwnedWrapper { .. }))
    }
}

/// Decide the whole leg from supplied facts. The user leg is a chain that stops
/// at its first failure while keeping what it already resolved; the project file
/// is read regardless, since its class is what the report discloses. The codex
/// CLI is required only for a pending user write — reconciling an owned wrapper
/// in doctrine's own project file needs no CLI.
#[cfg_attr(not(test), expect(dead_code, reason = "wired in SL-286 PHASE-04"))]
pub(crate) fn plan(
    user: FileRead<'_>,
    project: FileRead<'_>,
    facts: &BakeFacts,
    env: &CodexEnv,
) -> CodexUserPlan {
    let mut plan = CodexUserPlan {
        user_config: None,
        bake: None,
        user: None,
        project: ProjectEntry::Absent,
        degraded: None,
    };
    let resolved = destination(env)
        .and_then(|config| {
            plan.user_config = Some(config);
            bake(facts)
        })
        .and_then(|command| {
            plan.bake = Some(command.clone());
            classify_user(user, &command)
        });
    match resolved {
        Ok(entry) => plan.user = Some(entry),
        Err(degraded) => plan.degraded = Some(degraded),
    }
    plan.project = classify_project(project);
    if plan.degraded.is_none()
        && matches!(plan.user, Some(UserEntry::Add | UserEntry::Replace { .. }))
        && !env.codex_on_path
    {
        plan.degraded = Some(Degraded::NoCodexCli);
    }
    plan
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

    // --- classify_user (the user's [mcp_servers.doctrine] entry) ---

    const BAKE: &str = "/usr/local/bin/doctrine";

    fn classify(text: &str) -> Result<UserEntry, Degraded> {
        classify_user(FileRead::Text(text), Path::new(BAKE))
    }

    #[test]
    fn classify_user_add_when_the_file_is_absent() {
        assert_eq!(
            classify_user(FileRead::Absent, Path::new(BAKE)),
            Ok(UserEntry::Add)
        );
    }

    #[test]
    fn classify_user_add_when_mcp_servers_is_missing() {
        assert_eq!(classify("model = \"gpt-5\"\n"), Ok(UserEntry::Add));
    }

    #[test]
    fn classify_user_add_when_the_doctrine_entry_is_missing() {
        assert_eq!(
            classify("[mcp_servers.other]\ncommand = \"x\"\n"),
            Ok(UserEntry::Add)
        );
    }

    #[test]
    fn classify_user_current_when_command_and_args_match() {
        let text = format!(
            "[mcp_servers.doctrine]\ncommand = \"{BAKE}\"\nargs = [\"serve\", \"--mcp\"]\n"
        );
        assert_eq!(classify(&text), Ok(UserEntry::Current));
    }

    #[test]
    fn classify_user_current_with_enabled_true() {
        let text = format!(
            "[mcp_servers.doctrine]\ncommand = \"{BAKE}\"\nargs = [\"serve\", \"--mcp\"]\nenabled = true\n"
        );
        assert_eq!(
            classify(&text),
            Ok(UserEntry::Current),
            "a `true` flag changes nothing codex runs, so there is nothing to write"
        );
    }

    #[test]
    fn classify_user_current_with_a_nested_env_table() {
        let text = format!(
            "[mcp_servers.doctrine]\ncommand = \"{BAKE}\"\nargs = [\"serve\", \"--mcp\"]\n\n[mcp_servers.doctrine.env]\nFOO = \"x\"\n"
        );
        assert_eq!(
            classify(&text),
            Ok(UserEntry::Current),
            "extra keys do not stop a Current — nothing is written, so nothing is lost"
        );
    }

    #[test]
    fn classify_user_replace_when_disabled() {
        let text = format!(
            "[mcp_servers.doctrine]\ncommand = \"{BAKE}\"\nargs = [\"serve\", \"--mcp\"]\nenabled = false\n"
        );
        assert_eq!(
            classify(&text),
            Ok(UserEntry::Replace {
                old: format!("{BAKE} serve --mcp"),
                dropped: vec!["enabled = false".to_string()],
            }),
            "a disabled matching entry is still replaced — the write is what enables it"
        );
    }

    #[test]
    fn classify_user_replace_when_the_command_differs() {
        let text = "[mcp_servers.doctrine]\ncommand = \"/old/bin/doctrine\"\nargs = [\"serve\", \"--mcp\"]\n";
        assert_eq!(
            classify(text),
            Ok(UserEntry::Replace {
                old: "/old/bin/doctrine serve --mcp".to_string(),
                dropped: vec![],
            })
        );
    }

    #[test]
    fn classify_user_replace_drops_enabled_false_and_nested_env() {
        let text = format!(
            "[mcp_servers.doctrine]\ncommand = \"{BAKE}\"\nargs = [\"serve\", \"--mcp\"]\nenabled = false\n\n[mcp_servers.doctrine.env]\nFOO = \"x\"\n"
        );
        assert_eq!(
            classify(&text),
            Ok(UserEntry::Replace {
                old: format!("{BAKE} serve --mcp"),
                dropped: vec!["enabled = false".to_string(), "env.FOO".to_string()],
            }),
            "the report names exactly what the user loses"
        );
    }

    #[test]
    fn classify_user_malformed_for_a_url_transport() {
        let with_command = format!(
            "[mcp_servers.doctrine]\ncommand = \"{BAKE}\"\nargs = [\"serve\", \"--mcp\"]\nurl = \"https://example.test/mcp\"\n"
        );
        assert_eq!(classify(&with_command), Err(Degraded::UserMalformed));
        assert_eq!(
            classify("[mcp_servers.doctrine]\nurl = \"https://example.test/mcp\"\n"),
            Err(Degraded::UserMalformed),
            "a url entry is not a stdio server doctrine can own"
        );
    }

    #[test]
    fn classify_user_malformed_when_command_is_not_a_string() {
        assert_eq!(
            classify("[mcp_servers.doctrine]\ncommand = 3\nargs = [\"serve\"]\n"),
            Err(Degraded::UserMalformed)
        );
    }

    #[test]
    fn classify_user_malformed_when_args_are_not_strings() {
        let base = format!("[mcp_servers.doctrine]\ncommand = \"{BAKE}\"\n");
        assert_eq!(
            classify(&format!("{base}args = [\"serve\", 3]\n")),
            Err(Degraded::UserMalformed)
        );
        assert_eq!(
            classify(&format!("{base}args = \"serve --mcp\"\n")),
            Err(Degraded::UserMalformed),
            "a bare string is not the array codex expects"
        );
        assert_eq!(classify(&base), Err(Degraded::UserMalformed), "args absent");
    }

    #[test]
    fn classify_user_malformed_when_mcp_servers_is_not_a_table() {
        assert_eq!(classify("mcp_servers = 3\n"), Err(Degraded::UserMalformed));
        assert_eq!(
            classify("[mcp_servers]\ndoctrine = 3\n"),
            Err(Degraded::UserMalformed),
            "the entry itself is not table-like"
        );
    }

    #[test]
    fn classify_user_unparseable_when_the_text_is_not_toml() {
        assert_eq!(classify("not = [toml\n"), Err(Degraded::UserUnparseable));
    }

    #[test]
    fn classify_user_unreadable_when_the_read_failed() {
        assert_eq!(
            classify_user(FileRead::Unreadable, Path::new(BAKE)),
            Err(Degraded::UserUnreadable)
        );
    }

    // --- classify_project (the project's [mcp_servers.doctrine] entry) ---

    /// The project entry as doctrine's own install emitted it, built from the
    /// constants so a wording change cannot leave a stale fixture behind.
    fn wrapper_entry(extra: &str) -> String {
        format!(
            "[mcp_servers.doctrine]\ncommand = \"{CODEX_MCP_SHELL}\"\n\
             args = [\"{CODEX_MCP_SHELL_FLAG}\", '{CODEX_MCP_WRAPPER}']\n\
             env_vars = [\"{CODEX_MCP_ENV}\"]\n{extra}"
        )
    }

    fn project(text: &str) -> ProjectEntry {
        classify_project(FileRead::Text(text))
    }

    #[test]
    fn classify_project_absent_when_the_file_is_absent() {
        assert_eq!(classify_project(FileRead::Absent), ProjectEntry::Absent);
    }

    #[test]
    fn classify_project_absent_when_mcp_servers_is_missing() {
        assert_eq!(project("model = \"gpt-5\"\n"), ProjectEntry::Absent);
        assert_eq!(
            project("[mcp_servers.other]\ncommand = \"x\"\n"),
            ProjectEntry::Absent,
            "another server is not ours"
        );
    }

    #[test]
    fn classify_project_owned_wrapper_with_no_extra_keys() {
        assert_eq!(
            project(&wrapper_entry("")),
            ProjectEntry::OwnedWrapper { extra_keys: vec![] },
            "env_vars is part of what doctrine emitted, not a user extra"
        );
    }

    #[test]
    fn classify_project_owned_wrapper_lists_extra_keys() {
        assert_eq!(
            project(&wrapper_entry("enabled = false\nstartup_timeout_sec = 5\n")),
            ProjectEntry::OwnedWrapper {
                extra_keys: vec![
                    "enabled = false".to_string(),
                    "startup_timeout_sec = 5".to_string(),
                ],
            },
            "the report names exactly what removing the wrapper loses"
        );
    }

    #[test]
    fn classify_project_override_for_an_absolute_command() {
        assert_eq!(
            project("[mcp_servers.doctrine]\ncommand = \"/opt/doctrine\"\n"),
            ProjectEntry::Override {
                target: "/opt/doctrine".to_string(),
            }
        );
    }

    #[test]
    fn classify_project_override_for_a_relative_command() {
        assert_eq!(
            project("[mcp_servers.doctrine]\ncommand = \"./bin/doctrine\"\n"),
            ProjectEntry::Override {
                target: "./bin/doctrine".to_string(),
            },
            "a relative path is still an explicit choice, not a PATH shadow"
        );
    }

    #[test]
    fn classify_project_override_for_a_url_transport() {
        assert_eq!(
            project("[mcp_servers.doctrine]\nurl = \"https://example.test/mcp\"\n"),
            ProjectEntry::Override {
                target: "https://example.test/mcp".to_string(),
            }
        );
    }

    #[test]
    fn classify_project_malformed_when_url_is_not_a_string() {
        assert_eq!(
            project("[mcp_servers.doctrine]\nurl = 3\n"),
            ProjectEntry::Malformed
        );
    }

    #[test]
    fn classify_project_path_shadow_for_a_bare_name() {
        assert_eq!(
            project("[mcp_servers.doctrine]\ncommand = \"doctrine\"\n"),
            ProjectEntry::PathShadow {
                command: "doctrine".to_string(),
            },
            "a bare name resolves through PATH, so it may run something else"
        );
    }

    #[test]
    fn classify_project_malformed_when_command_is_empty() {
        assert_eq!(
            project("[mcp_servers.doctrine]\ncommand = \"\"\n"),
            ProjectEntry::Malformed
        );
    }

    #[test]
    fn classify_project_malformed_when_command_is_not_a_string() {
        assert_eq!(
            project("[mcp_servers.doctrine]\ncommand = 1\n"),
            ProjectEntry::Malformed
        );
        assert_eq!(
            project("[mcp_servers.doctrine]\nargs = [\"serve\"]\n"),
            ProjectEntry::Malformed,
            "command is absent"
        );
    }

    #[test]
    fn classify_project_malformed_when_args_hold_a_non_string() {
        assert_eq!(
            project("[mcp_servers.doctrine]\ncommand = \"doctrine\"\nargs = [\"serve\", 3]\n"),
            ProjectEntry::Malformed
        );
        assert_eq!(
            project("[mcp_servers.doctrine]\ncommand = \"doctrine\"\nargs = \"serve --mcp\"\n"),
            ProjectEntry::Malformed,
            "a bare string is not the array codex expects"
        );
    }

    #[test]
    fn classify_project_malformed_when_the_entry_is_not_a_table() {
        assert_eq!(
            project("[mcp_servers]\ndoctrine = \"x\"\n"),
            ProjectEntry::Malformed
        );
        assert_eq!(
            project("mcp_servers = 3\n"),
            ProjectEntry::Malformed,
            "mcp_servers itself is not table-like"
        );
    }

    #[test]
    fn classify_project_unreadable_when_the_text_is_not_toml() {
        assert_eq!(project("not = [toml\n"), ProjectEntry::Unreadable);
    }

    #[test]
    fn classify_project_unreadable_when_the_read_failed() {
        assert_eq!(
            classify_project(FileRead::Unreadable),
            ProjectEntry::Unreadable
        );
    }

    // --- plan (design sec-leg § Plan) ---

    fn env_ok() -> CodexEnv {
        codex_env(None, false, Some("/home/u"))
    }

    fn facts_ok() -> BakeFacts {
        bake_facts(Some(BAKE), Some(BAKE), Some(BAKE))
    }

    fn current_user() -> String {
        format!("[mcp_servers.doctrine]\ncommand = \"{BAKE}\"\nargs = [\"serve\", \"--mcp\"]\n")
    }

    fn plan_of(user: &str, project_text: &str) -> CodexUserPlan {
        plan(
            FileRead::Text(user),
            FileRead::Text(project_text),
            &facts_ok(),
            &env_ok(),
        )
    }

    #[test]
    fn plan_add_proposes_change() {
        let planned = plan_of("model = \"gpt-5\"\n", "");
        assert_eq!(
            planned,
            CodexUserPlan {
                user_config: Some(PathBuf::from("/home/u/.codex/config.toml")),
                bake: Some(PathBuf::from(BAKE)),
                user: Some(UserEntry::Add),
                project: ProjectEntry::Absent,
                degraded: None,
            }
        );
        assert!(planned.proposes_change());
    }

    #[test]
    fn plan_current_with_an_absent_project_does_not_propose_change() {
        let planned = plan_of(&current_user(), "");
        assert_eq!(planned.user, Some(UserEntry::Current));
        assert_eq!(planned.project, ProjectEntry::Absent);
        assert!(!planned.proposes_change(), "nothing to write anywhere");
    }

    #[test]
    fn proposes_change_for_an_owned_wrapper_with_a_current_user_entry() {
        // No user write, so no codex CLI is needed — the removal is doctrine's
        // own edit of its own project file.
        let mut env = env_ok();
        env.codex_on_path = false;
        let planned = plan(
            FileRead::Text(&current_user()),
            FileRead::Text(&wrapper_entry("")),
            &facts_ok(),
            &env,
        );
        assert_eq!(planned.degraded, None);
        assert!(
            planned.proposes_change(),
            "removing the old wrapper is a change even with the user entry current"
        );
    }

    #[test]
    fn plan_degraded_no_codex_cli_when_the_user_write_is_pending() {
        let mut env = env_ok();
        env.codex_on_path = false;
        let planned = plan(FileRead::Absent, FileRead::Text(""), &facts_ok(), &env);
        assert_eq!(planned.degraded, Some(Degraded::NoCodexCli));
        assert_eq!(planned.user, Some(UserEntry::Add));
        assert!(
            !planned.proposes_change(),
            "a degraded plan proposes nothing"
        );
    }

    #[test]
    fn plan_degraded_bad_codex_home_leaves_the_user_leg_unclassified() {
        let env = codex_env(Some(""), true, Some("/home/u"));
        let planned = plan(
            FileRead::Absent,
            FileRead::Text(&wrapper_entry("")),
            &facts_ok(),
            &env,
        );
        assert_eq!(planned.degraded, Some(Degraded::BadCodexHome));
        assert_eq!(planned.user_config, None);
        assert_eq!(planned.bake, None);
        assert_eq!(planned.user, None, "stopped before classification");
        assert_eq!(
            planned.project,
            ProjectEntry::OwnedWrapper { extra_keys: vec![] },
            "the project layer is classified regardless"
        );
    }

    #[test]
    fn plan_degraded_exec_not_on_path_keeps_the_destination() {
        let planned = plan(
            FileRead::Absent,
            FileRead::Text(""),
            &bake_facts(Some("/src/doctrine"), None, None),
            &env_ok(),
        );
        assert_eq!(planned.degraded, Some(Degraded::ExecNotOnPath));
        assert_eq!(
            planned.user_config,
            Some(PathBuf::from("/home/u/.codex/config.toml")),
            "step 1 succeeded, so the report can still name the file"
        );
        assert_eq!(planned.bake, None);
        assert_eq!(planned.user, None);
    }

    #[test]
    fn plan_degraded_user_unparseable_does_not_propose_change_even_with_a_wrapper() {
        let planned = plan_of("not = [toml\n", &wrapper_entry(""));
        assert_eq!(planned.degraded, Some(Degraded::UserUnparseable));
        assert_eq!(planned.bake, Some(PathBuf::from(BAKE)));
        assert_eq!(planned.user, None);
        assert_eq!(
            planned.project,
            ProjectEntry::OwnedWrapper { extra_keys: vec![] }
        );
        assert!(!planned.proposes_change());
    }

    #[test]
    fn plan_replace_proposes_change() {
        let user = "[mcp_servers.doctrine]\ncommand = \"/old/bin/doctrine\"\nargs = [\"serve\"]\n";
        let planned = plan_of(user, "");
        assert!(matches!(planned.user, Some(UserEntry::Replace { .. })));
        assert!(planned.proposes_change());
    }

    #[test]
    fn plan_path_shadow_with_a_current_user_entry_does_not_propose_change() {
        let planned = plan_of(
            &current_user(),
            "[mcp_servers.doctrine]\ncommand = \"doctrine\"\n",
        );
        assert_eq!(
            planned.project,
            ProjectEntry::PathShadow {
                command: "doctrine".to_string()
            }
        );
        assert_eq!(planned.degraded, None, "a shadow is warned, not degraded");
        assert!(!planned.proposes_change(), "we write nothing, warn only");
    }
}
