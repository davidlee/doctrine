# Review RV-346 — design of SL-248

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

<!-- Pre-reading + lines of attack: what this review is probing, the invariants
     it must hold the subject to, and where the bodies are likely buried. Seeded
     at `review new`; the reviewer fills it before raising findings. -->

### Round 3 — sec-8 and sec-9

**Subject.** `.doctrine/slice/248/design.md` at design-run revision 60. The
review target is `sec-8` (code impact and verification alignment) and `sec-9`
(risks, residuals, and what stays open), both newly written and neither
externally read. `sec-6` and `sec-7` have also never had an external pass;
findings there are in scope **where they bear on sec-8's or sec-9's
correctness**, and out of scope as a general sweep.

**Standing discipline for this slice.** Build-level and confinement claims are
verified by **execution on a minimal reproduction**, not by reading. Round 1's
`F-1` (bubblewrap dereferences a `--ro-bind` source path) was found by running a
probe against flags that read as if they said the opposite. A negative read is
not evidence here.

**Lines of attack.**

1. **Is the modified-paths table actually the whole diff?** `sec-8` claims it
   is, and `sec-6` invariant 2 asserts it. Candidates it does not name:
   `Cargo.lock`, `clippy.toml`, `.github/workflows/*`, the `[package] include`
   allow-list, `justfile` recipes, `.gitignore`, `flake.nix`. Each absence is
   either correct or a finding.
2. **The `default-members` ruling.** `sec-8` claims one key gets `cargo fmt`,
   `clippy`, `build` and `test` onto the new crate with no recipe edit, and
   leaves `crates/cordage` where it is. Verify by execution. Then check what
   else it moves: `cargo package` / publish, `cargo install --path .`,
   `cargo binstall`, the crane/nix build in `flake.nix`, and any recipe that
   assumes `cargo build` produces one binary.
3. **A package with both a bin and a lib.** `sec-6` adds `src/lib.rs` to a
   package whose bin is also named `doctrine`. Does that build, and does it
   need an explicit `[lib]` or `[[bin]]` section? `sec-6`'s three
   execution-verified claims (E0364 on re-exporting `pub(crate)`, same module
   in both targets, the `crate::kinds` E0432 under the library's own test
   build) are stated as confirmed — re-run them if cheap, and say so either way.
4. **`R7`, the published library API.** `sec-9` claims `src/lib.rs` falls inside
   the published `include` list and that the `doctrine` package therefore
   acquires public semver surface at its next release. Check the claim and the
   rejected alternative.
5. **The requirement closure table.** `sec-8` moves `REQ-449` and `REQ-461` to
   `satisfied` and holds `REQ-448`, `REQ-450`, `REQ-459` as contributing
   changes. Read the requirements' actual criteria against the evidence each
   section offers. Two specific doubts: `REQ-461`'s only executed leg may report
   *skipped*, and `sec-8` attributes `REQ-448`'s denial half to table A "rows
   1–5" — check that against what `sec-2` and `sec-7` actually say.
6. **`R4`'s retirement.** `sec-9` retires the `bwrap_core_argv` byte-parity risk
   on the ground that `src/worktree/` is unedited under `DEC-155`. Find anything
   the capsule backend needs that only `jail.rs` has.
7. **The `#[ignore]` versus `--skip` distinction.** `sec-8` argues a recipe-level
   test filter is lawful under `DEC-156` while `#[ignore]` is not. Is that a real
   distinction or a rationalisation of a convenient escape hatch?
8. **`sec-9` residual 1, the resolution-time race.** It is dismissed as not
   capsule-reachable on the strength of `sec-2` invariant 4. Check the invariant
   covers what the dismissal needs it to cover.
9. **Internal contradiction.** Anything `sec-8` or `sec-9` asserts that
   `sec-1`–`sec-7` contradicts. `sec-8` already corrects the slice's own scope in
   four places; look for a fifth it missed, and for a correction it got wrong.
10. **The indicative counts** in `sec-8`'s evidence table. Labelled indicative,
    but a count that is wrong by a lot is a phase-sizing defect.
