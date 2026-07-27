# Design SL-232: Corpus-aware memory verify gate

## 0. How to read this document

**Status: reviewed on RV-314; one blocker open (F-2).** This replaces the
inherited SL-230 text wholesale. Amended after RV-314's adversarial pass —
**DEC-069/070/071** split measurement from reporting (§ 5.2), and **DEC-076**
settled the Revision routing (§ 5.6). § 10 carries the finding-by-finding state.

**Not implementable yet.** F-2 has no answer, and two verified findings are
settled in shape but open in detail: F-8's byte-domain call and F-7's exhaustion
classification. F-7's guard repair is a **prerequisite** to § 5.2's split, not a
parallel task.

### Reference legend

Three naming systems are in play. They are not interchangeable.

| Form | What it is | Where to look it up |
|---|---|---|
| `SL-` `DEC-` `QUE-` `ISS-` `IMP-` `REQ-` `REV-` `ADR-` `SPEC-` `POL-` `STD-` `RV-` `RFC-` | **Entities.** Durable ids, files on disk. | `doctrine <kind> show <ID>` |
| `OQ-` `D-` `E-` `I-` `R-` `T-` `V-` | **Doc-local labels**, meaningful only inside this file. `OQ` § 6, `D` § 7, `E`/`I`/`V` § 5.5, `R` § 8, `T` § 9. | this document |
| `RV-307 F-NN`, `RV-313 F-N` | **Findings on a review ledger** — always written with the ledger id. | `.doctrine/review/<n>/review-<n>.toml` |
| `FAL-` | **Falsifiers** registered in a probe header before the probe ran. | `probes/*.sh`, `probes/*.py` |

**Ledger findings are always qualified.** The inherited text declared once that
bare `F-NN` meant RV-307, then dropped the prefix throughout. That was safe when
RV-307 was the only ledger; it is not — RV-313 is cited here, and this slice
opens its own next. Every citation below names its ledger.

**Criteria ids are immutable.** `E12` is struck (withdrawn by DEC-020) and never
reused; `E10` was never minted. `I5` was never minted. New edge cases start at
`E14`, new invariants at `I10`, new decisions at `D12`, new tests at `T40`.
Retired tests keep their ids and are listed as retired, never renumbered.

### Evidence base

Every measured claim traces to an executable probe in `probes/`, each with its
falsifiers registered **in-header before the probe ran**. Re-run them rather than
trusting this prose. Corpus figures are stamped with the HEAD they were taken at,
because RV-313 F-1 caught a design-time absolute that failed to reproduce at
execution purely through corpus growth.

| probe | establishes |
|---|---|
| `route1.sh` `route2.sh` `route3.sh` | RV-307 F-37's three routes, reproduced |
| `shapes.sh` | the RV-307 R-A enumeration; `realpath` and contribution are uncorrelated **in both directions** |
| `candidate.sh` | the index-first rule against five falsifiers. **FAL-4 and FAL-5 failed** and are recorded as failures |
| `residue.sh` | what index-first does *not* close; `core.quotePath`, `core.ignoreCase`, the ancestor walk |
| `census.py` | claim-surface shape over the live corpus |
| `populations.py` | the decision populations the open items turned on |
| `control-chars.py` | RV-307 F-38's two obligations, separated by measurement |

Current figures at HEAD **`377022dfa`**: 389 tracked memories, 440 scope entries
(298 `paths` / 142 `globs`), 59 attested.

---

## 1. Design Problem

`memory verify` refuses to attest against a dirty working tree. Doing doctrine
work means the authored corpus is almost always dirty, so the common case is
self-inflicted: you cannot attest a memory because of the corpus edit you just
made — or, as observed live during the SL-230 design round, because of an
*unrelated* backlog file another agent left uncommitted. In practice agents hit
the refusal and reach for `git stash` rather than `--allow-dirty`, which is
undiscoverable and postdates the filing of IMP-221.

**The cost is measured, not asserted.** Of 59 attested memories, **24 carry a
`checkout_state_id` rather than a commit** in `verified_sha` — they were stamped
through the `--allow-dirty` escape hatch against a dirty tree (`populations.py`).
41% of this corpus's attestations are not commit-anchored. That is what the
current gate costs in practice.

The naive remedy — ignore doctrine's own authored trees — is **wrong**, and
RV-307 F-1/F-6 proved it on this corpus. Memory items live at
`.doctrine/memory/items/<key>/`, so a blanket exclusion removes *the memory being
verified* and stamps a commit that provably lacks the attested body. And **81
items declare `.doctrine/**` scopes**, so an ADR a memory explicitly names is
claim evidence exactly as `src/` is.

So the gate must be **claim-aware**: exclude unclaimed dirt, never claimed
evidence. Constructing that per-memory surface is the whole difficulty of this
slice, and it is where all 29 inherited findings live.

## 2. Current State

Line numbers are cited alongside symbol names because line numbers rot — prefer
the symbol. Verified at HEAD `377022dfa`.

| Surface | Behaviour | Site |
|---|---|---|
| `memory verify` | refuses on any dirty tree unless `--allow-dirty` | `memory.rs::run_verify` `:3484` |
| `stamp_verification` | writes `frame.commit`, **or `frame.checkout_state_id` under `--allow-dirty`**, into the *same* `verified_sha` field | `memory.rs::stamp_verification` `:3425`, branch `:3465-3470` |
| `capture()` | blanks the commit oid whenever the tree is dirty; yields a `checkout_state_id` hash instead | `git.rs::capture` `:2185` |
| Verification axis | `[review].verification_state`, `[review].reviewed`, `[git].verified_sha` — written **only** by `stamp_verification` | `memory.rs` |
| `memory validate` Check 2 | staleness = commits touching **scoped paths** since `verified_sha`; gated on `!scope.paths.is_empty()`, array passed raw; `None` binds in a let-chain and **falls out silently** | `memory.rs` `:3520-3531` |
| `memory validate` Check 4 | own-body drift; same silent-`None` let-chain | `memory.rs` `:3547-3578` |
| `retrieve::git_facts` | same raw scope seam, gated on `scope.paths.is_empty()`, feeding ranking | `retrieve.rs::git_facts` `:556` |
| `retrieve::staleness` | branch 1 gated on `!scope.paths.is_empty() && !verified_sha.is_empty()` — **the same predicate** `git_facts` gates on | `retrieve.rs::staleness` `:371` |
| `git::commits_touching` | ancestry guard: non-ancestor or bad object ⇒ `None`. **Correct, and stays.** | `git.rs::commits_touching` `:2493` |
| `coverage::IsStale` | `{Fresh, Stale, Unknown}` over the *same* seam, contract `None => Unknown` | `coverage.rs` `:150-166` |
| `collect_all` | unions `items/` and `shipped/` into one `Vec<Memory>`, erasing which root supplied each row | `memory.rs::collect_all` `:2934` |
| `fsutil::safe_join` | rejects absolute paths and `..`, but performs **no symlink canonicalisation** | `fsutil.rs::safe_join` `:20` |
| `memory::scrub_line` | escapes `\n`/`\r`/`\t` and every control char `< 0x20` — built for exactly the RV-307 F-38 hazard class | `memory.rs::scrub_line` `:2010` |
| `retrieve::is_global_reference` | the ADR-002 signature as a record-local predicate | `retrieve.rs::is_global_reference` `:345` |

**`capture()` has exactly three callers** — the retrieve read path, `record`, and
`verify`. Two of the three would be damaged by unconditional leniency, which is
why the exclusion is a parameter and not a change to `capture()`.

## 3. Forces & Constraints

| Authority | Constraint |
|---|---|
| **SPEC-007** | Asserts verify attests "against a clean working tree, refusing a dirty one" — amended by **REV-034**. Also carries the REV-041 clause binding `validate`. Full re-taken inventory in § 5.6. |
| **REV-041** (approved, done) | The five-state resolution is the **render contract**, binding `find`/`retrieve`. The prohibition on **silent over-trust is surface-independent** and binds `memory validate`'s health checks. "A surface that emits findings rather than states discharges this by emitting a finding, not by falling silent." This is objective 7's normative anchor. |
| **ADR-013** | Governance→work dependency routes through a Revision. `SL-232 needs REV-034` is authored. |
| **ADR-001** | `corpus_guard` = leaf, `git` = leaf, `memory` = command. Downward edges only. |
| **ADR-002** | The global/derived orientation class is repo-empty, unanchored, evergreen. Its scope is **not a claim about the querying repo's tree** — the basis for E14. |
| **POL-002** | The exclusion set must rest on doctrine-owned contracts, never host layout. Also why the anchor question must not be called "the code" (§ 5.4) — a client project's non-doctrine tree need not be code. |
| **STD-001** | Named constants, not path literals. Satisfied by reuse: `DOCTRINE_PATHSPEC` already exists. The two pathspec magic prefixes are likewise constants. |
| **SL-008 D6** | `thread_expiry` is reviewed canon — not loosened. |
| **DEC-020** | Non-contribution is reported and attested over, **never classified by a derived instrument**. Three were refuted. The stable answer is a *declared* boundary — objective 3. |
| **DEC-053** | The claim surface is built from the index, never the filesystem. Replaces the inherited ordered algorithm. |
| **DEC-054** | `validate`'s two unknowns unify; ISS-257 absorbed as objective 7. |
| **DEC-055** | One flat undeterminable state; `verified_sha`'s kind is not discriminated here. |
| **SL-230** | Owns the body-write seam and attestation invalidation. Not re-opened here. Its R4 runs unmitigated until this lands (DEC-027). |

## 4. Guiding Principles

- **The frame tells the truth.** `capture()` reports the literal state of the
  tree. Leniency is a *policy* applied by one consumer, never baked into the
  measurement.
- **Attestation is about the claim — the whole claim, and only the claim.** Dirt
  the memory does not declare says nothing about it; a change to a path it *does*
  declare says everything. Governance dirt is not exempt by being governance
  (RV-307 F-6/F-33).
- **A tool property is a claim needing a falsifier, not a premise.** "Stable",
  "total", "deterministic" must each be probed by varying the local state the
  instrument reads. Measuring that a discriminator *works* is not evidence it is
  *stable*. The named dominant cost driver of the eight rounds behind this text,
  and it caught two proposals during this design round.
- **Scope entries are untrusted data, never syntax.** SPEC-007 § Concerns treats
  stored memory text as hostile input.
- **A property the writer knows must not be re-derived by the reader from local
  repository state.** This design answers *"which instrument decides X?"* with
  *"none — record it at the source"* four separate times (§ 5.7). Every derived
  instrument tried in this slice's history reads state that a shallow clone, a
  pruned repo, a dispatch worktree, or a different object format legitimately
  disagrees about.
- **State a fix's value by what it makes impossible, not by what it fixes
  today.** Several mechanisms here add zero live coverage and exist for totality.
  Saying otherwise is the overclaim RV-307 F-25/F-33 punished.

## 5. Proposed Design

### 5.1 System Model

```
command tier   memory.rs ─┬─ run_verify              composes pathspec sets (policy)
                          └─ memory_health_findings  composes the same expander (policy)

leaf tier      corpus_guard.rs  DOCTRINE_PATHSPEC              (existing constant, STD-001)
               git.rs   observe_dirt(root, pathspecs) -> Dirt        ← the dirt primitive
                        Dirt::is_dirty() -> bool                     ← the cheap projection
                        capture_with(root, excludes) -> Frame        ← consumes a Dirt
                        capture(root) = capture_with(root, &[])      ← unchanged behaviour
                        expand_scope_entry(root, entry, magic)       ← the index-first expander
               memory.rs::scrub_line                                 ← existing report framing
```

Three elements at their correct altitudes: one parameterised dirt observation,
one per-entry index expander, and policy composition at command tier. **There is
exactly one dirtiness measurement** (`observe_dirt`, used twice by `verify` with
different pathspec sets, and once by `capture_with`) and **exactly one entry
expander** (`expand_scope_entry`, composed differently by `verify` and
`validate`).

The third element is objective 7's answer to RV-307 F-36, which the inherited
model left as an acknowledged hole.

#### The primitive returns an observation, not a bool (RV-314 F-11)

*Revised. The inherited model specified `dirty_under(root, pathspecs) -> bool`
with `capture_with` "delegating to it". That factoring cannot be built.*
`capture()` computes **three** artefacts and consumes all of them
(`src/git.rs:2230-2255`): `diff_bytes` → `worktree_fp`, `untracked_fp`, and
`index_tree` from `write_tree_with_retry` — all feeding `checkout_state_id`. A
`bool` discards every one, so `capture_with` would have to recompute them: the
parallel implementation D3 itself rejects.

```rust
pub(crate) struct Dirt {
    tracked:     Vec<u8>,        // `git diff HEAD --binary …` output, hashed by capture
    untracked:   Vec<String>,    // matched paths; NOT fingerprinted here
    index_dirty: bool,           // `diff-index --quiet --cached HEAD`
}
impl Dirt { pub(crate) fn is_dirty(&self) -> bool { … } }
```

**Untracked fingerprinting is deferred deliberately.** `capture_with` hashes the
untracked set when it needs a `checkout_state_id`; `verify`'s claim question
takes `is_dirty()` and never hashes anything. Without that split, closing F-11
would make every verify pay to fingerprint the entire untracked working tree —
closing the finding by making the verb expensive.

**I2 is preserved and was probed, not assumed.** All three legs are read-only;
each completes with `.git/index.lock` held. Only `capture_with`'s
`CheckoutState` branch reaches `write_tree_with_retry`, exactly where it does
today.

#### The legs must read raw bytes (RV-314 F-19/F-21/F-23/F-24, DEC-089)

The three legs answer *"is git satisfied?"*; **I6** claims *"the attested commit
contains these bytes"*. Anything separating those two questions is a hazard, and
there are two families: **content conversion** — git reads the file and
transforms it — and **freshness suppression** — git declines to read the file at
all. `NORMATIVE_FLAGS` (`src/git.rs::NORMATIVE_FLAGS`) pins `core.autocrlf`,
`core.eol` and `core.fileMode` so *machine-local config* cannot perturb the
frame: the right principle, applied to three keys out of the set that matters.
Committed `.gitattributes` is **committed content** and slipped that net
entirely. Measured on git 2.54.0 with the design's own flags:

| route | HEAD vs worktree | tracked | untracked | `diff-index --cached` |
|---|---|---|---|---|
| `text eol=crlf` | `…body\n` vs `…body\r\n` | 0 | 0 | 0 |
| `clean` filter → `CANONICAL` | `CANONICAL` vs **arbitrary attacker content** | 0 | 0 | 0 |

Both are tracked, non-ignored, stable and index-watched — outside DEC-070,
DEC-071 *and* DEC-082 alike. **I6 was false**, and the filter route hides not line
endings but arbitrary content.

**DEC-087's repair was refuted on three limbs in round 4 and is replaced by
DEC-089.** It put `--attr-source=<empty tree>` into `NORMATIVE_FLAGS`, which was
simultaneously **too wide** and **too narrow**, from one cause: the wrong
altitude.

- *Too narrow (F-21).* `--attr-source` replaces only the **tree** attribute
  source. `.git/info/attributes` and `core.attributesFile` still convert —
  measured, both hiding arbitrary content through a `clean` filter that was never
  committed. And `core.attributesFile` is **machine-local config**, which is
  precisely what § 5.1's own argument says `NORMATIVE_FLAGS` exists to close. The
  decision left open the category the mechanism it extends was built for.
