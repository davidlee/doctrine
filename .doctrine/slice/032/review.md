# Code Review SL-032 — Worker guard, trunk-ref allocation, validate/reseat

Reviewer: code-review skill (adversarial). Date: 2026-06-10.
Gate state at review: `cargo build` clean, `cargo clippy` zero warnings,
738 unit + all e2e green.

**Overall**: acceptable

## Synopsis

Three near-independent additions — worker guard (D2a), trunk-ref allocation
machinery (D3), validate/reseat backstop — plus the memory-record warning. The
guard is genuinely good: exhaustive, wildcard-free, compile-error-on-new-verb.
But the headline D3 deliverable is **built and unwired**, the new identity table
is a parallel copy wearing a "single source" badge, and the largest test
artifact is brittle theatre.

## Findings

### 🟠 F-1 — trunk-ref allocation is dead in every production mint path
`src/entity.rs` / all `*::run_new`. Every real caller passes `&[]`:
```
src/slice.rs:207    &[], // trunk ids: production minting wires them in SL-031
src/backlog.rs:539  &[]
src/spec.rs:661     &[]
src/requirement.rs:238  &[]
src/governance.rs:338   &[]
```
The only live consumer of `git::trunk_entity_ids` is `integrity.rs:345`
(reseat's default `--to`). So `slice new` on a coordination branch behind trunk
**still collides** — the exact failure D3 names. There is **zero end-to-end
test** of a `new` picking up a trunk id (the git tests exercise
`trunk_entity_ids` in isolation; the integrity e2e runs in a trunkless tempdir
by design). The slice closure intent asserts "ids union local ∪ trunk via pure
`next_id`, with the peeled `^{commit}` ladder" — but no mint verb does this.
Scoping production wiring to SL-031 is defensible; silently shipping it inert
while the closure doc claims union allocation is live is not.

### 🟠 F-2 — `KINDS` is a parallel 12th identity table, not "the single" one
`src/integrity.rs:37`. Every `(dir, prefix)` already exists as the
source-of-truth `entity::Kind` const in the owning module (`slice.rs:39`,
`adr.rs:33`, `backlog.rs:60…`, `spec.rs:51`, `requirement.rs:43`,
`policy.rs:36`). `KINDS` re-types all eleven as raw string literals with no
compile-time link back. `kinds_table_covers_the_eleven` pins the table against a
hardcoded string list — it checks the copy against itself. A new numbered kind
added elsewhere silently escapes `validate` (self-acknowledged R-b at `:34`).
The memory `numbered-kind-identity-table` brands this "the single corpus-wide id
table" — it is the opposite: a new scatter point.

### 🟠 F-3 — `write_class_tests` is 330 lines of struct-literal theatre
`src/main.rs:1361`. Every assertion hand-builds a full `Command` variant with
all fields. Breaks on any field addition to any command — a change with no
behavioural relevance to classification. Bypasses clap, so it cannot catch a
CLI-wiring regression the way the argv-driven e2e (`e2e_worker_guard.rs`) does.
The compiler's exhaustiveness already proves totality; the marginal value is the
Read-arm labels, which a handful of argv-driven assertions would cover at a
fraction of the coupling. Tests implementation shape, not behaviour.

### 🟠 F-4 — reseat is non-atomic and bails non-zero on its own success path
`src/integrity.rs:375-427`. The mutation is six unguarded sequential fs ops
(rename dir → rename toml → rename md → rewrite toml → drop alias → plant alias)
with no rollback. A failure at step 4 leaves dir `045` whose toml still declares
`031` and a missing alias — worse than the start, from a repair tool. On the
happy path, a single inbound citation makes a fully-completed reseat `bail!`
(`:424`) after printing `reseated SL-031 → SL-045`. So `reseat && git commit`
never commits, and a naive retry hits `no SL-031` because the dir already moved.

### 🟡 F-5 — `has_runtime_state: bool` + hardcoded `.doctrine/state/slice`
`src/integrity.rs:365`. Guard keys on a generic boolean then hardcodes the slice
state path. A second stateful kind would satisfy the bool and check the wrong
directory. Carry the state dir on `KindRef`.

### 🟡 F-6 — `line_cites` trailing-alpha leak
`src/integrity.rs:457`. `after_ok` only rejects a following digit, so `SL-031x`
is reported as a citation of `SL-031`. Tighten to non-alphanumeric, matching
`before_ok`.

### 🟡 F-7 — `scan_danglers` globs disposable prose
`src/integrity.rs:434`. `.doctrine/**/*.md` sweeps gitignored/disposable files —
`handover.md`, `.doctrine/state/**` phase notes, `.doctrine/memory/items/**` —
and the reseated entity's own body. "Fix this by hand" against a `rm -rf`-able
handover file is noise.

### 👍 Good
`src/main.rs:965` `write_class` — wildcard-free, exhaustive,
compile-error-on-new-verb; `worker_mode` reads env at the shell edge; `next_id`
extracts a pure helper with a byte-identical-to-`candidate_id` proof (INV-1).
Pure/impure split (trunk ladder env-injected via `trunk_ladder`, F4 asymmetry as
a hard error) is clean and well-tested. The guard half is solid.

## Action items

1. State the D3-unwired reality explicitly in `audit.md` before close — the
   closure-intent wording currently overclaims (F-1).
2. Link `KINDS` to the source-of-truth `Kind` consts, or add a set-equality
   test, so a new kind can't silently escape `validate` (F-2).
3. Replace `write_class_tests` struct literals with argv-driven assertions;
   delete the redundancy (F-3).
4. Reconsider reseat's success-path exit code, or document the non-atomic /
   non-zero-on-success contract at the verb (F-4).
