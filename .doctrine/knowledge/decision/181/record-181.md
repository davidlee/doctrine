# DEC-181: Bin-target pub-use shim carries the lib export set

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## The obstacle

`SL-248` `PHASE-01` `EX-4` promotes five items from `pub(crate)` to `pub` so the
new `src/lib.rs` can re-export them: `read_path_at` and `CaptureError`
(`src/git.rs`), `today` (`src/clock.rs`), and `DOCTRINE_TOML` /
`read_doctrine_toml_text` (the new `src/config_file.rs`). `design.md` `sec-6`
records the promotion as compiler-required — `pub use` of a `pub(crate)` item is
`E0364` — and verified "by execution" on a minimal package.

That minimal package did not carry this workspace's lint policy. `Cargo.toml:90`
opts the root package into `[workspace.lints]`, which sets `warnings = "deny"`
and `unreachable_pub = "deny"` (`:189`) on the rustc side, and
`clippy::allow_attributes = "deny"` plus `allow_attributes_without_reason =
"deny"` (`:213-214`) on the clippy side.

`sec-6` also rules that `main.rs` keeps its own module tree — it declares
`mod git;` rather than importing `doctrine::git`, so `git.rs` compiles twice,
once per target. The lib target reaches the promoted items through `pub use`.
**The bin target does not**, so each promoted item is a `pub` in a private
module there, and `unreachable_pub` denies it.

## What was measured

A scratch package reproducing the arrangement under the same lint config. Four
combinations, all executed:

| approach | lib target | bin target |
|---|---|---|
| bare `pub` | passes | **fails** `unreachable_pub` |
| `#[expect(unreachable_pub, reason = …)]` | **fails** `unfulfilled_lint_expectations` | passes |
| `#[allow(unreachable_pub, reason = …)]` | **fails** `clippy::allow_attributes` | passes |
| `pub use` shim at the bin crate root | passes | passes |

The two targets impose opposite requirements on the same source line, and the
repo's clippy policy closes the usual escape hatch by banning `#[allow]` in
favour of `#[expect]`. So `EX-4` as written is unsatisfiable — not merely
awkward.

## The decision

`src/main.rs` carries a `pub use` shim mirroring the lib target's export set, so
the promoted items are reachable from the bin crate root as well. Measured clean
under `cargo build` **and** `cargo clippy`, including when the re-export path is
never referenced — a `pub use` is a re-export rather than an import, so the
`unused` deny does not fire on it.

The shim is inert: a binary crate has no external consumers, so nothing can
depend on the surface it nominally publishes.

## What this costs, stated plainly

`PHASE-01` `EX-3` says `src/main.rs` gains *exactly one* new declaration,
`mod config_file;`. It now also gains the re-export lines. `EX-3` and `EX-4` are
amended in `plan.toml` to say so; the ids are untouched (amend text, never
renumber).

`sec-6`'s claim that the visibility promotions are the whole of the change to
existing code — and invariant 2's "four visibility promotions, one module
relocation behind re-exports, and one test parameter" — are **incomplete as
written**. The design is not reopened for it. The correction is owed to the
reconciliation brief, alongside the corrections `sec-8` § Requirement closure
already carries.

## Alternatives rejected

- **Relax `unreachable_pub` for the root package only.** Defensible on the
  merits: `PHASE-01` `VT-1`'s export-set assertion
  (`the_root_library_exports_exactly_the_named_set`) bounds the crate's public
  surface far more tightly than `unreachable_pub` ever did, so the new gate
  arguably supersedes the lint for this package. Rejected because it trades a
  permanent repo-wide loosening for three lines saved, and because a lint
  relaxation outlives the reason for it.
- **Reopen the design to amend `sec-6`.** The locked design costs six section
  attestations plus two user acts to re-lock. Not worth it for a correction the
  reconciliation brief can carry.
- **`main.rs` as a thin binary over the library.** Already rejected in `sec-6`
  on its own grounds: it would force every `pub(crate)` item the command layer
  reaches to become `pub` (110 in `git.rs` alone, 203 in `memory.rs`).

## Evidence trail

The friction record for the verification gap — a design claim checked on a
minimal package that did not inherit the repo's lint config — is
`.doctrine/observations/records/cb/019fdf9f-64e1-7510-9f55-61fed7558fcb.toml`
(`RFC-011` instrumentation).
