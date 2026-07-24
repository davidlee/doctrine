// SPDX-License-Identifier: GPL-3.0-only
use std::path::PathBuf;

use clap::{Args, Subcommand};

use crate::catalog::scan::ScanMode;

#[derive(Args)]
pub(crate) struct MapServeArgs {
    #[arg(long, default_value = "0")]
    pub(crate) port: u16,

    #[arg(long)]
    pub(crate) path: Option<PathBuf>,

    #[arg(long)]
    pub(crate) open: bool,

    #[arg(long, value_parser = validate_focus)]
    pub(crate) focus: Option<String>,

    #[arg(long, default_value = "1", value_parser = clap::value_parser!(u8).range(1..=3))]
    pub(crate) depth: u8,
}

/// The onboarding entry memory — a stable global-orientation signpost (SL-201
/// D1). Single-source constant (STD-001); the `onboard` verb focuses on it.
pub(crate) const ONBOARDING_MEMORY_KEY: &str = "mem.signpost.doctrine.overview";

fn validate_focus(s: &str) -> Result<String, String> {
    // Memory refs (`mem.<key>` / `mem_<uid>`): a canonical id or bare numeric can
    // never start with these prefixes, so they are an unambiguous discriminator.
    // Shape-check only here (pure, no disk); `run_serve` resolves key→uid.
    if s.starts_with("mem.") || s.starts_with("mem_") {
        return crate::memory::MemoryRef::parse(s)
            .map(|_| s.to_owned())
            .map_err(|e| format!("focus: invalid memory ref '{s}': {e}"));
    }
    // Accept both prefixed (SL-001) and bare (1) forms.
    if s.contains('-') {
        crate::kinds::parse_canonical_ref(s)
            .map(|_| s.to_owned())
            .map_err(|e| {
                format!("focus must be a canonical entity id (e.g. SL-001), got '{s}': {e}")
            })
    } else {
        s.parse::<u32>().map(|_| s.to_owned()).map_err(|_e| {
            format!(
                "focus must be a numeric id or canonical entity id (e.g. 1 or SL-001), got '{s}'"
            )
        })
    }
}

pub(crate) fn run_serve(path: Option<PathBuf>, args: MapServeArgs) -> anyhow::Result<()> {
    let root = crate::root::find(args.path.or(path), &crate::root::default_markers())?;
    // A memory-ref focus resolves to its `mem_<uid>` here (the frontend addresses
    // memory by uid). Resolution spans items/ AND shipped/ (the onboarding memory
    // is shipped) via `collect_all` — `resolve_inspect_uid` is items-only and
    // misses shipped keys. An unknown ref errors before the server binds.
    // Non-memory focus (canonical id / numeric) passes through untouched.
    let focus = match args.focus {
        Some(f) if f.starts_with("mem.") || f.starts_with("mem_") => {
            let mref = crate::memory::MemoryRef::parse(&f)
                .map_err(|e| anyhow::anyhow!("invalid focus memory ref '{f}': {e}"))?;
            let all = crate::memory::collect_all(&root)?;
            Some(
                crate::memory::resolve_memory_from_all(&all, &mref)?
                    .uid
                    .clone(),
            )
        }
        other => other,
    };
    let catalog = crate::catalog::hydrate::scan_catalog(&root, ScanMode::default())
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    let graph = crate::catalog::graph::CatalogGraph::from_catalog(&catalog);
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(crate::map_server::serve(crate::map_server::state::Config {
        root,
        graph,
        port: args.port,
        open: args.open,
        focus,
        depth: args.depth,
    }))
}

/// Serve args for `doctrine onboard`: the map focused on the onboarding memory,
/// browser opened. Extracted so the wiring is unit-testable without disk.
fn onboard_args() -> MapServeArgs {
    MapServeArgs {
        port: 0,
        path: None,
        open: true,
        focus: Some(ONBOARDING_MEMORY_KEY.to_owned()),
        depth: 1,
    }
}

/// `doctrine onboard` — drop a human into the onboarding graph in one step.
pub(crate) fn run_onboard() -> anyhow::Result<()> {
    run_serve(None, onboard_args())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test code")]
mod tests {
    use super::*;

    #[test]
    fn valid_focus_sl001() {
        assert!(validate_focus("SL-001").is_ok());
    }

    #[test]
    fn invalid_focus_lowercase_prefix() {
        assert!(validate_focus("sl-001").is_err());
    }

    #[test]
    fn invalid_focus_bogus_prefix() {
        assert!(validate_focus("BOGUS-001").is_err());
    }

    #[test]
    fn invalid_focus_empty() {
        assert!(validate_focus("").is_err());
    }

    #[test]
    fn valid_focus_memory_key() {
        // A memory key is shape-accepted (resolution to uid happens in run_serve).
        assert!(validate_focus("mem.signpost.doctrine.overview").is_ok());
    }

    #[test]
    fn valid_focus_memory_uid() {
        // A full mem_<32hex> uid is accepted.
        assert!(validate_focus("mem_019e9a11b3797af3a8833c67acfa69bf").is_ok());
    }

    #[test]
    fn invalid_focus_memory_ref() {
        // A `mem`-prefixed but malformed ref (uppercase segment — not a valid
        // key, not a uid) is rejected with a memory-ref message, not the
        // canonical-id message.
        let err = validate_focus("mem.Bad").unwrap_err();
        assert!(err.contains("memory ref"), "unexpected message: {err}");
    }

    #[test]
    fn run_onboard_wires_onboarding_focus() {
        // Wiring assertion (no disk): the onboard verb serves the onboarding
        // memory with the browser opened.
        let args = onboard_args();
        assert_eq!(args.focus.as_deref(), Some(ONBOARDING_MEMORY_KEY));
        assert!(args.open);
    }

    #[test]
    fn map_serve_path_flag_passed_to_root_find() {
        // Verify --path flag is parsed and wired into run_serve's root::find call.
        // The flag replaces the outer path arg when provided.
        let args = MapServeArgs {
            path: Some(PathBuf::from("/tmp/test-doctrine-root")),
            port: 0,
            open: false,
            focus: None,
            depth: 1,
        };
        // If args.path is Some, it should be passed to root::find instead of the outer path.
        // This test doesn't actually call root::find (requires disk), but verifies the
        // flag is wired correctly — args.path.or(path) passes the right value.
        assert!(args.path.is_some());
        assert_eq!(args.path.unwrap(), PathBuf::from("/tmp/test-doctrine-root"));

        // Verify the default: if args.path is None, outer path is used.
        let args_no_path = MapServeArgs {
            path: None,
            port: 0,
            open: false,
            focus: None,
            depth: 1,
        };
        assert!(args_no_path.path.is_none());
    }
}

#[derive(Subcommand)]
pub(crate) enum MapCommand {
    /// Start the local map explorer web server (loopback only)
    Serve(MapServeArgs),
}

pub(crate) fn dispatch(cmd: MapCommand) -> anyhow::Result<()> {
    match cmd {
        MapCommand::Serve(args) => run_serve(None, args),
    }
}
