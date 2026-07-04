# Design SL-201: Map focus on memory refs; onboarding command

<!-- Reference forms: entity ids padded (SL-201, SL-200); doc-local refs bare —
     D1/D2, OQ-1/OQ-2, VT-*. Scope: slice-201.md. -->

Downstream of the scope (`slice-201.md`). Two gaps: `--focus` rejects memory
refs, and there is no one-liner human onboarding entry. This design closes both
open questions and corrects a factual error the scope carried about the reuse
seam.

## Decisions

### D1 — Command surface: first-class CLI verb (OQ-1 resolved)

`doctrine onboard` — a thin top-level verb, no flags. It delegates to the
existing map serve path with the onboarding memory pre-focused and the browser
opened. Chosen over a `.agents/skills/` entry (agent-only, invisible to the
human newcomer the slice targets) and over "both" (a skill shim is cheap to add
later if agents want it — YAGNI now). A CLI verb is self-documenting in
`doctrine --help` and auto-exposed over MCP.

The entry memory is **hard-coded**, not configurable (YAGNI), as a single-source
named constant (STD-001):

```rust
/// The onboarding entry memory — a stable global-orientation signpost.
const ONBOARDING_MEMORY_KEY: &str = "mem.signpost.doctrine.overview";
```

### D2 — Focus seeding is server/CLI-side; frontend untouched (OQ-2 resolved)

Focus already flows CLI → hash with **no** frontend involvement:
`run_serve` → `Config.focus` → `open::map_url` → `#/focus/<focus>`
(`src/map_server/open.rs:16`). The frontend already resolves a `mem_<32hex>` uid
in that hash and renders `entry.title` on the node. So no `web/map/src` and no
`src/map_server/` change is needed: we only have to make the `focus` string a
resolved uid before it enters `Config`.

## Reuse seam (scope correction)

The scope cites `build_memory_key_map` in `src/catalog/hydrate.rs` for key→uid
resolution. **That symbol does not exist.** The real seam is two existing
`src/memory.rs` functions:

- **`MemoryRef::parse(s) -> Result<MemoryRef>`** (`memory.rs:1022`) — the ref
  *classifier*. `mem.<key>` → `Key`, `mem_<32hex>` → `Uid`, `mem_<8..32hex>` →
  `UidPrefix`; `SL-001` and bare numerics → `Err`. Pure, no disk — safe inside a
  clap `value_parser`.
- **`memory::collect_all(root)` + `memory::resolve_memory_from_all(&all, &mref)`**
  (`memory.rs:2834`, `:3052`) — resolve a `MemoryRef` to its `Memory` (take
  `.uid`) across **both** `items/` and shipped/ tiers. Needs `root`; runs in
  `run_serve`, not the value_parser. An unknown ref returns `Err`, surfaced
  before the server binds.

  > **Correction (impl):** an earlier draft named `resolve_inspect_uid`
  > (`memory.rs:2459`) here. That resolver is **items-only** — it misses the
  > *shipped* onboarding memory (`mem.signpost.doctrine.overview` lives in
  > shipped/, not symlinked in items/). `memory show` works because `run_show`
  > adds an explicit shipped fallback; `resolve_inspect_uid` has none. The
  > items+shipped union seam is `collect_all` + `resolve_memory_from_all` (the
  > path `run_resolve_links` uses).

No new parsing, no duplicated map.

## Current vs target behaviour

| Surface | Current | Target |
|---|---|---|
| `doctrine map serve --focus mem.<key>` | rejected by `validate_focus` | shape-accepted, resolved to `mem_<uid>`, `#/focus/mem_<uid>` |
| `doctrine map serve --focus mem_<uid>` | rejected | accepted, passes through |
| `doctrine map serve --focus SL-001 / 1` | accepted | unchanged |
| `doctrine onboard` | no such verb | serves map focused on `ONBOARDING_MEMORY_KEY`, opens browser |
| unknown `mem.` key | n/a | clear `Err` before serve; server never binds |