- *Too wide (F-23).* `run_git_env` and `git_stdin` apply `NORMATIVE_FLAGS` to
  **every** invocation, including `check_attr_merge_z`, which backs SL-212's
  fail-**closed** `custom_merge_driver_paths` guard. Measured: a committed
  `f merge=ours` reads `ours` (custom ⇒ refuse) without the flag and
  `unspecified` (in that guard's `BUILT_IN` allow-list ⇒ allow) with it. A global
  flag silently inverted a safety guard in a subsystem this slice does not touch.
- *Unbuildable (F-24).* `NORMATIVE_FLAGS` is a compile-time `const &[&str]`; a
  per-repository derived oid cannot be a member of one, and deriving it needs an
  invocation that EX-1 routes through the chokepoint needing the value.

**The partition, not the list.** The repair is stated by *where the mechanism
lives*, so it answers for a mechanism nobody has found yet — the property DEC-082
was refuted for lacking:

| where it lives | doctrine's move |
|---|---|
| committed content — tree `.gitattributes` | neutralise: `--attr-source=<empty tree>` |
| machine-local config — `core.attributesFile`, `core.fsmonitor` | neutralise: `-c <key>=<off>`, `NORMATIVE_FLAGS`' founding principle at the right altitude |
| machine-local **default** — `$XDG_CONFIG_HOME/git/attributes`, read when `core.attributesFile` is *unset* | neutralise: the same `-c core.attributesFile=/dev/null`. Measured separately — `--attr-source` alone leaves it converting |
| system-wide — `$(prefix)/etc/gitattributes` | neutralise: `GIT_ATTR_NOSYSTEM=1` via `run_git_env`'s existing env seam. **The one unmeasured row** — see below |
| git-dir state — `$GIT_COMMON_DIR/info/attributes` | **no flag, config or env var reaches it** — detect and refuse |
| index state — `assume-unchanged`, `skip-worktree`, sparse, unmerged | refuse as **unmeasurable** (DEC-090) |
| ignore/exclude — `.git/info/exclude`, `core.excludesFile` | *deliberately untouched* — DEC-070 puts ignored files outside the evidence domain, so an operator-set exclude is a declared boundary, not a hole |

**Placement: every read whose contract is bytes.** The neutralising flags do
**not** join `NORMATIVE_FLAGS`. They are extra argv contributed by `observe_dirt`'s
argv builder — the single dirt observation § 5.1 already establishes.
`capture_with` consumes `observe_dirt` and `capture()` delegates to
`capture_with`, so **capture and verify still observe the same tree**: DEC-087
rejected scoping as "split-brain", and that objection is correct against a
*command-class* scope but does not reach a scope whose unit **is** the one
measurement. `check_attr_merge_z` does not route through `observe_dirt`, so
SL-212's guard is preserved by construction — and its suites are part of the
behaviour-preservation gate.

**`observe_dirt` is not the whole unit, and finding that out cost one probe
(self-attack, pre-round-5).** An earlier draft of this subsection said the flags
on `observe_dirt` cover *every* attribute-sensitive read `capture()` makes. That
is **false**. `capture()` makes a third such read outside the three legs:
`untracked_fingerprint` hashes each untracked path with
`git hash-object -- <path>`, which applies the clean filter **and** eol
conversion. Measured — two untracked files with entirely different content:

```
plain hash-object   a.dat = 3c79cdb822b066786a19331faffc066a4543efb3
                    b.dat = 3c79cdb822b066786a19331faffc066a4543efb3   ← collision
--no-filters        a.dat = 81920715936ccdb198cec402f62f990f5ec4838b
                    b.dat = d004ceeef7fd8d12daaca9febe3973d6767f1cba
```

So `checkout_state_id` cannot distinguish two distinct dirty states — and
`checkout_state_id` is the persisted `verified_sha` for **24 of this corpus's 59
attestations** (§ 5.4). This is not I6's false attestation; it is a **false
identity**, on the `--allow-dirty` path, and it is material.

**The remedy is narrower than the flags: `hash-object --no-filters`.** Measured
sufficient for the eol route as well — under `text eol=crlf`, plain `hash-object`
returns the LF-normalised oid (`0eabd516…`) while the raw CRLF bytes are
`126799cc…`, and `--no-filters` returns the raw oid. It needs no derived
empty-tree oid, so unlike the flags it carries **no CON-002 floor**. The general
rule this establishes, and the reason the subsection is titled as it is: *wherever
doctrine asks git for bytes, it must say so* — three sites, three spellings, one
contract.

**A third instance of the class is out of scope and routed, not fixed.**
`worktree_blob_oid` uses plain `hash-object` behind a doc comment promising "the
blob oid the **working tree** file would hash to", and `src/dispatch.rs:6122`
compares it against `blob_oid_at`'s *unfiltered* tree oid. Pre-existing, another
subsystem, and **unchanged by this decision** precisely because the flags stay off
`NORMATIVE_FLAGS` — filed as **ISS-261**. Worth noting that DEC-087 would have
incidentally *fixed* it while breaking `custom_merge_driver_paths`: the same
global reach doing both, which is the argument for placement in one sentence.

**EX-1 survives verbatim.** `run_git_env` remains the sole chokepoint applying
`NORMATIVE_FLAGS`; these flags arrive as ordinary `args` from one caller. No
second runner — and F-24 dissolves, because a runtime oid is now an *argument*
rather than a `const` member. The bootstrap has no cycle: `git hash-object -t
tree /dev/null` reads no attributes, so it goes through the plain chokepoint.
Measured: it writes no object (loose-object count unchanged), so **I2 holds**.
It must run **inside** the target repository — outside one it returns the sha1
oid unconditionally — and a well-formed but nonexistent oid is accepted silently
(`exit 0`), so a mis-derivation **fails open**. Derived once per observation; no
cache, hence no invalidation question in the long-lived MCP server.

**One row of that table is unmeasured, and says so.** `GIT_ATTR_NOSYSTEM` is the
only member of the neutralisation set not probed: `git var GIT_ATTR_SYSTEM`
reports `/etc/gitattributes`, which does not exist in this jail, and constructing
the fixture means writing to an `/etc` shared with other agents — declined. The
claim rests on git's documentation alone. It is flagged here so a later round
attacks it rather than inheriting it as measured; every other row was probed.
**An ambient `GIT_ATTR_SOURCE` is *not* a residual route** — measured, the
explicit flag wins over the environment variable.

**The residual is refused, not neutralised.** `$GIT_COMMON_DIR/info/attributes`
is reachable by nothing — measured under every combination of `--attr-source`,
`core.attributesFile` and `GIT_ATTR_NOSYSTEM`. So `verify` reads that one file at
the path git itself reports and **refuses legibly** when any line declares a
conversion attribute (`filter`, `text`, `eol`, `working-tree-encoding`, `ident`,
or legacy `crlf`). Patterns are deliberately **not** matched against the claim
surface — that is D18's rejected enumeration. The rule is coarse and over-refuses
in I9′'s recoverable direction. Note the common dir, not the per-worktree one:
measured, a linked worktree inherits the main repository's file, so E9's dispatch
case does not escape it.

Per-file alternatives were rejected: a raw-byte check on
`verified_sha:<uid>/memory.md` covers only the memory's prose, not the declared
evidence that carries the same hazard, and widening it to the whole claim surface
requires enumerating concrete paths — the 7,670-match expansion **D18 already
rejected**, then hashing each one. The **leg-swap** alternative (`diff-index`
without `--cached`) is reconsidered on the record rather than left as a silent
rejection: it still does not reach `info/attributes`, does not touch the
freshness family at all, and changes the leg whose byte-identity I1/T59 pin.

**Its cost is real and bounded.** In a repo legitimately using `* text eol=crlf`
every file differs from its blob, so the anchor leg reads dirty near-universally
and `verify` refuses. That is I9′'s recoverable direction, but it is the
usability failure DEC-070 rejected for ignored files, and **POL-002** makes a
CRLF client project a real case. Live population here is **0** — this repo has no
`.gitattributes`. The mitigation is diagnostic, not behavioural: **on the refusal
path only**, the un-neutralised probe runs too, and when the two disagree the
refusal names *attribute conversion* rather than blaming the operator's edits.
The happy path pays nothing. This lands inside objective 6's refusal-legibility
work rather than opening a new one.

**The freshness half is the same move (RV-314 F-22).** `core.fsmonitor` blinds
**all three legs** on a tracked, *unflagged* path on a stable checkout: measured,
an 11-byte blob against a 47-byte worktree reads `0 / 0 / rc=0` with `git status`
silent on the path, while `ls-files -v` shows `H` — so DEC-082's flags are not
involved and its enumeration never reaches it. It is config, so it neutralises
like config: `-c core.fsmonitor=false` restores the tracked leg completely,
**including on an index already primed by an fsmonitor-enabled `git status`, and
again after re-priming**. That the `FSMONITOR_VALID` bits do not survive the
config being off at read time is the fact this repair rests on, and it was
measured rather than assumed. The index-state members that no config reaches are
DEC-090's, not this decision's.

**Three consequences that must not be assumed away.** This touches `capture()`,
so the behaviour-preservation gate applies — I1 and T59's byte-identity
assertions are **re-run, not reasoned about**; in a repo with no `.gitattributes`
nothing changes and the suites should stay green unchanged, which is a claim to
demonstrate. `check_attr_merge_z` is **outside** the flag's reach by placement,
so SL-212's existing suites are the proof for F-23 and must stay green unchanged
too. And `checkout_state_id`'s inputs change in attribute-using repos, so
previously persisted values are not reproducible there — **`CHECKOUT_NORMALIZER`
therefore moves to `forget.checkout.v2`** (RV-314 F-28). The tag exists to version
this exact algorithm; leaving it at `v1` would label two different computations
with one name. A persisted value carrying `v1` is then legible as
pre-neutralisation rather than ambiguous, which is a third distinguishable state
for **IMP-325**, where discriminating `verified_sha`'s value kinds already lives.

**Absolute byte counts are fixture-dependent and are not restated here.** Rounds
2–3's figures do not re-run against fresh fixtures; the durable claim on every
row above is the **discrimination** — zero versus non-zero, `exit 0` versus
`exit 1` — and exact oids and exit codes do reproduce. `probes/README.md` carries
this rule; the probes are authoritative over the prose.

**It creates a git version floor doctrine has never declared — CON-002.**
`--attr-source` is git 2.40. Unmet, `verify` **refuses legibly**, naming the
requirement; it never degrades silently to un-neutralised probes, because an
attestation that cannot guarantee byte equality is what I6 exists to prevent —
the same principle as § 5.4's "a worthless stamp is worse than an extra
`git commit`".

**The floor binds callers by what they persist, not doctrine-wide (RV-314
F-25).** CON-002 as first written said the obligation was doctrine-wide "because
`NORMATIVE_FLAGS` applies to every invocation"; DEC-089 moves the flags off the
runner, and the premise goes with it. But the finding survives the move, because
`capture()` still routes through `observe_dirt`, and the three callers do three
different things with a failure — confirmed in source: `memory.rs::run_verify`
and `run_record` propagate with `?` (an opaque `CaptureError::Git`), while
`retrieve.rs::freeze` is `capture(root).ok()` and its own doc comment says it
*swallows* the error. One constraint, three behaviours.

The partition is **what the caller persists**:

| caller | persists a claim? | below the floor |
|---|---|---|
| `memory verify` | yes — `verified_sha` + `verification_state` | **refuse legibly**, naming CON-002 |
| `memory record` | yes — the born frame | **refuse legibly** — same requirement, same message |
| `retrieve::freeze` | no — a query decoration | **degrade to `None`, as B18/B19 already specify** |

`retrieve` is not an exception grudgingly admitted; degradation there is a
*settled decision from another slice* and it makes no attestation, so a thinner
visibly-`Unknown` result set is the correct behaviour and silence about the git
version is not a false claim about anything. What was wrong was `record`
propagating an opaque error under a constraint promising legibility. CON-002 is
amended to say this rather than "doctrine-wide", which it never was.

Detection is a **capability probe**
(`git --attr-source=<oid> rev-parse --git-dir`, non-zero ⇒ unsupported), not a
parsed version string: backports and vendored builds make version arithmetic
unreliable, and capability is the question that matters.

### 5.2 Two surfaces — measurement and reporting (DEC-069)

**Read this before § 5.2a's rule.** An earlier draft of this design built *one*
surface from the index and used it for two different jobs. RV-314 F-1 and F-10
proved that admits false attestation, and the repair is a split, not a patch.

#### The reframing

The inherited **I9** guaranteed *"every path in the claim surface is a real
tracked index entry"* — a **soundness** property: nothing false gets in. But the
hazard this slice exists to close is the opposite shape, **completeness**: real
evidence *omitted* from the surface, so nothing ever probes it. Both RV-314
blockers are completeness failures, and they survived eight adversarial rounds
because the invariant was watching the wrong direction.

The cause is a reuse. The index-first rule below was built to answer *"does this
entry contribute?"* — a **reporting** question — and the inherited § 5.2 then used
the same instrument to build the **measurement** surface. Two questions, one
instrument: the fifth instance of the error § 5.7 names four times, unnoticed
because both questions look like *"which paths?"*.

#### The legs were never the problem — the domain was

There are three predicates per path, not one: presence in **HEAD**, in the
**index**, and on **disk**. Over the **content/existence projection** —
`HEAD ∈ {absent, A} × index ∈ {absent, A, B} × worktree ∈ {absent, A, B}` — the
cube has 18 states, 16 of them dirty, and the three probe legs of § 5.1's `Dirt`
detect **all sixteen**. The decisive case is `HEAD=A, index=B, worktree=A` —
tracked diff `0`, untracked `0`, index diff `1` — which also proves the index leg
is not redundant.

**That enumeration is a projection, not a totality proof (RV-314 F-17, DEC-082).**
It establishes that the three legs are jointly *necessary*, not that they are
jointly *sufficient* over git's real state space. Dimensions outside the
projection were probed separately and the legs do catch them — mode-only
(`100644→100755`) and file→symlink changes produce tracked diff bytes; empty and
non-empty `add -N` produce diff bytes and index diff `1`; unmerged stages
likewise; submodules are rejected before hashing by `git.rs::reject_submodules`
and cannot attest cleanly. **Index-flag suppression is the exception, and it is
the third bound on I9′** — see below.

So measurement was not deficient *along the dimension F-1 and F-10 failed on*.
The **pathspec domain** was: under index-only construction an index-detached path
is absent from the surface, and no leg is ever *asked* about it. That is why F-1,
F-10 and F-11 are **one repair**.

#### The two surfaces

| | built from | consumed by | question it answers |
|---|---|---|---|
| **measurement** | uid dir ∪ declared entries (magic-prefixed) ∪ the symlink-target closure, emitted per DEC-080 | `verify`'s claim leg | *is the claim's evidence committed?* |
| **reporting** | the index expansion of § 5.2a's rule, unchanged | E7, the § 5.4 table, `validate` | *does this entry contribute?* |

**The uid directory base is the resolved directory — neither constructed nor
re-derived** (DEC-091, RV-314 F-18 → F-26). *This inverts two earlier drafts and
the inversion is the point; both are recorded rather than quietly replaced.*

The first draft built the base by string construction from `memory_uid`. **F-18**
showed the parsed uid was never bound to the *storage* identity: `resolve_show`
joins `items_root` with the reference, reads `dir/memory.toml`, parses it, and
returns the embedded uid **without checking it matches the directory it came
from**, so a memory in directory `A` declaring an independently-valid uid `B`
yields a base of `items/B` — a directory that need not exist — while `A`'s body,
*the prose actually read and about to be attested*, is measured by nothing.

The second draft repaired that with `read_link` on the **key symlink** plus an
equality check against the parsed uid. **F-26** showed that mechanism has no
object on two of the three references `verify` accepts. `MemoryRef::parse`
classifies into `Uid`, `UidPrefix` and `Key`; `resolve_show` takes the literal
name for `Uid`/`Key`, resolves a prefix for `UidPrefix`, then `fsutil::safe_join`s
— **a plain join with two rejections and no canonicalisation**. On the `Key`
route the joined dir *is* the symlink and `read_link` works; on the `Uid` and
`UidPrefix` routes it is the real uid directory and **there is no symlink to
read**. So F-18's false attestation survived on both unreached routes, and under
DEC-080/T68 an unmatched `:(literal)` pathspec is inert on all three legs — the
surface reads clean while the body is measured by nothing.

**The rule, and it deletes machinery rather than adding it:** `run_verify`
already binds `dir` from `resolve_show` and already load-bears on it three lines
later, stamping `dir.join("memory.toml")`. **That** path is the claim base, on
every route — with one symlink hop resolved when `dir` is itself a symlink, and
used as-is otherwise. The parsed `memory_uid` is **not consulted**, so there is no
identity to bind and no equality check to write: *the directory measured and the
file stamped are the same value by construction*, which was the actual harm in
both findings. A `read_link` target that is not a single bare component is
malformed and reported (I10), never silently followed.

**No `realpath`, no canonicalisation — DEC-053 is untouched.** One `read_link`
resolves *the memory's own storage location*, a different question from
classifying a declared pathspec's shape. Measured corpus shape: **349** key
symlinks, every target a single-component bare uid, none absolute, nested or
dangling.

F-10's requirement survives and is better served: a freshly recorded memory's key
symlink is untracked, and `resolve_show` reads the directory from disk, which does
not require the symlink to be tracked at all.

The two lexical defences the design used to lean on stay true and are **demoted
out of the safety argument** — the uid is a closed alphabet validated at parse
time (`is_uid`: `mem_` plus exactly 32 lowercase hex digits, both
`RawMemoryToml → Memory` paths bailing on failure), and `verify` is items-only by
resolution boundary. Neither is load-bearing for a base that is no longer
constructed.

*This finding is worth more than its repair.* F-18 was the third occurrence of
verifying a property that is **true but not load-bearing**; F-26 is the fourth
occurrence of the round-2 lesson — the repair was written where the finding
pointed rather than where the class lived. A uid disagreeing with its directory
remains a real corpus defect; it belongs to `validate`, not to `verify`, and is
routed there rather than dropped.

**Ordinary concrete index matches are not added to the measurement surface.** The
raw selector already measures those paths; expanding them adds only argv. The
`.doctrine/**` resolve alone is 7,670 matches — passing those back to git as
pathspecs is an argument-size hazard on every verify. Expansion contributes to
measurement **only** the symlink-target closure that git pathspec traversal
cannot supply, which is the part I7 exists for.

#### The closure emits; the index only discovers (DEC-080)

*This subsection is RV-314 F-15/F-16's repair.* DEC-069 moved the measurement
surface off the index for **declared** entries and left the **derived** ones
behind: § 5.2a step 4 read a matched symlink's target from its index blob and
then emitted only what `ls-files` matched. That prose admits two readings and
**neither is safe** — both reproduced on git 2.54.0:

| reading | failure |
|---|---|
| emit only index-matched targets | a target detached from the index (or never tracked but present and non-ignored) never enters the surface. Measured: the surface `[:(literal)link]` reports tracked `0`, untracked `0`, index `0` while the control emitting the target reports **145 bytes, index `1`**. This is **F-1 relocated one hop behind a symlink** |
| emit the joined target raw | the target string is author-controlled blob content. A tracked symlink whose blob reads `:(exclude)uid/**` **subtracts the mandatory uid directory**: tracked `0`, index `0` (false attestation) raw, versus **152 bytes, index `1`** when literal-prefixed |

Step 4 was doing two jobs. They separate:

1. **Emission (measurement).** Every lexically-eligible joined target is emitted
   **immediately** as `:(literal)<target>`, *whether or not the index carries it*,
   and *without any further test*.
2. **Discovery (traversal).** The index re-expansion is retained **solely** to
   find deeper mode-`120000` entries and continue the walk — bounded and
   cycle-checked as before.

**No coverage filter (RV-314 F-20).** An earlier draft restricted emission to
targets "not already covered by their originating selector". That wording is
struck. It was vestigial from the pre-DEC-080 draft, where emission was
index-conditioned and the restriction did real work; kept alongside unconditional
emission it contradicted the rule and left `covered` as an **undefined predicate
standing between a derived target and the surface** — a false positive there
reintroduces exactly the suppression this decision removes, under a new name. It
also bought nothing measurable: T68 establishes that an unmatched `:(literal)`
pathspec is inert on all three legs. The duplicate pathspec is cheaper than the
classifier that would avoid it.

Contribution **reporting** is untouched and stays index-first (DEC-053).

**Unconditional emission is safe on all three legs, measured not assumed.** A
`:(literal)` pathspec matching nothing anywhere returns `0` from `diff HEAD`,
`ls-files --others` and `diff-index` alike, and does not mask a real signal
emitted alongside it (`tracked=143 bytes, index=1` still comes through). The
over-approximation I9′ licenses is therefore inert, not merely tolerable.

**The lexical guard is not the mitigation — the constant prefix is.** I10 rejects
empty, control-character, absolute-outside and `..`-escaping shapes; it *accepts*
`:(exclude)uid/**`, which is relative, escapes nothing and carries no control
character. So F-7's repair does not close F-16. **I8 is restated** (§ 5.5) to bind
*derived* pathspecs as well as declared ones — § 5.2a's prefix rule was scoped
"per entry of `scope.paths` and `scope.globs`", and that scope was the defect.

**Why the closure is kept rather than deleted.** Dropping it from measurement and
pinning a boundary beside E15/R-H is lawful and would make both findings *cease
to exist* rather than be fixed — no derived pathspecs, no injection surface. It
was rejected on two grounds. First polarity: it concedes a **declared
under-approximation** in the very invariant this round exists to establish, and
lands asymmetrically, since reporting would still call such an entry
*contributing* while measurement never probes it. Second population: `census.py`
measures **0** symlink-rooted entries, but that is 0 *by accident*, not by
construction — doctrine mints a slug symlink for every entity and requires it
committed, and for memories the key symlink is the only ergonomic handle agents
hold. A memory scoping another memory or a slice reaches it through a link **by
default**, so the boundary would be pinned directly across the idiomatic path.

#### The replacement invariant

> **I9′ — the measurement surface may over-approximate the claim, never
> under-approximate it, within the declared evidence domain.**

The polarity is the whole point: over-measuring yields a **refusal**, which is
recoverable (`--allow-dirty`, or committing); under-measuring yields a **false
attestation**, which is not. **Two** bounds on it are not optional — **DEC-070**
names the evidence domain and **DEC-071** the temporal boundary; without either,
I9′ asserts more than it can deliver. Each is stated below rather than assumed.

The third position was held by DEC-082's index-flag exclusion and is **no longer a
bound**: DEC-090 converts it into a **refusal**, so I9′ now reads *"holds, or the
verb refuses"*. That is a strictly stronger statement, and it is stated as a
closure precisely because a bound is what F-22 falsified.

#### The evidence domain (DEC-070)

`git ls-files --others` answers differently with and without `--exclude-standard`,
so I9′ is undefined until the domain is named. **The domain is *tracked or
non-ignored commit-eligible* evidence — `--exclude-standard` stays on.** An
ignored-but-present file matching a declared entry is not claim evidence and does
not block.

Refuted on measurement, not preference: counting ignored files puts **19
`.doctrine`-scoped memories against 2,983 files**, and **39 memories against
15,319 corpus-wide**. Verification would be blocked near-universally by build
output and derived state. The principle underneath: an attestation names a
*commit*, so a file git is configured never to commit cannot be evidence for it —
this is a definition of the domain, not a concession within it. It also agrees
with the storage rule rather than working around it, since derived and runtime
state are gitignored by construction. Where the boundary genuinely bites,
`scope.unobservable` is the sanctioned declaration, which keeps it falsifiable
(V2) rather than silent.

**E8 is consistent, not in tension:** a gitignored but *tracked* path stays
evidence, because ignore rules do not bind tracked files. Tracked-ness dominates
ignore-ness.

#### The temporal boundary (DEC-071)

The three legs are **not an atomic snapshot** — measured: leg 1 clean, mutate,
legs 2 and 3 clean, final state dirty. I9′ is therefore scoped to *a checkout
stable for the duration of the probes*, stated rather than assumed.

This window is **inherited, not introduced**: `capture()` already sequences the
same three probes at `src/git.rs:2230-2239`, so every existing attestation
carries it. DEC-069 widens the pathspec domain, not the temporal one. Locking or
snapshotting was rejected — it would take `.git/index.lock` on the clean path and
destroy I2.

#### The third outcome: unmeasurable (DEC-090, replacing DEC-082's bound)

A path the index has been told to stop watching is invisible to **all three
legs**, on a stable checkout, while tracked. Measured on git 2.54.0 — a tracked
file modified on disk:

```
git update-index --assume-unchanged  → tracked 0, untracked 0, index 0
git update-index --skip-worktree     → tracked 0, untracked 0, index 0
```

Neither existing bound excuses this: the path is **tracked**, so DEC-070's domain
includes it, and the checkout is **stable**, so DEC-071 does not reach it. DEC-082
declared it a third **bound** on I9′ and pinned it with expected-blind tests.
**RV-314 F-22 refuted that one round later** — `core.fsmonitor` blinds all three
legs on a path carrying *neither* flag (`ls-files -v` reads `H`), so the bound was
an **enumeration**, not a principle, and I9′ was false again by the exact shape
DEC-082 was minted to close.

**The reframing.** The claim question was given two outcomes because it was
phrased *"is it dirty?"*. The verb's real question is *"can I attest this?"*,
which has **three** answers — yes, no, and **I cannot tell**. Once the third
exists, four open findings stop being separate defects and become its members:

| member | detected by | finding |
|---|---|---|
| `assume-unchanged` (`h`), `skip-worktree` (`S`), sparse-checkout (`S`) | `ls-files -v` tag ≠ `H` | F-17 / F-22 |
| unmerged entry — any stage ≠ 0 (`M`) | the stage field step 4 already reads | F-30 |
| an index pathname or derived target that is not valid UTF-8 | conversion failure at the emission boundary | F-8 / F-31 |
| a git probe that errors | non-zero exit | F-7 |

`verify` **refuses**, naming the cause. `validate` emits a **finding and
continues**, per B18's existing continuation policy. So I9′ changes polarity from
*"holds within these bounds"* to **"holds, or the verb refuses"** — a closure
rather than a bound, which is precisely what F-22 charged DEC-082 with failing to
be.

**It costs no new git invocation.** § 5.2a step 4 already runs
`git ls-files -s -z` per entry; `-s -v -z` carries the suppression tag *and* the
stage in the same output — measured layout `<tag> <mode> <oid> <stage>\t<path>`,
NUL-terminated. The uid-directory base adds one narrow invocation over its own
literal pathspec. **`observe_dirt`, `Dirt`, `capture_with` and `capture()` are not
touched**, so I1/T59 byte-identity holds and `retrieve::freeze`'s per-query cost
is unchanged — the classification rides the *constructor*, where the data was
already being read and thrown away.

**F-30 is closed by ordering, not by classification.** The unmerged entry is
caught from the stage field *before* step 4 calls `cat-file blob :<path>`, so the
`exit 128` the reviewer reproduced is never reached rather than being handled
after the fact.

**F-8's byte domain is settled by refusing, not by widening.** The alternative
horn — widening `run_git_env` to `OsStr`/`OsString` so non-UTF-8 pathspecs can be
emitted — was rejected on cost against population: it widens the single
chokepoint's signature across every caller, to *measure* a path `scrub_line` can
then only report in escaped form, for a live population of **0**. Refusing is
total, costs nothing, and is in the recoverable direction. **The domain stays
UTF-8 and the out-of-domain case refuses** — which is F-8's demand ("name the
domain or narrow I9′ honestly") answered, not deferred.

**DEC-082's measurement survives; its conclusion does not.** The two flags remain
exactly the states it measured — they are now refusals rather than declared
blindness. Two rejected alternatives are recorded because both are attractive.
*Honouring the flags* (DEC-082's own reasoning: "the flags exist precisely to tell
git to stop looking") is consistent for a **status** verb; `verify` issues an
attestation whose entire content is I6, and attesting over a path git was told to
ignore is the worthless stamp § 5.4 refuses to trade for an extra `git commit`.
*Redefining DEC-070's domain* to exclude these paths stays rejected as dishonest —
they are tracked, and hiding a boundary by redefinition is the opposite of
declaring it.

**Sparse checkout is a mainstream population here.** `git sparse-checkout set`
marks every out-of-cone entry `S` (measured), so a memory declaring a scope
outside the cone refuses rather than silently attesting. Correct, and a real
usability cost — stated beside R6 rather than buried.

#### What DEC-053 keeps

All of it, for **reporting**. No `realpath`, no character-based shape
classification, no whole-component-prefix rule; RV-307 F-37's three routes stay
closed. RV-307 F-27's *history-vs-now* cut is untouched — everything here is a
*now* question and nothing widens `commits_touching`. **E15/R-H is unchanged**: a
path reachable only through a symlinked directory matches nothing in HEAD or the
index, and `ls-files --others` does not descend symlinked directories, so the
known boundary neither closes nor widens.

#### Sequencing: RV-314 F-7 is a prerequisite

Step 4 manufactures pathspecs from index blob content. Unguarded, a derived
target of `/etc/hostname` or `../../outside` returns **exit 128 on all three
legs** — so this split *triples* the failing command surface until step 1's
lexical guard applies recursively to every derived path. F-7's repair lands
first, not alongside.

**F-7's second half — the exhaustion classification — is settled by DEC-090.**
The guard is lexical and prevents the reachable aborts; what remained open for
four rounds was what to *call* a probe that errors anyway. It is **unmeasurable**:
`verify` refuses, `validate` reports. That is a state distinct from *malformed*
(lexical, decided without reading repository state — I10/D16) and from
*non-contributing* (a real, empty match set). Classifying an unmerged entry as
malformed was rejected on exactly that line: stage number **is** repository state,
so putting it in the malformed bucket would be the derived-instrument
classification DEC-020 forbids.

### 5.2a The contribution constructor — index-first

**This section replaces the inherited ordered algorithm wholesale** (DEC-053). It
is not a fourth repair of it. *Its scope is now the **reporting** surface alone
(DEC-069); measurement is built as § 5.2 specifies.* The inherited rule classified an entry's shape from
its *characters*, canonicalised it with `realpath`, then asked git a question
about the index. RV-307 F-37's three routes are all one defect: **a filesystem
oracle answering an index question.**

Reproduced on git 2.54.0 (`route1.sh`, `route2.sh`, `route3.sh`):

| route | why the inherited rule missed it |
|---|---|
| `missing/../link` | git normalises `..` **lexically**; `realpath -e` requires the intermediate directory to exist |
| sparse checkout / `skip-worktree` | git matches the **index**; `realpath` requires the working tree |
| a literal filename containing `*` | git's `:(literal)` reads `*` as a character; the shape rule read it as a wildcard |

`shapes.sh` shows the two instruments are uncorrelated in *both* directions:
`missing/../link` and a sparse entry fail `realpath -e` and still contribute;
`linkdir/target.txt` resolves cleanly and contributes nothing.

#### The rule

Applied per entry of `scope.paths` and `scope.globs`:

1. **Guard lexically, before emission.** An entry that is empty/whitespace-only,
   contains a control character, is absolute-outside-the-repo, or escapes the
   root by `..` is **never emitted** (I10). It is reported as **malformed**
   (§ 5.5) and the run continues. Absolute-inside entries are rewritten
   repo-relative.
2. **Emit magic-prefixed by field of origin, never by character.**
   `scope.paths` → `:(literal)`, `scope.globs` → `:(glob)`. The schema already
   records the distinction the inherited step 2 was re-deriving unreliably. This
   is RV-307 F-37's structural correction and answers RV-307 F-32's returned
   contest at the root rather than at the split point.
3. **Expand against the index** — `git ls-files -s -v -z -- <spec>`. `-z` is
   **required, not stylistic**: `core.quotePath=true` renders `ünï.txt` as
   `"\303\274n\303\257.txt"` and corrupts any parsed output (`residue.sh` (d)).
   **`-v` is required too, and it is free** (DEC-090): it prefixes each row with
   the index state tag, giving `<tag> <mode> <oid> <stage>\t<path>` in one
   invocation — measured. A row whose tag is not `H`, or whose stage is not `0`,
   is **unmeasurable** and stops the entry there.
4. **Resolve matched symlinks from the index blob.** Every match of mode `120000`
   **at stage 0** has its target read via `cat-file blob :<path>`, joined
   **lexically** to the link's parent, put through step 1's guard, and
   re-expanded. Bounded and cycle-checked. The stage test is step 3's, not a
   second probe: an unmerged entry is classified from data already in hand, so
   `cat-file`'s `exit 128` on a stage-1/2/3 path is **unreachable** rather than
   handled (RV-314 F-30). *For the **measurement** surface the joined target is
   also emitted directly as `:(literal)<target>`, independent of what the
   re-expansion matches — see § 5.2's DEC-080. For **reporting**, which is this
   section's scope, only the re-expansion's matches count.* A target that is not
   valid UTF-8 cannot be emitted and is **unmeasurable**, not lossily converted
   (RV-314 F-31).
5. **Non-empty match set ⇒ contributes. Empty ⇒ non-contributing** — objective
   7's sink, declarable under objective 3. **Unmeasurable is neither** and is
   never declarable away: `unobservable` suppresses *reporting* of a
   non-contributing entry, and an entry git cannot be asked about is a different
   claim (I8).

Step 4 is what closes the sparse-checkout route, and it is load-bearing:
`cat-file blob :<path>` returns the link target **while the file is absent from
the working tree** (`candidate.sh` FAL-2, passed).

#### What this retires

- **Character-based shape classification.** The schema records path-vs-glob in
  the field name.
- **The whole-component-prefix rule.** Nothing is resolved before emission, so
  there is no prefix to split.
- **E13's mechanical-necessity basis** — see § 5.5.
- **Most of RV-307 R-A's enumerate-then-probe burden.** The obligation was
  discharged in method (`shapes.sh`), but the taxonomy is now
  **non-load-bearing**: shapes are no longer classified by us, so a fourth
  unenumerated shape has nothing to break. That is the substantive reason to
  prefer this over a repair — the three prior totality claims (RV-307 F-26, F-32,
  F-37) each failed by asserting a rule over an under-enumerated domain, and this
  rule has no domain to under-enumerate.

#### Stated honestly: what it buys, and what it does not

On this corpus the symlink-resolution step adds **zero** coverage for declared
scopes. 25 entries match symlinks, all self-covering; **0** entries are
symlink-*rooted* (`census.py`), which *confirms* the inherited "no glob
declaration is symlink-rooted" claim rather than refuting it. The full resolve
pass over `.doctrine/**` — 7,670 matches, 2,071 symlinks — costs **7ms**, so
scale is not a constraint.

*Corrected this round:* an earlier draft called the step "live and load-bearing
for the uid directory base, reached through one of 347 key symlinks". It is not —
the base is constructed from `memory_uid` (§ 5.2), which is both simpler and
survives the fresh-record case where the key symlink is itself untracked. The
step's measurement-side population is therefore **0 today**.

The value here is **totality by construction, not live defect count** — and here
that phrase is carrying real weight rather than decorating a null result, because
the 0 is an accident of the corpus rather than a property of it. Entity slug
symlinks are the handle agents hold, so a declaration reaching evidence through a
link is the *expected* future shape, not an exotic one. That is the argument
DEC-080 rests on.

#### Scope entries are data, not pathspec syntax (RV-307 F-18)

Interpolated raw, an entry of `:(exclude).doctrine/memory/items/mem_<uid>`
subtracts the mandatory uid directory from the claim surface and the attestation
goes through against a modified body. Demonstrated, not postulated:

```
git diff-index --quiet HEAD -- items/<uid>                            → exit 1  (dirty, correct)
git diff-index --quiet HEAD -- items/<uid> ':(exclude)items/<uid>'    → exit 0  (CLEAN — false attestation)
git diff-index --quiet HEAD -- items/<uid> ':(literal):(exclude)…'    → exit 1  (dirty — magic neutralised)
```

Git parses magic only at the head of a pathspec, so the prefix renders the
remainder inert. The uid directory is emitted the same way, so **nothing a memory
declares can subtract it** (I8). The two prefixes are named constants (STD-001).

**The same rule binds *derived* strings, not only declared ones (RV-314 F-16).**
A symlink's target is author-controlled blob content, so it is exactly as
untrusted as a `scope.paths` entry and reaches the pathspec set by a different
door. Emitted raw, `:(exclude)uid/**` as a link target reproduces the middle row
of the table above — tracked `0`, index `0`, a clean read on a modified body.
Scoping this rule "per entry of `scope.paths` and `scope.globs`" was the defect;
**every** string that becomes a pathspec is prefixed, whatever its source.

#### The base of the claim surface is the uid directory, one hop from resolution

(RV-307 F-15; settled by DEC-091.) `run_verify` resolves through
`fsutil::safe_join`, which performs no symlink canonicalisation, so a reference
given as a *key* yields `.doctrine/memory/items/<key>` — and every key in `items/`
is a symlink to the uid dir. **Git does not traverse symlinks in pathspecs**: such
a pathspec matches the symlink entry alone, so all three probe legs report clean
while the body is modified. Agents address memories **by key** (the boot snapshot
and `/retrieve-memory` both emit keys), so this is the mainstream path, not an
edge.

The base is therefore the directory `resolve_show` returned, with **one
`read_link` hop** taken when that directory is itself a symlink and none when it
is not — which is what makes the rule uniform across the `Uid`, `UidPrefix` and
`Key` routes rather than correct on one of them (§ 5.2, RV-314 F-26). "Canonicalised"
was the wrong word for it and is retired: there is no `realpath`, no ancestor
walk, and DEC-053 is untouched.

### 5.3 Data, State & Ownership

`observe_dirt` and `expand_scope_entry` return values and own no state; `Dirt` is
a plain owned observation with no interior mutability and no handle to the repo.
`MEMORY_SHIPPED_DIR` and `MEMORY_ITEMS_DIR` are both under `.doctrine`, so one
exclusion root covers them; only `MEMORY_MASTERS_DIR` (repo-root `memory/`) sits
outside, contributed only when it exists (E4).

**Objective 3 is the one schema change: `scope.unobservable`.**

```toml
[scope]
paths  = ["src/dispatch.rs"]
globs  = [".claude/skills/dispatch*/**"]
unobservable = [".claude/skills/dispatch*/**"]
```

| property | rule |
|---|---|
| type | `Vec<String>`, in the existing `[scope]` table |
| matching | **exact string equality** against `paths ∪ globs`. No pathspec semantics, no instrument, no local state |
| both-fields case | one entry text appearing in both `paths` and `globs` is covered in both by a single declaration. Live population: 1 (`src/dispatch.rs`) |
| effect | suppresses the **non-contribution report only**. Never subtracts from the claim surface |
| naming | not `untracked` (a git term of art for a *state*, and misleading given E8's force-added case) and not `external` (answers "is this part of the claim?" with *no*, which is wrong) |

**Validation rules**, all findings, none refusals:

- **V1** — an `unobservable` entry matching no member of `paths ∪ globs` declares
  nothing. Finding.
- **V2** — an `unobservable` entry that git **does** match is a stale
  declaration. Finding. *This is the falsifiability property that earned the
  shape*: the boundary is self-policing rather than a permanent silence.
- **V3** — empty/whitespace entries dropped and reported, exactly as § 5.2a step 1
  treats them in `paths`/`globs`.
- **V4** — duplicates deduped silently. Intra-field duplicates: 0 corpus-wide.
- **V5** — an `unobservable` declaration **never** suppresses a *malformed*
  finding. The escape hatch is offered only where it is the correct answer;
  declaring `../gone` unobservable would silence a broken declaration forever,
  and V2 could never fire to catch it because git will never match it.

**The producer (RV-314 F-2, DEC-081).** A declared boundary needs an authoring
verb. `memory edit` is the sole one, riding the existing scope-array arm exactly:

| joint | change |
|---|---|
| CLI | `#[arg(long, num_args = 0..=1)] unobservable: Option<Vec<String>>` on `MemoryCommand::Edit` |
| seam | `src/memory.rs:708-730` — a direct pass-through, *not* the `is_empty() → None` collapse the other three arms use |
| `EditFields` | `unobservable: Option<Vec<String>>`, plus a `has_any()` arm — without it `--unobservable` alone is "no fields given" |
| `apply_edit` | a fourth replace-whole-array block beside `paths`/`globs`/`commands`; `scope.insert` mints the key on demand — **but not copied verbatim: see the no-op clear below** |
| MCP — deserialiser | `unobservable: Option<Vec<String>>` on `EditParams`, the local struct inside `src/mcp_server/tools.rs::call_tool` |
| MCP — **advertised schema** | the `memory_edit` entry in `src/mcp_server/tools.rs::tools`. **A distinct joint** (RV-314 F-27) |
| `record` | **unchanged.** So is the embedded `memory.toml` template |

**The MCP surface has two joints, not one (RV-314 F-27).** An earlier draft named
only `EditParams`, which is not the public contract — confirmed in source:
`tools()` publishes the JSON input schema clients discover, while `EditParams` is
a separate local struct used only to deserialise incoming arguments, and
`unobservable` appears in neither today. Adding the field to the deserialiser
alone makes handler-level tests pass while MCP clients never discover it: the
field would be writable only by a client that *guessed* it. That matters more than
a missed line, because § 5.3's whole argument is that `memory edit` is the **sole
producer** of a declared boundary — and a producer half of whose surface is
undiscoverable is not a producer. T70 as written compares persisted TOML and so
exercises the deserialiser, not the advertised contract; it gains a
discriminating half asserting the field is **present in the published schema**.

**The clearing arm must not be copied verbatim (RV-314 F-32).** Confirmed in
source: `apply_edit`'s `--path-scope`, `--glob` and `--command` arms each set
`changed = true` unconditionally whenever the field is `Some`, with no comparison
against the existing value — while the function's own doc comment promises
"`updated` stamped ONCE at root if any field changed. Returns `true` iff any field
changed." For the siblings that no-op is **unreachable**, because the CLI collapses
an empty `Vec` to `None` before it arrives. DEC-081 deliberately removes that
collapse for `unobservable` in order to make clearing possible, which makes a
**no-op clear reachable for the first time**: `--unobservable` against an already
empty array would rewrite the TOML and stamp `updated`, against the stated
contract. So the fourth block compares before it writes. The fix DEC-081 chose for
one problem opened a behaviour the sibling arms never exposed — recorded here
rather than discovered at implementation.

**Semantics: replace, and clearable.** Replace matches the sibling arms, and the
coupling forces it — `unobservable` shadows entries in `paths ∪ globs` by exact
string match, so if those replace and this appended, the two would drift and V1
would fire on the residue. Clearing matters more here than for the siblings:
**V2's only remedy is deleting the entry**, and the typical memory declares one
`unobservable`, so the single-entry case *is* the mainstream case. Probed on
clap 4:

```
absent                             → None       (no edit)
--unobservable                     → Some([])   (clear)
--unobservable a                   → Some(["a"])
--unobservable a --unobservable b  → Some(["a","b"])
--unobservable a mem_x             → ref=mem_x, Some(["a"])   (value count caps at 1)
```

*Residual hazard, documented in the flag's help:* a **bare** `--unobservable`
placed immediately before the positional eats the reference. It can never be
silent — `<REFERENCE>` is the sole positional, so the outcome is always a hard
clap error with nothing written — but the message names a missing argument and
does not hint that the flag consumed it. `require_equals` and a separate
`--clear-unobservable` flag were both rejected (DEC-081): the first gives one of
four sibling flags a different call syntax, the second admits a contradictory
input needing its own refusal rule. *Not a second clearing route:*
`--unobservable=` yields `[""]`, which **V3** drops and reports — identical to
`--glob=` today.

**Absent means empty; there is no migration.** `RawScope` gains
`#[serde(default)] unobservable: Vec<String>` and `Scope` the matching field.
Reads are already safe by construction: `scope_array` returns empty for a missing
key, and the only `toml::to_string` round-trips in `src/memory.rs` are in tests —
production writes are all `toml_edit`, which preserves unknown keys. The field
has exactly **one reader**, at command tier, in `validate`'s contribution
reporting.

**No lexical guard at write time.** `--unobservable ../gone` writes; `validate`
reports it malformed and V5 refuses to let the declaration suppress that. Same
posture as `paths`/`globs`, and consistent with "all findings, none refusals".

**No corpus backfill in this slice**, and not as a punt. `check-ignore` is a
local-state instrument — which is *precisely why* this boundary is declared
rather than derived — so a mechanical bulk backfill of the ~33 candidates would
be the derived judgement the field exists to refuse. The intended operating mode
is one declaration in response to one finding. F-4's disposition already re-based
T49 onto a constructed `GitScratch` fixture, so nothing in the test matrix needs
the backfill either. The population routes to the backlog, HEAD-stamped.

**It does not clear the attestation.** SL-230 D4/D8 clear the verification axis on
a *claim-field* edit. `unobservable` changes reporting, not measurement, so it is
not a claim field. This falls out of the "never subtracts" rule and is a useful
consistency check: if editing it *had* to clear the attestation, the
parallel-assertion shape would be wrong.

Mechanically this is **by construction, not by rule**: `ClaimSnapshot`
(`title`, `summary`, `paths`, `globs`, `commands`) stays five fields, the fourth
`apply_edit` block writes the array, and `claim_snapshot` never reads it. There
is no code path that could clear the axis, so nothing has to remember not to.

**Sizing, with its instrument named.** Of 59 non-contributing entries, **33 have
a root this checkout ignores** and 26 do not (`populations.py`). So declarations
could plausibly convert 59 undifferentiated reports into ~26 actionable findings.
This is an **estimate, not a target**: `check-ignore` is itself a local-state
instrument, which is exactly why the boundary must be *declared* rather than
derived. The earlier figure of 20 declarable (39 actionable) does not reproduce;
it used a fixed root list that omitted `.agents/skills/**`, `.mcp.json`,
`.worktrees/**`, `docs/claude/workflows.md` and `web/map/dist`.

**Rejected shapes**, on the record: a sigil inside the entry string
(character-sniffing, the exact error § 5.2a deletes, and it collides with real
filenames); a per-memory flag (too coarse — the typical memory declares several
paths and one unobservable); a separate `external` list (see naming above); an
array-of-tables carrying a `reason` (documentation, not mechanism — and if wanted
later, a parallel field keyed by the same exact match is additive).

### 5.4 Lifecycle, Operations & Dynamics

#### `verify` — two questions, both of which must pass

```
if allow_dirty {
    let full = capture(root)?;      // UNEXCLUDED — the real state of the tree
    stamp(full);                    // Commit if genuinely clean, else CheckoutState
} else {
    let anchor = capture_with(root, corpus_excludes)?;               // 1. ANCHOR question
    let claim_dirty = observe_dirt(root, claim_pathspecs)?.is_dirty(); // 2. CLAIM question
    match (anchor.anchor_kind, claim_dirty) {
        (Commit, false) => attest against anchor.commit,    // the only success
        _               => refuse, naming which question failed,
    }
}
```

**The two questions are named on substance** (RV-307 F-39 limb 1). The inherited
text called the first "is the code dirty?" at three sites, contradicting § 4's
claim-not-code boundary. It is worse than a wording slip: the first question
excludes `.doctrine/**` and `memory/`, so what remains is *everything else* —
which in a client project may be docs, assets or config. Calling it "the code"
bakes in a host-project assumption that **POL-002** prohibits. The questions are:

- **the anchor question** — is the tree *outside doctrine's own authored corpus*
  clean enough to anchor an attestation?
- **the claim question** — is the claim's own evidence committed?

| Set | Contents |
|---|---|
| `corpus_excludes` | `:(exclude)` + `DOCTRINE_PATHSPEC`; plus `:(exclude)memory` **only when that directory exists** |
| `claim_pathspecs` | the **measurement surface** of § 5.2 (DEC-069): the memory's own **uid** directory (the resolved one — DEC-091), plus every declared `scope.paths` / `scope.globs` entry emitted magic-prefixed by field of origin, plus **every** resolved symlink target, emitted unconditionally and without any further test (DEC-080). *Not* the full index expansion — see § 5.2 |

**Why `--allow-dirty` re-captures unexcluded** (RV-307 F-13). Both `Commit`
branches of `capture` leave `checkout_state_id` empty; only the dirty branch
computes one. A claim-only-dirty tree therefore yields a `Commit`-anchored frame
carrying no `checkout_state_id`, and the claim leg is deliberately a bool (I2). The
escape hatch would have had nothing to stamp. Taking the anchor from an
unmodified `capture(root)` makes I4 literally true. It is **not an extra
capture**: the `allow_dirty` branch is taken *before* the gate probes.

**This costs the `record` → `verify` convenience, deliberately** (RV-307 F-1). A
freshly recorded memory's directory is untracked, so `verify` refuses until it is
committed. The alternative was a `verified_sha` naming a commit that provably did
not contain the attested prose. A worthless stamp is worse than an extra
`git commit`.

**Refusal legibility.** The current message never mentions `--allow-dirty`. At the
one moment an agent is looking for the escape hatch, the tool hides it and
prescribes committing. The refusal names its own flag (objective 6).

#### `validate` — one mechanism, two unknowns (objective 7)

**D11 is falsified.** The inherited decision said `validate` "keeps its existing
raw seam"; that cannot survive objective 7, which must touch the same two call
sites (DEC-054). Its four-defect enumeration was also incomplete — the
`None`-swallow is a fifth, the largest by population and the only one that is
**non-conformant** against amended SPEC-007 rather than merely weak. Both are
restated here rather than silently corrected, the discipline RV-307 F-34/F-39
established.

**ISS-257's remedy — the tri-state.** Checks 2 and 4 bind `commits_touching` in a
let-chain, so `None` falls out and emits nothing. *Cannot determine* renders as
*no drift*. The correct shape already exists one module away: `coverage.rs`'s
`IsStale{Fresh, Stale, Unknown}` over the **same** seam with the contract
`None => Unknown`. **Ride it; do not re-invent it.** The ancestry guard in
`commits_touching` is **correct and stays** — a non-ancestor `since` over-counts
a set difference, so `None` is the documented no-over-trust posture. The defect
is in how the callers consume it.

**The state is flat** (DEC-055). `None` has three live causes — non-ancestor
commit (8), dangling object (2), and a `checkout_state_id` that was never a commit
at all (24) — and `validate` reports one undeterminable state for all of them.
Discriminating them is routed as **IMP-325**. The rejected shortcut (split by
stamp width) is falsified: `git init --object-format=sha256` yields 64-hex commit
ids, so the rule fails totally on that class of repo. Recorded because the idea is
attractive and cheap-looking, and the next reader will re-derive it.

**Population, corrected.** 34 of **59 attested** memories are silently
unstaleable; reach 42.4% (`populations.py`). The scope document's "67 of 115
anchored" used the wrong denominator — Checks 2 and 4 both gate on
`!verified_sha.is_empty()`, so *attested* is the code-relevant set. The ratio
survived re-measurement; the absolute was overstated roughly twofold.

**F-36's remedy — the contribution probe.** `validate` composes
`expand_scope_entry` over `paths ∪ globs` and asks only *empty or not*.

***This does not reopen RV-307 F-27, and the distinction is load-bearing.*** F-27
holds that `verify`'s surface must not be reused for a *historical* question —
canonicalising against today's checkout erases a committed symlink retarget
(measured 1 → 0). Contribution is a **now** question: *does this entry match
anything in the index today?* Both verbs ask it identically. What stays unshared
is the drift seam (`commits_touching`). **The cut is history-vs-now, exactly as
F-27 drew it — not verify-vs-validate.**

***RV-307 F-28's cost objection also does not reach this limb.*** The inherited
constructor needed `(root, memory, dir)`, and `collect_all` discards provenance.
Contribution needs no `dir`: the uid directory is `verify`-only, and Check 4
already builds its body path from `memory.uid`, canonical by construction. The
signature is `(root, entry, magic)`. No dataflow change, no `collect_all` touch.
F-28 remains correct about IMP-317 limb (b); it simply does not apply here.

**Continuation policy** (RV-307 F-29) — **B18's precedent, not a new posture.**
Per-entry, per-memory: a failure degrades that entry to a finding and the run
continues. Two precedents already in the tree: `retrieve::git_facts` ("a
`commits_touching` failure is per-candidate, never a query abort") and
`coverage_scan` (degrades cells to `Unknown` rather than dropping them).

**The verify/validate asymmetry, which is F-29's actual answer:**

| entry outcome | `verify` | `validate` |
|---|---|---|
| malformed (empty / control char / escaping / absolute-outside) — **lexical** | report, then attest | finding, continue |
| **unmeasurable** · index tag ≠ `H` (`assume-unchanged`, `skip-worktree`, sparse) | **refuse**, naming the cause | finding, continue |
| **unmeasurable** · stage ≠ 0 (unmerged) | **refuse**, naming the cause | finding, continue |
| **unmeasurable** · pathname or derived target not valid UTF-8 | **refuse**, naming the cause | finding, continue |
| **unmeasurable** · probe errored (git failure) | **refuse** — cannot attest what it cannot measure | finding, continue |
| matches nothing, not declared | stderr report, then attest | finding |
| matches nothing, declared `unobservable` | silent | silent |
| matches, declared `unobservable` | stderr report (V2) | finding (V2) |
| matches | must be clean, else refuse | no finding |

The asymmetry is principled, not convenient: `verify` attests one memory, so
refusing is available and correct; `validate` surveys the corpus, so refusing one
row destroys the survey.

**The four `unmeasurable` rows are one outcome, not four** (DEC-090). They are
listed separately only because each is *named* separately on the refusal path —
objective 6's legibility — and they collapse to a single classification in the
constructor. Note the split against *malformed*: malformedness is decided
**lexically**, without reading repository state (I10/D16), which is what keeps
D16 clear of DEC-020; every row above it reads repository state and therefore
cannot join that bucket. **This resolves the D10-versus-table contradiction in
the table's favour** — see § 7.

**Cost.** Roughly 440 added `ls-files` invocations. Measured baseline: `memory
validate` currently takes **73s** on this corpus, 99% user CPU, dominated by an
unrelated O(relations × corpus) rescan filed as **ISS-258**. Against that
baseline the probe is noise; against the sub-second baseline ISS-258's fix
produces, it becomes the dominant term. **Re-measure after ISS-258 lands** — this
is a re-measure trigger, not a settled figure.

#### Objective 4 — IMP-317 limb (a), and a lockstep that must not break

`validate` Check 2 and `retrieve::git_facts` both gate on
`!scope.paths.is_empty()` and pass the array raw. Limb (a) widens both to
`paths ∪ globs` and neutralises pathspec magic before either reaches
`commits_touching`. This fixes the **13 of 43** scoped-and-attested memories that
are glob-only and therefore ranked on a 30-day calendar instead of by commits
touching their evidence (QUE-175, answered `yes` on measurement), and closes the
RV-307 F-18 injection route into the historical seam.

**`retrieve::staleness` branch 1 must widen with it.** It gates on the *same*
predicate `git_facts` gates on. Widening `git_facts` alone changes nothing
observable — the glob-only memory would still fall through to the time branch.
The lockstep is now an invariant (I11): a hypothesis that these two could
disagree was **refuted** during the design round precisely because they share the
predicate, and widening one without the other would reintroduce the collision.

**This is not a shared surface, and F-27 is untouched.** Limb (a) widens the raw
seam's *input* and neutralises it; it resolves nothing. Limb (b) — own-directory
drift in the historical seam — stays routed as IMP-317, and F-28's dataflow cost
stands there.

### 5.5 Invariants, Assumptions & Edge Cases

#### Invariants

- **I1** — the three existing `capture()` call sites see byte-for-byte identical
  frames. Guaranteed by construction (`capture` delegates with `&[]`).
- **I2** — the clean-after-exclusion path never calls `write-tree`, so it takes no
  index lock. `observe_dirt`'s three legs are all read-only and none computes
  `checkout_state_id`, so the *claim* probe never reaches `write_tree_with_retry`
  even when the claim surface is dirty. **Probed, not assumed** (RV-314): each leg
  completes with `.git/index.lock` held. Only `capture_with`'s `CheckoutState`
  branch takes the lock, exactly where it does today.
- **I3** — a genuinely dirty **anchor** tree still refuses without
  `--allow-dirty`. (See OQ-5: this invariant is what OQ-5 would delete.)
- **I4** — `--allow-dirty` semantics unchanged: it bypasses **both** gate
  questions and stamps the frame from an **unexcluded** `capture(root)`.
- **I6** — a successful attestation's `verified_sha` **contains the attested
  body**, asserted as **byte equality**, not existence: any stale ancestor blob
  satisfies `cat-file -e` (RV-307 F-14). **Kept, not narrowed — but it was false
  as delivered until DEC-087, and still false under DEC-087 (RV-314
  F-19/F-21/F-22).** Byte equality is a claim about *bytes*; the three legs
  measured git's view, which content conversion and freshness suppression each
  separate from the bytes. It holds now on **three** legs of support, not one
  (§ 5.1): the conversion sources reachable by flag, config or environment are
  **neutralised at `observe_dirt`** (DEC-089); `$GIT_COMMON_DIR/info/attributes`,
  which nothing reaches, is **detected and refused**; and the index-state family
  is **refused as unmeasurable** (DEC-090). It holds **only above CON-002's git
  floor** — below it `verify` and `record` refuse rather than asserting something
  they cannot deliver, while `retrieve` degrades, because it asserts nothing.
- **I7 — restated for the two surfaces** (DEC-069). The inherited form ("the claim
  surface names **real tracked files**, never a symlink standing in for them") is
  false of the measurement surface, which deliberately carries *selectors* and
  tracked symlink entries. The property that survives: *a matched symlink is
  measured **itself**, and its eligible target closure is measured **in
  addition**.* Rooted at the uid directory, which is what stops a key-form
  reference measuring the symlink instead of the body (RV-307 F-15) — constructed
  from `memory_uid`, not resolved through the key symlink (§ 5.2). **Sharpened by
  RV-314 F-15 / DEC-080:** "in addition" means *emitted unconditionally*, not
  *emitted if the index also carries it*. The index-conditioned reading is an
  under-approximation and reproduces F-1 behind a symlink.
- **I8 — nothing a memory *declares or derives* can subtract from what it is
  *measured against*. Binding on all three legs, and on every string that becomes
  a pathspec whatever its source.** Entries are emitted magic-prefixed;
  `unobservable` suppresses reporting only, never measurement. *Widened by
  DEC-069:* declared entries now reach the measuring probes directly, so
  magic-prefixing must be applied on the tracked, index **and untracked** legs
  alike. Probed on all three: `:(literal):(exclude)<uid dir>` is neutralised
  everywhere, and `ls-files --others` still returns the uid body. The inherited
  demonstration exercised only `diff-index`; that was never sufficient.
  *Widened again by RV-314 F-16 / DEC-080:* the rule was scoped to what a memory
  **declares**, leaving symlink targets — author-controlled index blob content —
  unprefixed. Reproduced: a link whose blob reads `:(exclude)uid/**` subtracts the
  uid directory on all three legs when emitted raw. I10's lexical guard does
  **not** catch it (the string is relative, escapes nothing, holds no control
  character), so the constant prefix is the sole mitigation.
- ~~**I9**~~ — **struck and replaced by I9′.** The inherited I9 asserted
  *soundness* ("every path in the claim surface is a real tracked index entry")
  where the hazard is *completeness*. It was polarised backwards, which is why
  RV-314 F-1 and F-10 passed under it. Struck id, never reused.
- **I9′ — the measurement surface may over-approximate the claim, never
  under-approximate it, within the declared evidence domain — *or the verb
  refuses*** (§ 5.2, DEC-069). Bounded by **DEC-070** (tracked-or-non-ignored
  commit-eligible) and **DEC-071** (a checkout stable for the duration of the
  probes). The former third bound, DEC-082's index-flag exclusion, is **replaced
  by a closure**: under **DEC-090** every state that would under-approximate —
  index tag ≠ `H`, stage ≠ 0, non-UTF-8, probe error — is classified
  **unmeasurable** and `verify` refuses. The trailing clause is what makes this an
  invariant rather than a hope: F-22 proved a bound list cannot be completed, and
  a refusal does not need the list to be complete. Over-measuring yields a
  recoverable refusal; under-measuring yields an unrecoverable false attestation.
  Scoped to `verify` deliberately (RV-307 F-27). *The 18-state cube does not prove
  this invariant* — it is a content/existence projection showing the three legs
  jointly **necessary**, not jointly sufficient.
- **I10 — nothing lexically ineligible is ever emitted as a pathspec, declared or
  derived.** Empty or whitespace-only, control-char-bearing, absolute-outside, or
  root-escaping entries are dropped before git sees them. **Lexical, therefore
  total by construction rather than by enumeration** — the property RV-307 F-26,
  F-32 and F-37 each failed to achieve. This is what makes the `exit 128` abort
  *unreachable* rather than *handled*. **Under RV-314 F-7 this is a filter on
  every path entering the emission set, not a check on the declared entry alone:**
  step 4 manufactures candidates from index blob content — untrusted data — and
  the inherited guard ran before the only step that creates them, so the totality
  claim was false as written. I10 now carries a second load: declared entries
  reach measuring probes, not merely reporting ones.
- **I11 — the two historical-seam gates move together.** `retrieve::git_facts`
  and `retrieve::staleness` branch 1 gate on the same predicate and must continue
  to. Widening one alone is a silent no-op.

#### Edge cases

- **E2** — masters and shipped never reach `verify`: `run_verify` resolves
  through `items_root` alone, so `verify` is items-only by construction.
- **E4** — `memory/` absent (every client project) → that exclusion root is
  simply not contributed.
- **E5** — `scope.commands` is not path-shaped and contributes no pathspec; a
  memory scoped only by command has just its item directory in the claim surface.
  Exempt by kind, never reported as a defect.
- **E6** — a memory with an empty scope has a claim surface of exactly its own
  item directory. Still meaningful: the body must be committed.
- **E7** — **every** non-contributing scope entry is reported on stderr at verify
  time and raised by `validate`, unless declared `unobservable`. Silent narrowing
  of the claim surface is a false attestation reached quietly.
- **E8** — a **gitignored** scope entry that is nonetheless tracked is *kept*:
  ignore rules do not bind tracked files, so a force-added path is real evidence.
- **E9** — the inside/outside split is a property of the **checkout**, not the
  string. The 3 absolute-inside entries resolve inside the primary tree and
  outside a linked worktree, so a memory's claim surface narrows when verified
  from a dispatch worktree. Announced by E7 rather than silent.
- **E11 — an empty or whitespace-only entry is malformed: reported, not
  refused.** Never emitted (I10) — a bare `:(literal)`/`:(glob)` matches the
  **entire index**, which would invert the failure and make `verify` refuse on
  any unrelated dirt anywhere. Live population: **0**.
- ~~**E12**~~ — **withdrawn by DEC-020.** Struck id, never reused.
- **E13 — an entry whose emitted form would leave the repository is malformed:
  reported, not refused.** *Its basis has changed and the refusal is gone.*
  DEC-020 grounded the only surviving refusal in **mechanical necessity** — git
  aborts rather than returning a verdict — and called that "not a judgement about
  the memory, which is what makes this cut principled rather than merely
  smaller." DEC-053 removes the mechanism (I10). Keeping the refusal would
  therefore convert it into exactly the judgement DEC-020 forbids. The refusal is
  not optional to drop; it is compelled by DEC-020's own reasoning. Live
  population: **0**.
- **E14 — the contribution probe excludes the ADR-002 global/derived class.**
  `validate` runs over `collect_all`, which unions items and shipped. A global
  master is repo-empty and unanchored *by design*; its scope is not a claim about
  the querying repo's tree. **9 of 44** shipped scope entries are non-contributing
  in doctrine's own repo (`doc/entity-model.md`, `.doctrine/state/boot.md`,
  `.doctrine/skills/**`); in a client project this would be near-total and
  permanent. Emitting findings for them is the RV-307 F-25 error at corpus scale.
  Gated on `retrieve::is_global_reference` — record-local, so no provenance is
  needed and F-28 stays dissolved.
- **E15 — an entry traversing a symlinked *directory* is non-contributing, not
  resolved.** `linkdir/target.txt` and `:(glob)linkdir/**` match nothing under
  index-first, because step 4 only re-expands symlinks that are **themselves
  matched**. This is `candidate.sh` **FAL-4, which failed** and is recorded as a
  failure. An index-only ancestor walk recovers it (`residue.sh` (b), measured)
  and is deliberately **not built**. Live population: **0** (`census.py` COUNT 3).
  Carried as R-H. *Note this is a capability the inherited design claimed and this
  one does not* — see § 8.
- **E16 — a control-character-bearing entry is malformed: reported, not
  refused**, and never emitted (I10). A NUL cannot cross the argv boundary at all,
  so no git process is created and there is no exit code to classify — it sits
  outside E11/E13's original taxonomy, which is RV-307 F-38's first obligation.
  Live population: **0**.

#### Report framing (RV-307 F-38's second obligation)

Every scope entry text passes through `memory::scrub_line` before entering any
finding or stderr line. It already escapes `\n`, `\r`, `\t` and every control
char below `0x20`, and was built for this hazard class — its doc comment notes
that a scope value carrying a newline "would otherwise inject a forged metadata
line into the 'data, not instruction' block". **Riding the seam, not building a
second one.** Measured: a newline reaches git and returns an ordinary exit 1, so
it is a *reporting* hazard, distinct from NUL's *argv* hazard (`control-chars.py`
FAL-N4) — which is why F-38 insisted the two obligations not be conflated.

### 5.6 SPEC-007 reconciliation — the re-taken REV-034 inventory

**REV-034's inventory was drawn for `verify` before objective 7 existed and is
re-taken here, not deferred to close.** Objective 7 gives `validate` a probe, a
reporting contract and a continuation policy against a spec surface that barely
names it: `validate` appears in SPEC-007 as **one normative statement**, carried
in both tiers (`spec-007.toml:20` capability line and `spec-007.md:120`
§ Git-anchored staleness) — the sentence REV-041 added.

| Site | Current text | Why it must change |
|---|---|---|
| `spec-007.toml:22` | "stamp the verification axis against a clean working tree, refusing a dirty one" | already in REV-034 |
| `spec-007.md:138-141` | "it refuses a dirty tree so no false attestation is recorded" | already in REV-034 |
| **REQ-147** | title **is** the retired contract verbatim | already in REV-034 |
| **REQ-146** | "…scoped+attested by commits touching **scoped paths** since verified_sha…" | **NEW.** Objective 4 limb (a) widens the seam to `paths ∪ globs`. Both tiers carry "scoped paths". |
| **REQ-155** | "Resolve every undecidable git-reachability case to an explicit **fresh/stale/unknown/unanchored/reference** state" | **NEW.** REV-041 split the five-state vocabulary out as the *render contract* binding `find`/`retrieve`. A findings surface discharges the same obligation "by emitting a finding, not by falling silent" — which is **not** in REQ-155's title vocabulary. |

REQ-146 and REQ-155 are the **queried-surface trap of RV-307 F-39** exactly:
their titles are active members of SPEC-007 asserting what the body now qualifies.

**Routing: settled here, not deferred** (DEC-076, discharging RV-314 F-3). Both
land as **added REV-034 change rows**, authored — the revision now carries four
`modify` rows (REQ-147 primary, SPEC-007, REQ-146, REQ-155) and its title widened
from `verify`'s contract to the turnover it actually is. An earlier draft deferred
this to `/reconcile`, which contradicted the scope's own instruction that the call
"is settled during design, but it is not deferrable to close", and left ADR-013's
dependency (`SL-232 needs REV-034`) not authorising two of the four changes it
must cover.

A second revision was rejected on **atomicity**, not tidiness. All four sites go
false at the same instant — when this slice's code lands — and ADR-013 makes
`revision apply` the forcing-function tying approval to the truth-write. Two
revisions means two applies for one landing, opening a window where SPEC-007
asserts a mix of retired and current contracts: the exact trap REQ-147's row
exists to close. ADR-013 also favours accumulation directly — a Revision is "born
as content-light pending intent", "accumulates staged deltas as it is worked", and
gives dependents "a crisp single anchor".

### 5.7 The convergence, stated as a principle

This design answers *"which instrument decides X?"* with *"none — record it at
the source"* four times. Stated once so it is a principle rather than four
coincidences:

| question | instruments refuted | answer |
|---|---|---|
| is this entry a path or a pattern? | character sniffing (`*`/`?`/`[`) | **the field it came from** (DEC-053) |
| is this entry expected to be unobservable? | filesystem existence (RV-307 F-25), `rev-list --all` (RV-307 F-31) | **declared on the record** (objective 3) |
| is this `verified_sha` a commit? | stamp width, `cat-file -e` | **record the kind** (IMP-325, not here) |
| is this entry emittable? | probe-and-see (`exit 128`) | **decided lexically** (I10) |

Every refuted instrument reads state that a shallow clone, a pruned repo, a
dispatch worktree, or a different object format legitimately disagrees about.

## 6. Open Questions & Unknowns

**Answered this round — struck, not left stale:**

- ~~**OQ-2**~~ — answered **`yes`** on measurement (QUE-175). 13 of 43
  scoped-and-attested memories are glob-only and ranked by calendar. IMP-317
  splits; limb (a) taken as objective 4.
- ~~**OQ-A**~~ — answered **`no`**. The declared boundary is an authored input;
  IMP-318 and QUE-173 are machine-written outputs of a verify run. Different
  writers, lifecycles and validation. Sequenced, not merged.
- ~~**OQ-6**~~ — answered: `scope.unobservable`, § 5.3.
- ~~**OQ-B**~~ — answered: the shared *now*-question expander plus B18's
  continuation policy, § 5.4.

**Still open:**

- **OQ-3 / QUE-173** — a body digest stamped at verify time would make
  invalidation git-independent and path-independent. Routed, not built here.
- **OQ-5 — should the *anchor* leg narrow to declared scopes too**, so a dirty
  file no memory claims against stops blocking? **The inherited framing treated
  this as a nicety; it is not.** § 4's own principle — *dirt the memory does not
  declare says nothing about the claim* — points toward answering it **yes**, so
  leaving it open is a live tension this design names rather than hides. Deferred
  because taking it **deletes I3**, an inherited invariant governing every memory
  at once, and that needs its own evidence. **Reopen trigger:** if the relaxation
  this slice ships proves insufficient in practice — agents still hitting
  refusals from tree dirt no memory claims — reopen with measurement, not
  argument.

## 7. Decisions, Rationale & Alternatives

- **D3 — extract one dirt primitive (`observe_dirt -> Dirt`); `capture_with`
  consumes it; `capture()` delegates with `&[]`.** *Revised three times.* The
  original was a separate `source_clean` probe, which confused *behaviour* with
  *code* — the invariant worth protecting is I1, which delegation gives by
  construction. The second parameterised `capture()` alone, still short: `verify`
  needs a narrow answer for the claim question, and building a whole `Frame` to
  answer it would take the index lock on precisely the path I2 protects. **The
  third specified `-> bool`, which RV-314 F-11 falsified**: `capture()`'s dirty
  branch needs `diff_bytes`, `untracked_fp` and `index_tree`, and a bool discards
  all three, so `capture_with` would have to recompute them — the parallel
  implementation this decision exists to prevent. The observation type with a
  deferred-fingerprint `is_dirty()` projection serves both callers (§ 5.1).
  *Alternative:* bake the exclusion into `capture()` unconditionally. *Rejected:*
  two of its three callers would be damaged.
- **D9 — the gate asks two questions: is the ANCHOR tree clean, and is the CLAIM
  committed?** *Forced by RV-307 F-1/F-6* — a single exclusion set cannot express
  "ignore corpus dirt except the part this memory is about", because git offers no
  re-inclusion after an exclude. **Reworded on substance** per RV-307 F-39 limb 1
  and POL-002: "the code" was both a category error and a host-project assumption.
- **D10 — non-contribution is reported and attested over; it is not classified.**
  *Settled by DEC-020 after four revisions.* Every derived instrument reads local
  repository state, so a fourth would fail as the first three did. **Narrowed
  here:** D10's surviving *lexical* refusal (the probe-aborting entry) is gone,
  because DEC-053 removed the mechanical necessity that justified it (E13).
  **Corrected here (RV-314 F-30, DEC-090):** the claim "`verify`'s only refusals
  are the two gate questions" was **under-stated**, and it contradicted § 5.4's own
  outcome table, which never stopped carrying a *probe errored ⇒ refuse* row. The
  table was right. `verify` has **three** refusals — the two gate questions and
  **unmeasurable**. This does not reopen DEC-020: unmeasurable is not a
  classification *within* non-contribution, it is a state disjoint from it, and it
  is reached from data git reports rather than from a derived instrument
  adjudicating a memory.
- ~~**D11**~~ — **falsified** (DEC-054). "The constructor serves `verify` alone"
  cannot survive objective 7. Superseded by D13 and D17.
- **D12 — the *contribution* surface is built from the index, never the
  filesystem** (DEC-053). See § 5.2a. *Alternative:* a fourth repair of the
  ordered algorithm. *Rejected:* three totality claims had already failed over the
  same domain; the generalisable move was to make the failing taxonomy
  non-load-bearing, not to enumerate harder. **Narrowed by D18** (DEC-069): as
  originally written this said *the claim surface*, full stop, and that scope is
  what RV-314 F-1/F-10 falsified. The rule is correct and stays — for the question
  it was built to answer.
- **D13 — `validate`'s two unknowns are one mechanism** (DEC-054, objective 7).
  *Alternative:* build ISS-257 and RV-307 F-36 separately. *Rejected:* it
  implements the same epistemic honesty twice — the same parallel-implementation
  objection that forced `dirty_under` to be extracted once rather than duplicated
  (D3), one level up.
- **D14 — the undeterminable state is flat** (DEC-055). *Alternative:* split by
  stamp width. *Rejected on a falsifier:* sha256 repos have 64-hex commit ids, so
  the rule fails totally there; it also needs RV-307 F-31's refuted `cat-file`
  instrument for the dangling rows, and would introduce doctrine's first sha-width
  assumption. Routed as IMP-325.
- **D15 — the declared boundary is `scope.unobservable`: a parallel assertion
  matched by exact string equality** (§ 5.3). Chosen because it is *falsifiable*
  (V2) and never subtracts from the claim surface (I8). Three shapes rejected on
  the record.
- **D16 — malformed entries are reported, never refused, and the malformed /
  non-contributing split is lexical.** This does **not** violate DEC-020, which
  forbids classifying *within* non-contribution using instruments that read local
  state. A lexical split reads no state. The split earns its keep through the
  *remedy*: E7's remedy is a `unobservable` declaration, which is the wrong answer
  for a broken entry (V5).
- **D18 — measurement and reporting are two surfaces, not one** (DEC-069,
  § 5.2). *Forced by RV-314 F-1/F-10.* I9 was polarised backwards — soundness
  where the hazard is completeness — so an index-only surface silently omitted
  real evidence. *Alternative:* union the expansion with `ls-tree HEAD`.
  *Rejected:* it still needs raw selectors for untracked additions and the index
  leg for staged state, and must resolve symlinks across a second tree — more
  machinery for less coverage. *Alternative:* narrow to declared selectors with
  expansion advisory-only. *Rejected:* loses I7, since git does not traverse
  symlinks in pathspecs and agents address memories **by key**. *Alternative:*
  union selectors with the **entire** index expansion. *Rejected on cost, not
  correctness:* equally complete, but it passes thousands of concrete pathspecs
  back to git — 7,670 for `.doctrine/**` alone — an argv-size hazard on every
  verify, for paths the raw selector already measures.
- **D19 — the evidence domain is tracked-or-non-ignored commit-eligible**
  (DEC-070). *Alternative:* count every filesystem object matching a declaration.
  *Refuted on measurement:* 39 memories against 15,319 ignored files corpus-wide.
  An attestation names a commit, so a file git will never commit cannot be
  evidence for it — a definition of the domain, not a concession within it.
- **D20 — claim measurement inherits `capture()`'s stable-checkout boundary**
  (DEC-071). *Alternative:* lock or snapshot for the duration. *Rejected:* it
  takes `.git/index.lock` on the clean path and destroys I2. *Alternative:*
  re-probe until two passes agree. *Rejected:* unbounded on an actively edited
  tree, and it converts a stated assumption into a latency cliff without closing
  the window. Named rather than assumed, the same treatment R-E and R-F get.
- **D17 — the contribution probe is shared; the historical seam is not.**
  Contribution is a *now* question both verbs ask identically; drift is the
  historical question RV-307 F-27 protects. The cut is history-vs-now, not
  verify-vs-validate. *Alternative:* give `validate` its own probe. *Rejected:*
  parallel implementation, and it needs no `dir`, so F-28's cost objection does
  not apply.
- **D21 — neutralisation belongs to the observation, not to the runner**
  (DEC-089, superseding DEC-087). *Forced by RV-314 F-21/F-23/F-24.* Putting
  `--attr-source` in `NORMATIVE_FLAGS` was simultaneously too wide — it reached
  `check_attr_merge_z` and inverted SL-212's fail-closed guard — and too narrow —
  it closed one of four attribute sources. One cause, one fix: move it to
  `observe_dirt`'s argv builder, which `capture_with` and `verify` both consume,
  so they agree by construction. *Alternative:* exempt `check_attr_merge_z` from
  the chokepoint. *Rejected:* a conditional chokepoint is the thing EX-1 denies,
  and it fixes neither the coverage hole nor the compile-time-const problem.
  *Alternative:* scope to a command class. *Rejected — this is DEC-087's own
  split-brain objection and it is correct;* the decision taken scopes to the
  **observation primitive**, which is a different unit. *Alternative:* swap the
  index leg to the worktree-inclusive form. *Reconsidered on the record and
  rejected again:* it reaches neither `info/attributes` nor the freshness family,
  and it moves the leg I1/T59 pin.
- **D22 — the claim question has three outcomes, and the third is a refusal**
  (DEC-090, superseding DEC-082's bound). *Forced by RV-314 F-22*, with F-7, F-8,
  F-30 and F-31 collapsing into it. A bound list was falsified one round after it
  was written; a refusal does not need the list to be complete. *Alternative:*
  keep the bound and pin it with expected-blind tests. *Rejected:* that is the
  status quo F-22 refuted, and the next mechanism will be unenumerated too.
  *Alternative:* honour the index flags, per DEC-082's own reasoning. *Rejected on
  the verb's promise* — consistent for a status verb, not for one whose entire
  output is I6. *Alternative:* widen argv to `OsString` so non-UTF-8 paths can be
  emitted. *Rejected on cost against a population of 0.*
- **D23 — carry the resolved identity; do not re-derive it** (DEC-091). *Forced by
  RV-314 F-26.* `run_verify` already holds the directory it stamps; the design
  reached past it, first to construct a base from `memory_uid` (F-18) and then to
  bind that construction with a `read_link` equality check that only exists on one
  of three reference routes (F-26). The rule deletes both. *Alternative:* extend
  the equality check to all three routes. *Rejected:* it keeps two derivations of
  one fact plus a rule that they agree, which is the enumeration answer to a
  totality problem — the move § 5.2a exists to refuse.

## 8. Risks & Mitigations

- **R6 — `verify` is harder to satisfy, not easier, for the freshly-recorded
  memory. DEEPENED BY DEC-069.** Unrelated corpus dirt stops blocking; your own
  uncommitted claim still does. *The second clause now bites harder than the
  inherited text implied:* the measurement surface includes the declared entries
  themselves, so an **untracked** file under a declared glob refuses, where the
  index-only surface ignored it. Correct — untracked evidence is not in the commit
  — but for the 29 `.doctrine`-scoped memories it means an uncommitted new ADR or
  skill file under a claimed glob blocks verification. Say this plainly wherever
  the relaxation is described; the slice both loosens and tightens, and only the
  loosening is intuitive. The pressure this creates toward `--allow-dirty` is not
  free: per R-G it feeds the permanent undeterminable-finding inflow.
- **R7 — partially closing.** Limb (a) fixes the glob gate and magic
  neutralisation in both historical consumers. Limb (b) — own-directory drift
  needing item-directory provenance through `collect_all` — stays routed as
  IMP-317, where RV-307 F-28's dataflow cost stands.
- **R8 — an attestation does not record what it covered. SURVIVES THIS SLICE.**
  OQ-A answered `no`, so IMP-318 is not built here and a stamp still cannot
  distinguish a full attestation from a partial one. **Objective 3 does not close
  this.** The declared boundary makes the shortfall *authored* rather than
  inferred, which answers RV-307 F-25 **in part only**. Say so plainly wherever
  objective 3 is described.
- **R-A — discharged in method, narrowed to R-E/R-F/R-H.** The enumerate-then-probe
  obligation was met (`shapes.sh`). D12 then makes the taxonomy non-load-bearing.
- **R-E — index bits suppress the measurement itself, and no pathspec approach
  closes it.** A tracked file marked `skip-worktree` (`S`) or `assume-unchanged`
  (`h`) reads `diff-index` exit 0 while modified on disk. This is `candidate.sh`
  **FAL-5, which failed**. Detectable via `git ls-files -v`, and it affects the
  **anchor** leg as well as the claim surface, so it is wider than this slice.
  Live population: **0 rows** (`populations.py`, whole index). Latent and
  pre-existing, not introduced — named because a slice whose purpose is closing
  false-attestation routes cannot leave a known one unstated.
  ***Promoted in round 2 (RV-314 F-17, DEC-082), then CLOSED in round 4 (RV-314
  F-22, DEC-090).*** The promotion was right about the diagnosis and wrong about
  the remedy. Right: it is a counterexample sitting *inside* I9′'s other two
  bounds — the path is tracked, so DEC-070 admits it, and the checkout is stable,
  so DEC-071 does not reach it — which made I9′ false as written rather than
  merely unbounded. Wrong: making it a **named bound** pinned by expected-blind
  tests froze the defect in place, and F-22 then found a third suppressor
  (`core.fsmonitor`) carrying *neither* bit, proving the bound was an enumeration.
  It is now **discharged, not carried**: `ls-files -v` — named in this risk's own
  text as the detector, and declined at the time as out of scope — is the
  mechanism, it costs no extra invocation (§ 5.2a step 3), and the states it finds
  are refusals under DEC-090. **R-E leaves the known-open list.** The residue is
  narrower and honest: this risk also named the **anchor** leg, which DEC-090 does
  not cover, and that half survives as **R-I**'s neighbour rather than as R-E.
  Re-measured across `assume-unchanged`, `skip-worktree`, sparse-checkout and
  unmerged: `h`, `S`, `S`, `M` respectively, all discriminable from `H`.
- **R-F — case-insensitive collision is unmeasured, not cleared.**
  `core.ignoreCase` alone did not flip pathspec matching on ext4 (`residue.sh`
  (e)). A genuinely case-insensitive filesystem could not be probed from this
  jail.
- **R-G — absorbing ISS-257 widens the blast radius to a corpus-wide seam.**
  `memory_health_findings` is consumed corpus-wide, so the behaviour-preservation
  gate applies and the tri-state must not convert a silent exemption into a noisy
  one. **Narrowed by DEC-055:** the 34 newly-visible rows drain — objective 1 is
  what makes clean re-verification possible — so this is a one-time backlog the
  slice creates the remedy for, not a standing degradation.
- **R-H — index-first does not resolve a symlinked directory in an entry's
  ancestry** (E15). *This is a capability the inherited design claimed and this one
  does not*, so it must not be presented as a pure gain. The recovery mechanism is
  measured and available (`residue.sh` (b)) and deliberately not built. Live
  population **0**; reopen if a symlink-rooted declaration ever appears.
- **R-I — the Rust TOML parser's handling of an escaped NUL is unmeasured.**
  `control-chars.py` measured Python's `tomllib`, which parses `\u0000` to a real
  NUL. The rule holds regardless because the MCP route is open, but the Rust-side
  parse is carried as unmeasured, not cleared — same treatment as R-F.
- **R-C — R4 runs unmitigated meanwhile.** SL-230 ships invalidation without this
  relaxation. DEC-027's accepted tradeoff, and the reason to sequence this next.

## 9. Quality Engineering & Validation

Model test: `memory_verify_allow_dirty_stamps_checkout_state_id`; fixture:
`GitScratch`.

**The inherited matrix is rebuilt, not edited** — T26, T27, T31, T34, T36, T39
pinned the ordered algorithm, the shape rule, the whole-component prefix, or
E11/E13's refusals, none of which exist. Retired ids are listed rather than
reused.

### Retained

| # | Test | Asserts |
|---|---|---|
| T7 | verify, unrelated `.doctrine/**` dirty, memory committed | succeeds, stamps **HEAD commit** |
| T8 | verify, memory dir untracked (`record` → `verify`) | **refuses**; message names cause and remedy |
| T9 | verify, anchor tree dirty | refuses; message names `--allow-dirty` |
| T10 | `--allow-dirty`, anchor tree dirty | unchanged, stamps `checkout_state_id` |
| T10b | `--allow-dirty`, **only the claim** dirty | stamps a real `checkout_state_id` from the unexcluded capture — I4 |
| T11 | `capture(root)` == `capture_with(root, &[])` | I1 — clean, dirty, unborn, non-repo |
| T14 | `memory/` absent | exclusion root not contributed; no error |
| T17 / T18 / T19 | staged-only / unstaged-binary / untracked corpus change | excluded; succeeds (one per probe leg) |
| T23 | verify on the clean-after-exclusion path while `.git/index.lock` is held | completes — I2 canary |
| T24 | after a successful verify | `git show "$verified_sha:<dir>/memory.md"` equals the on-disk body **byte-for-byte** — I6 |
| T24b | body **tracked but modified**, verify | **refuses** — where existence and equality disagree |
| T25 | memory scopes `.doctrine/adr/**`, an ADR under it modified | **refuses** — scoped corpus dirt is claim-relevant |
| T27b | `scope.commands` and no path scopes | **succeeds** — exempt by kind (E5) |
| T27c | once-tracked-but-moved and never-tracked entries | **both succeed**, each reported and raised — treated alike (DEC-020) |
| T28 | verify **by key**, tracked memory, `memory.md` modified | **refuses** — I7; must use the key form |
| T30 | `scope.paths` carries `:(exclude)<own uid dir>`, body modified | **refuses** — I8 |
| T32 | `validate` drift over a **retargeted tracked symlink** | counts the retarget — `validate` does **not** canonicalise (RV-307 F-27). Equality between the two verbs' surfaces is explicitly **not** asserted |
| T33 | memory scoping a **tracked symlink** whose target content changed | `verify` **refuses** — must probe the *claim* leg specifically |
| T35 | `validate` over non-contributing scopes | each raised once per entry, `scope.commands` excluded |
| T37 | one non-resolving entry under **three ref states** (never tracked; tracked on a live branch; that branch `git branch -D`'d) | **identical outcome all three** — the DEC-020 regression test; fails the moment any ref-derived discriminator returns |
| T38 | `scope.globs` wildcard **inside** a component (`foo*/bar`, tracked `foobar/bar`) | **observable, clean-or-refuse** — now by emission-as-declared rather than prefix splitting |

### Retired

**T26, T27, T31, T34, T36, T39** — each pinned a mechanism DEC-053 deleted. Ids
struck, never reused. Their surviving assertions are re-expressed below: T34's
"no bare magic prefix" becomes T42; T39's "must not abort" becomes T43; T36's
symlink-rooted glob is now E15/T45; T31's glob-only gate is **inverted** by
objective 4 into T46.

### New

| # | Test | Asserts |
|---|---|---|
| T40 | expander over the **three RV-307 F-37 routes** (`missing/../link`, sparse `skip-worktree` entry, literal filename containing `*`) | each **contributes and reads DIRTY**. The regression test for DEC-053; each route reproduced pre-fix in `probes/route[123].sh` |
| T41 | expander over a **symlink chain** (`chain → link → real/target.txt`) | surface contains all three; bounded and cycle-checked; a cycle terminates |
| T42 | empty / whitespace-only entry | **malformed finding, not a refusal**; and the discriminating half — the constructed surface contains **no bare** `:(literal)`/`:(glob)`, which would match the whole index (I10, E11) |
| T43 | outside-shaped entries (`../gone`, `/tmp/no-such`, `:(glob)/tmp/no-such-*/**`) | **malformed finding, not a refusal** (E13); and `verify` **does not abort** — git exits 128 on these, so an unguarded entry takes the process down |
| T44 | control-char and NUL-bearing entries | rejected at the **write verbs** (MCP route, since argv cannot carry a NUL); a hand-edited `\u0000` entry is a **malformed finding**; every reported entry is `scrub_line`-framed so one entry never spans two report lines (E16, RV-307 F-38) |
| T45 | entry traversing a **symlinked directory** (`linkdir/target.txt`, `:(glob)linkdir/**`) | **non-contributing and reported** — pins E15/R-H as a *known boundary* so it can be neither silently closed nor silently widened |
| T46 | `validate` staleness on a memory scoped **only by globs** | **flagged** — inverts retired T31. Objective 4 limb (a) |
| T47 | `retrieve::staleness` on the same glob-only attested memory | resolves in **commit mode**, not the time branch — I11. Fails if `git_facts` is widened without `staleness` |
| T48 | `validate` where `verified_sha` is a **non-ancestor commit**, a **dangling object**, and a **`checkout_state_id`** | **one finding each, all three the same flat undeterminable state** (DEC-055) — and the discriminating half: no finding claims *no drift* |
| T49 | `validate` behaviour-preservation | the 25 ancestor-resolvable rows emit **byte-identical** findings to today (R-G) |
| T50 | `validate` over a corpus containing an **ADR-002 global master** whose scopes match nothing | **no contribution finding for it** (E14); an items memory with the same scope **does** get one |
| T51 | `scope.unobservable` — entry declared and non-contributing / declared and matching / declared but absent from `paths ∪ globs` / declared over a *malformed* entry | silent / **V2 finding** / **V1 finding** / **V5: malformed finding still raised** |
| T52 | `unobservable` edit via `memory edit` | does **not** clear the verification axis (§ 5.3) — contrast with a `--path-scope` edit, which does |
| T53 | `validate` continuation: one memory whose entry errors the probe | that entry yields a finding and **the corpus run completes**, every later memory still checked (RV-307 F-29, B18) |
| T54 | `verify` where the probe errors | **refuses** — the verify/validate asymmetry (§ 5.4) |
| T55 | expander under `core.quotePath=true` with a non-ASCII entry (`ünï.txt`) | matches correctly — pins the `-z` requirement, which is not stylistic |

### New — the DEC-069 split (RV-314 F-1 / F-10 / F-11)

Tests whose **meaning** changes rather than their text: T8 becomes specifically an
*untracked-leg* test; T25 and T38 now prove that **raw declared selectors** measure
claim dirt; T30 must exercise injection independently against all three legs; T40
must split contribution success from measurement success, since one expander
result can no longer stand for both questions.

| # | Test | Asserts |
|---|---|---|
| T56 | the **HEAD × index × worktree cube** as a table test, all 18 states | the 16 dirty states each read dirty; **including `H=A, I=B, W=A`**, which fails if the index leg is dropped as redundant |
| T57 | each of F-1a (untracked file under a declared glob), F-1b (`git rm --cached` on a claimed path, then modified), F-10 (untracked uid directory) | **refuses**, and each asserts **which leg** caught it — so a later refactor cannot silently move the coverage |
| T58 | measurement surface built for a memory scoping `.doctrine/**` | contains the **selectors**, not the 7,670 concrete matches — pins D18's cost rejection so the expansion cannot creep back in |
| T59 | `capture(root)` vs `observe_dirt` + projection | I1 byte-identity across clean / tracked-dirty / staged-only / untracked-only; and `is_dirty()` **never fingerprints** the untracked set (the deferred-hash contract) |
| T60 | ignored-untracked file matching a declared entry (DEC-070) | **does not block** — pins the evidence domain, which is a normative property, not a flag choice. Discriminating half: the same path **force-added** (tracked) **does** block (E8) |
| T61 | derived symlink targets `/etc/hostname` and `../../outside` (RV-314 F-7) | dropped by I10's recursive guard; `verify` **does not abort** — git exits 128 on all three legs, so an unguarded derived target takes the process down |
| T62 | I8 injection `:(literal):(exclude)<uid dir>` | neutralised on **each of the three legs separately** — the tracked, index and untracked probes each measured independently. The inherited demonstration covered only `diff-index` |
| T63 | DEC-071's temporal boundary | stated as an explicit stable-checkout assumption with a deterministic seam; **not** an atomicity claim |

### New — the DEC-080 / DEC-081 / DEC-082 round (RV-314 F-2 / F-15 / F-16 / F-17)

T56's **meaning** narrows: it verifies the three legs are jointly *necessary* over
the content/existence projection, and must be labelled as such. It is not a
totality proof and § 9 must not let it read as one (DEC-082).

| # | Test | Asserts |
|---|---|---|
| T64 | a tracked path modified on disk under `assume-unchanged`, and again under `skip-worktree` | ~~**expected-blind** — all three legs read clean~~ **INVERTED by DEC-090 (RV-314 F-22).** Now: `verify` **refuses**, naming index-state suppression. The three legs still read clean — that half of the old assertion stands and is what makes the fourth signal load-bearing — but the *outcome* assertion flips from attest-blind to refuse. The inversion is recorded rather than smoothed: the old test pinned a defect in place |
| T65 | a declared entry that is a **tracked symlink** whose target is detached from the index (`git rm --cached`) and modified on disk (RV-314 F-15) | **refuses.** Discriminating half: with emission conditioned on the index re-expansion the surface reads clean on all three legs — the test fails if DEC-080's unconditional emission is dropped |
| T66 | the same shape where the target was **never tracked** but is present and non-ignored | **refuses** — the second F-15 route, inside DEC-070's domain |
| T67 | a tracked symlink whose **blob content** is `:(exclude)<uid dir>`, with the uid body modified (RV-314 F-16) | **refuses.** Discriminating half: emitted raw the uid directory is subtracted and all legs read clean. T62 covers the *declared* injection route only and does not discriminate this one |
| T68 | a derived target that matches nothing anywhere, emitted alongside a genuinely dirty selector | the run **completes** (no leg errors) **and** the real dirt is still reported — pins the inertness that makes unconditional emission safe |
| T69 | `memory edit --unobservable` (bare) on a memory declaring one entry (DEC-081) | the array is **cleared**. Discriminating half: `--path-scope` with no values is a no-op, so the test fails if the arm is copied from the siblings rather than given `num_args = 0..=1` |
| T70 | `unobservable` authored via the **MCP** `EditParams` and via the CLI | identical persisted TOML — the two surfaces cannot drift (the EX-5 discipline SL-230 PHASE-05 established for `body`) |

T52 extends to the write path rather than gaining a row: it now asserts the edit
**lands** and leaves `verification_state` / `verified_sha` standing, against a
`--glob` edit on the same fixture which clears them.

### New — the DEC-087 round (RV-314 F-18 / F-19 / F-20)

| # | Test | Asserts |
|---|---|---|
| T71 | the **three reference routes** — `mem_<uid>`, a unique uid **prefix**, and the **key** — against one memory whose body is modified (RV-314 F-18 → F-26, DEC-091) | each measures and stamps the **same** directory, and each **refuses**. Discriminating half: constructing the base from the parsed `memory_uid` reads clean on the `Uid` route when the TOML declares an independently valid but unequal uid. *Rewritten from the F-18 form, which asserted a refusal plus a corpus-integrity finding on uid mismatch — under DEC-091 the base is right regardless, so that assertion no longer describes the design* |
| T72 | `.gitattributes` `text eol=crlf`, worktree CRLF, HEAD LF (RV-314 F-19) | **refuses.** Discriminating half: without the neutralisation all three legs read clean and `git status` is empty |
| T73 | a `clean` filter mapping worktree content to a fixed blob, worktree holding **arbitrary** other content | **refuses** — the route that hides more than line endings. Discriminating half as T72; note the `--cached` index leg alone does *not* catch it |
| T74 | `capture()` byte-identity across the neutralisation change, in a fixture with **no** `.gitattributes` | **unchanged** — the behaviour-preservation gate for I1/T59. This is the demonstration the consequence demands, not an assumption |
| T75 | the empty-tree oid under **sha1 and sha256** fixtures | derived via `git hash-object -t tree /dev/null` and *not* equal across them — fails if the oid is hardcoded (STD-001) |
| T76 | the capability probe against a git lacking `--attr-source` (CON-002) | `verify` **refuses legibly**, naming the requirement; it does **not** fall back to un-neutralised probes. Discriminating half: a silent fallback would let T72's fixture pass |
| T77 | refusal-path diagnosis on the T72 fixture | the message names **attribute conversion**, not operator edits — objective 6's legibility applied to this cause |

### New — the DEC-089 / DEC-090 / DEC-091 round (RV-314 F-21…F-32)

| # | Test | Asserts |
|---|---|---|
| T78 | a `clean` filter declared in **`.git/info/attributes`**, never committed (RV-314 F-21) | **refuses**, naming the file. Discriminating half: the neutralisation flags do **not** move this fixture — measured `0/0/rc=0` under every combination — so a test asserting the flags close it would pass only by not looking |
| T79 | the same filter declared via **`core.attributesFile`** | **refuses.** Discriminating half: `--attr-source` alone leaves it converting; `-c core.attributesFile=/dev/null` is what moves the tracked leg |
| T80 | `custom_merge_driver_paths` against a committed `f merge=ours` (RV-314 F-23) | still **refuses** the tree — SL-212's fail-closed guard is byte-for-byte unaffected. Discriminating half: with the neutralisation applied globally the driver reads `unspecified` and the guard **allows**. This is a behaviour-preservation test for another slice's invariant and belongs to the gate |
| T81 | the empty-tree oid derivation (RV-314 F-24) | runs **inside** the target repo, through the ordinary chokepoint, writing **no object** (loose-object count unchanged, I2). Discriminating half: derived outside a repository it returns the sha1 oid against a sha256 repo, and `--attr-source` then exits 128 on every attribute-reading invocation |
| T82 | `core.fsmonitor` pointed at a hook returning a stable token, index primed, tracked file replaced (RV-314 F-22) | **refuses.** Discriminating half: without `-c core.fsmonitor=false` all three legs read clean, `git status` omits the path, and `ls-files -v` reports `H` — so neither DEC-082's flags nor DEC-090's tag catch it, and only the neutralisation does |
| T83 | a claim-surface entry at index **stage 1/2/3** — the state any conflicted merge produces (RV-314 F-30) | classified **unmeasurable** from the stage field and refused, **without** `cat-file blob :<path>` being reached. Discriminating half: reaching it yields `exit 128` and an unclassified probe error |
| T84 | a tracked symlink whose blob is a **non-UTF-8** byte string (RV-314 F-31) | classified **unmeasurable** and refused. Discriminating half: a lossy conversion emits a *different* pathspec than the one matched, which then matches nothing and is inert (T68) — a silent under-approximation |
| T85 | `verify` on a **sparse checkout** with a declared scope outside the cone (DEC-090) | **refuses** — `S`-tagged entries are unmeasurable. Pins the mainstream population, so the cost cannot be discovered by a user first |
| T86 | `memory record` below CON-002's floor (RV-314 F-25) | **refuses legibly**, naming the requirement — not a generic `CaptureError::Git`. Discriminating half: `retrieve::freeze` on the same repo **degrades to `None`** and does not refuse, per B18/B19 |
| T87 | the `memory_edit` **advertised MCP schema** (RV-314 F-27) | contains `unobservable`. Discriminating half: a build wiring only `EditParams` passes T70 and fails this — which is the whole point, since T70 exercises the deserialiser rather than the published contract |
| T88 | `--unobservable` (bare) against a memory whose array is **already empty** (RV-314 F-32) | a **no-op**: the TOML is byte-identical and `updated` is **not** stamped. Discriminating half: the sibling arms' unconditional `changed = true`, copied verbatim, rewrites and stamps |
| T90 | two **untracked** files with different content under a `clean` filter, fingerprinted by `capture()` (self-attack, pre-round-5) | their `checkout_state_id`s **differ**. Discriminating half: with plain `hash-object` both files hash to one oid and the two dirty states collide. Second half on `text eol=crlf`: the fingerprint is the **raw CRLF** oid, not the LF-normalised one |
| T91 | the `XDG_CONFIG_HOME` global attributes file with `core.attributesFile` **unset** (self-attack) | **refuses.** Discriminating half: `--attr-source` alone leaves `check-attr` reporting the filter — this route is closed only by `-c core.attributesFile=/dev/null`, and a test fixture that sets `core.attributesFile` explicitly would not exercise it |
| T89 | § 5.4's operational `claim_pathspecs` definition against § 5.2's rule (RV-314 F-29) | the two constructors agree — asserted as a **document** test over the design's own text is not available, so this is discharged by the strike itself plus a code-level assertion that emission carries **no** coverage predicate. Recorded as a **VA** obligation, not a VT: the failure mode was prose divergence, and a test cannot see prose |

T64 is **inverted** rather than added to — see above.

**Closure:** every test in § 9 green (stated as a **set**, so a test added by a
later review cannot fall outside the gate by omission — RV-307 F-9);
`doctrine check gate` clean; **REV-034 applied** per the § 5.6 inventory so
SPEC-007, REQ-146, REQ-147, REQ-155 and the implementation agree.

## 10. Review record

**RV-314** is this document's ledger (facet `design`, raiser `inquisitor`) — four
external adversarial rounds plus a local pass, **32 findings**. RV-307 stays
attached to SL-230 (append-only; it reviewed that document).

> **The ledger is the only authority on finding state, and this section is a
> copy.** Round 4 found that this table, the slice's Harvest notes and a handover
> packet all recorded F-1 and F-10 as *answered* while both stood **`open` with no
> disposition** on RV-314 — the remedy prose had landed, `review dispose` was never
> run, and each artefact agreed because each had copied the last. `review show`
> does not print the finding roll, so the divergence was invisible through the
> sanctioned read path. Read state with
> `command grep -n '^id = \|^status = \|^disposition = ' .doctrine/review/314/review-314.toml`,
> and treat any disagreement with the table below as the table being wrong.

### RV-314, by current state

| Finding | Sev | State |
|---|---|---|
| F-1 · index-detached evidence never probed | blocker | **answered** — DEC-069 § 5.2 for declared entries, **DEC-080** for derived ones (the round-2 gap); I9′, T56/T57/T65/T66. *Disposed in round 4; the remedy landed in round 1 and the disposition was never recorded* |
| F-10 · untracked uid dir invisible to both legs | blocker | **answered** — same repair; T57. Base is the **resolved** directory (DEC-091), which exists before the key symlink is tracked. *Disposed in round 4, same lapse as F-1* |
| F-11 · `dirty_under -> bool` cannot serve `capture_with` | major | **answered** — § 5.1 `Dirt`, D3 revised, T59 |
| F-3 · Revision routing deferred against the scope | blocker | **closed** — DEC-076, four rows on REV-034, § 5.6 |
| F-2 · `scope.unobservable` has no producer | blocker | **answered** — DEC-081, § 5.3; `edit`-only, replace, `num_args = 0..=1` clear; T69/T70, T52 extended |
| F-15 · measurement's symlink closure still index-conditioned | blocker | **answered** — DEC-080, § 5.2; T65/T66. *Raised round 2* |
| F-16 · derived pathspecs unprefixed ⇒ uid dir subtractable | blocker | **answered** — DEC-080, I8 restated, § 5.2a; T67. *Raised round 2* |
| F-17 · 18-state cube cited as a totality proof | major | **answered** — DEC-082, I9′ third bound, R-E promoted; T64. *Raised round 2* |
| F-18 · uid parsed but never bound to storage identity | blocker | **answered** — but its round-3 repair was Key-route-only; **superseded by DEC-091** via F-26. *Raised round 3, self-inflicted by round 3* |
| F-19 · `.gitattributes` conversion defeats all three legs | blocker | **answered** — diagnosis stands; its round-3 repair (DEC-087) was refuted and **replaced by DEC-089**; I6 kept; CON-002; T72–T77. *Raised round 3* |
| F-20 · undefined "not already covered" predicate | major | **answered** — struck; emission is unconditional. The strike was **incomplete** and finished in round 4 via F-29. *Raised round 3* |
| F-21 · `--attr-source` closes only the tree source | blocker | **answered** — DEC-089: the partition, `-c core.attributesFile=/dev/null`, `GIT_ATTR_NOSYSTEM`, and refusal on `info/attributes`; T78/T79. *Raised round 4* |
| F-22 · `core.fsmonitor` blinds all three legs | blocker | **answered** — DEC-089 neutralises the config half, **DEC-090** refuses the index-state half; T82. *Raised round 4. Its stat-cache limb did **not** reproduce and is excluded from the charge* |
| F-23 · the global flag turns SL-212's guard fail-open | blocker | **answered** — DEC-089's placement; the guard is outside `observe_dirt` by construction; T80. *Raised round 4* |
| F-24 · per-repo oid cannot join a compile-time const | blocker | **answered** — DEC-089: a runtime argument, not a const member; bootstrap needs no attributes; T81. *Raised round 4* |
| F-25 · CON-002 doctrine-wide but discharged verb-locally | major | **answered** — § 5.1: the floor binds by **what the caller persists**; `record` refuses legibly, `retrieve` degrades per B18/B19; T86. *Raised round 4* |
| F-26 · F-18's repair is Key-route-only | major | **answered** — **DEC-091**: carry the resolved directory; the equality check is deleted, not extended; T71 rewritten. *Raised round 4* |
| F-27 · MCP joint named the deserialiser, not the schema | major | **answered** — § 5.3: `tools()` is a distinct joint; T87. *Raised round 4* |
| F-28 · `checkout_state_id` algorithm changed, tag did not | major | **answered** — `CHECKOUT_NORMALIZER` → `forget.checkout.v2`; a `v1` value is legible as pre-neutralisation (IMP-325). *Raised round 4* |
| F-29 · F-20's strike survived in § 5.4's constructor | major | **answered** — the § 5.4 row is struck; swept for a third copy, none found (two remaining hits are the strike record and this table). *Raised round 4* |
| F-30 · unmerged entry hits an unclassified `cat-file` 128 | major | **answered** — **DEC-090**: unmeasurable, caught from the stage field before `cat-file`; T83. *Raised round 4, not reproduced by the responder* |
| F-31 · non-UTF-8 targets cannot enter the string surface | major | **answered** — **DEC-090**: the domain stays UTF-8 and the out-of-domain case **refuses**; F-8 settled, argv not widened; T84. *Raised round 4, not reproduced by the responder* |
| F-32 · T69's control is a parse error; the copied arm no-ops | minor | **answered** — § 5.3: the fourth arm compares before writing; T69's control replaced, T88 added. *Raised round 4* |
| F-4 · T49 demands byte-identity from rows T35 changes | major | verified — restate to the drift class; drop the live-corpus absolute |
| F-5 · R-G's "one-time backlog" | major | verified — restate as stock-and-flow |
| F-6 · I11 one-directional | major | verified — extract the shared predicate |
| F-7 · step 4 bypasses the lexical guard | major | verified — **prerequisite** to DEC-069; I10 amended, T61. **Its exhaustion classification is now chosen** (DEC-090, via F-30): a probe that errors is *unmeasurable* |
| F-8 · non-UTF-8 index pathnames | major | verified — **byte domain now named** (DEC-090, via F-31): the domain stays UTF-8 and the out-of-domain case refuses. Argv is not widened |
| F-9 · E8/E9/V3/V4 untested | minor | verified — tests or stated exemptions |
| F-12 · `memory_health_findings_native` prefix contract | minor | verified — inventory it; assert attribution |
| F-13 · R-E unpinned while R-H gets T45 | minor | verified — pin or state why not |
| F-14 · "81 `.doctrine` items" does not reproduce | minor | verified — **29 at HEAD `743e7fe61`**; re-measure, stamp, move into a probe |

**No blocker's remedy is unwritten.** Every finding above now has prose in this
document *and* a disposition on the ledger — the second half is stated because
round 4 found it was not true. What remains is *verification* by the raiser: only
**F-3** is verified. F-2, F-15–F-20 are answered-but-unverified; F-1, F-10 and
F-21–F-32 are answered here for the first time and have not survived a pass.
F-7's classification and F-8's byte domain are now settled in *detail* as well as
in shape (DEC-090).

**Round-4 lesson: a negative search result is evidence only if the search could
have returned a positive.** Before opening round 5, this round attacked its own
three decisions. One claim — "the flags on `observe_dirt` cover every
attribute-sensitive read `capture()` makes" — was tagged *assumed*, probed, and
**falsified**: `untracked_fingerprint`'s `hash-object` collides two different
untracked files to one oid under a `clean` filter. The first sweep for it had
grepped `capture()` for `run_git|git_stdin|Command::new` and returned **empty**,
which reads exactly like a clean result. `capture()` calls the wrappers
`git_bytes`/`git_opt`/`git_text` (65 occurrences in the file), so that grep could
not have produced a positive on any input. This is the same family as the earlier
ugrep binary-skip lesson, and the general form is now stated: **before trusting a
negative, confirm the query can produce a positive.** Two further instances of the
class were found in the same sweep and routed out rather than absorbed —
**ISS-261** and **ISS-262**.

**Self-attack before external review is cheaper than either alone, and it does not
substitute for the other.** The round-3 lesson was that self-probing "narrows but
does not close"; it held again here — the self-attack found a real blocker-shaped
defect in new prose, *and* the same round's earlier self-review had confidently
written the false claim it caught.

**Three rounds have each refuted the round before.** Round 2 refuted round 1's
repair (F-15/F-16), round 3 refuted round 2's (F-18), round 4 refuted round 3's
(F-21/F-23/F-24 against DEC-087, F-26 against F-18's repair, F-29 against F-20's).
The base rate for this document is that a fresh repair does not survive its first
adversarial pass. **DEC-089, DEC-090 and DEC-091 are new and unattacked**, and
should be read as the most likely place a fifth round lands — not as the round
that finally closed it.

**Round-2 lesson, recorded because it generalises.** F-15 and F-16 are the *same*
error as F-1 and F-10 — the reporting instrument answering a measurement question
— surviving one level below where the repair was applied. DEC-069 moved declared
entries off the index and left derived ones behind, because the fix was written
where the finding pointed rather than where the *class* lived. A repair to a
reuse defect must be checked against every consumer of the reused instrument, not
only the one that failed.

**Round-3 lesson: check the axis, not just the property.** Three findings across
three rounds are one mistake — *verifying something true that is not the
load-bearing thing*. I9 asserted soundness where the hazard was completeness
(F-1/F-10). The prefix rule was proved for declared strings where the hazard was
derived ones (F-16). The uid was validated for its alphabet where the hazard was
its identity (F-18) — and that one was committed **in the same round that
recorded the previous two**, in prose claiming to close exactly that class. A
true property stated confidently is the most reliable way this design has found
to stop looking. F-19 is the same shape at the tool boundary: `git diff` reporting
clean is true, and it is not the property I6 needs.

**And the corollary, earned the hard way in round 3: an external reviewer's
*acquittals* need re-derivation more than its findings do.** F-19's clean-filter
limb was cleared by the reviewer after probing `diff-index --quiet HEAD`; § 5.1
specifies `--cached`, and against the specified leg the miss is total and hides
arbitrary content. The finding it *did* raise was real but materially
understated. Convictions arrive with a probe attached and get re-run; acquittals
arrive as prose and get believed.

### Inherited findings, by current state

| Finding | Was | Now |
|---|---|---|
| RV-307 F-36 | blocker — `validate` sink has no mechanism | **answered** — § 5.4, D17, E14 |
| RV-307 F-37 | blocker — non-resolution ≠ non-contribution | **answered at the root** — D12/DEC-053, § 5.2a, T40 |
| RV-307 F-38 | major — NUL/newline escape the taxonomy | **answered as two obligations** — E16, § 5.5 framing, T44 |
| RV-307 F-39 limb 1 | major — code-only wording at D9 | **swept** — D9 reworded on substance, § 5.4 |
| RV-307 F-25 | contested — partial attestation | **answered in part only.** R8 survives; objective 3 does not close it |
| RV-307 F-26 | contested — I9 totality, class collision | **answered at the root** — the taxonomy is non-load-bearing (D12); I9 restated as an outcome property |
| RV-307 F-32 | contested — prefix splitting, probe abort | **answered at the root** — no prefix is split; aborts prevented lexically (I10) |
| RV-313 F-2 (ISS-257) | issue | **absorbed** — objective 7, D13 |
| RV-313 F-6 | → REV-041 | **the normative anchor** for objective 7 |

**Verified, do not re-litigate without new evidence:** RV-307 F-1, F-2, F-6, F-7,
F-11, F-13, F-14, F-15, F-16, F-18, F-19, F-20, F-21, F-22, F-23, F-24, F-27,
F-28, F-29, F-30, F-31, and the governance pair F-4/F-5.

### Terrain that is settled and must not be reforked

- **DEC-020** — non-contribution is reported, never classified by a derived
  instrument. Three refuted (RV-307 F-21, F-25, F-31). A fourth *derived*
  instrument is not a finding; a *declared* boundary is the answer.
- **DEC-053** — index-first. No `realpath`, no character-based shape
  classification, no whole-component-prefix rule.
- **DEC-054** — ISS-257 and F-36 are one mechanism. The `commits_touching`
  ancestry guard is correct; the defect is at the call sites.
- **DEC-055** — the undeterminable state is flat. **Do not re-derive the
  stamp-width discriminator** — it is falsified on sha256 repos.
- **RV-307 F-27 survives DEC-053** — but the cut is *history vs now*, not *verify
  vs validate*. The contribution probe is shared; the drift seam is not (D17).
- **The weak reading of `verified_sha`** is the only reading (RV-307 F-33).
- **I8 / RV-307 F-18** — nothing a memory declares can subtract from what it is
  measured against.
- **DEC-027's split boundary** — SL-230 owns body-write and invalidation.

### Known-open on purpose

R8 (survives this slice), R-F, R-H, R-I, OQ-3/QUE-173, OQ-5, IMP-317 limb
(b), IMP-318, IMP-325, ISS-258.

**Left this list in round 4:** **R-E** — discharged by DEC-090 rather than
carried; its anchor-leg half survives beside R-I. **F-7**'s classification and
**F-8**'s byte domain — both settled by DEC-090 rather than deferred again.