## Code impact (design-target)

- **`src/commands/map.rs`**
  - `validate_focus`: route memory refs to the classifier. A canonical id or
    numeric can never start with `mem.`/`mem_`, so a prefix guard is unambiguous:
    ```rust
    fn validate_focus(s: &str) -> Result<String, String> {
        if s.starts_with("mem.") || s.starts_with("mem_") {
            return crate::memory::MemoryRef::parse(s)
                .map(|_| s.to_owned())
                .map_err(|e| format!("focus: invalid memory ref '{s}': {e}"));
        }
        // ... existing canonical / numeric branches unchanged ...
    }
    ```
  - `run_serve`: after `root::find`, resolve a memory-ref focus to its uid before
    building `Config` (non-memory focus passes through):
    ```rust
    let focus = match args.focus {
        Some(f) if f.starts_with("mem.") || f.starts_with("mem_") => {
            let mref = crate::memory::MemoryRef::parse(&f)?;
            let all = crate::memory::collect_all(&root)?;         // items + shipped
            Some(crate::memory::resolve_memory_from_all(&all, &mref)?.uid.clone())
        }
        other => other,
    };
    ```
    Note: `--focus` seeds the initial `#/focus/<uid>` hash only under `--open`
    (existing `map serve` semantics — `map_url` is the sole focus consumer).
    `onboard` sets `open=true`, so its focus always lands.
  - add `const ONBOARDING_MEMORY_KEY` and a `run_onboard()` that builds a default
    `MapServeArgs { focus: Some(ONBOARDING_MEMORY_KEY.into()), open: true, .. }`
    and calls `run_serve` — the verb is a one-line delegation, no duplicated
    serve logic.
- **`src/commands/cli.rs`** — add an `Onboard` variant to the `Command` enum and
  a dispatch arm `Command::Onboard => crate::commands::map::run_onboard()`.
- **`src/commands/guard.rs`** — classify `Command::Onboard` for the worktree
  guard. `onboard` only serves (read-only); mirror `Map`'s classification for
  consistency (`guard.rs:59` treats `Map` as `Write("map")` — match it as
  `Write("onboard")` unless a Read classification is preferred; either is
  defensible since it never mutates authored state).

**Not touched** (narrower than the scope's guessed Affected Surface):
`src/map_server/*`, `web/map/src/*` — D2 shows focus already reaches the hash
untouched. The scope's `.agents/skills/<onboarding>/` is dropped by D1.

## Verification alignment

- **VT-1** `validate_focus("mem.signpost.doctrine.overview")` is Ok;
  `validate_focus("mem_<32hex>")` is Ok. (unit, pure)
- **VT-2** `validate_focus("SL-001")` / `"1"` still Ok; `"sl-001"` / `""` /
  `"BOGUS-001"` still Err — existing tests stay green, behaviour-preserving.
- **VT-3** a malformed memory ref (`"mem.Bad..Key"`, `"mem_deadbeef"` short
  prefix) → Err with a memory-ref message, not the canonical-id message.
- **VT-4** `run_onboard` builds `MapServeArgs` carrying
  `focus == Some(ONBOARDING_MEMORY_KEY)` and `open == true` — wiring assertion in
  the style of the existing `map_serve_path_flag_passed_to_root_find` test (no
  disk).
- **VH** manual: `doctrine onboard` opens the browser focused on the overview
  memory, node shows its title (not the uid); an unknown `--focus mem.nope.key`
  prints a clear error and never starts the server.

Resolution of the shipped onboarding key via `items/` (it is materialised by
`doctrine memory sync`) is confirmed by VT-4's follow-through in a rooted test if
one is cheap; otherwise VH covers it.

## Follow-Ups

- Optional `.agents/skills/onboard/` shim over `doctrine onboard` if agent
  invocation is later wanted (D1 defers this).
