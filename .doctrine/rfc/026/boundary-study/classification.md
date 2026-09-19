# Boundary check: what design-review findings are about

Read-only research over `doctrine review` ledgers, 2026-09-19. Hypothesis tested:
most blocker/major design-review findings concern cheap-to-reverse things a thin
implementation would settle (B), not costly commitments (A).

Classes: **A** costly surface (persisted format/migration, public CLI/skill/boot
surface, binding governance, confinement/security, released artefact) · **B**
mechanism prediction (code-level detail a thin implementation would expose) ·
**C** artefact prose (internal inconsistency, stale restatement, review history,
citations, wording) · **D** missing human-owned intent · **E** other. **R** =
target text was written to repair an earlier finding (best-effort; see caveats).

## Selection

Group N is the fixed list. Group C was selected by: list design-facet ledgers
(`doctrine review list --all --json`, filter `facet=="design"`); for each,
`doctrine review status RV-NNN` for `findings`/`rounds`; keep derived status
`done` with `findings <= 10`; take the 6 highest ids. Result, most recent first:
**RV-365, RV-359, RV-358, RV-355, RV-347, RV-344**. RV-358 is also Group N's
SL-238 ledger; it is counted in both (see caveats).

## 1. Group N

| ledger | slice | design lines | rounds | findings b/m/mi/ni (n) | A/B/C/D/E | R |
|---|---|---|---|---|---|---|
| RV-325 | SL-233 | 781* | 0* | 5/11/1/0 (17) | 3/1/11/0/1 | 10 |
| RV-370 | SL-246 | 1,817 | 97 | 1/17/10/3 (31) | 2/15/1/0/0 | 1 |
| RV-307 | SL-230 | 823 | 161 | 15/15/9/0 (39) | 5/21/4/0/0 | 7 |
| RV-314 | SL-232 | 1,974 | 85 | 19/18/5/0 (42) | 11/21/5/0/0 | 7 |
| RV-349 | SL-249 | 1,866 | 138 | 1/15/19/0 (35) | 0/9/7/0/0 | 5 |
| RV-358 | SL-238 | 2,176 | 38 | 1/5/4/0 (10) | 2/4/0/0/0 | 1 |

## 1. Group C (comparison)

| ledger | slice | design lines | rounds | findings b/m/mi/ni (n) | A/B/C/D/E | R |
|---|---|---|---|---|---|---|
| RV-365 | SL-259 | 621 | 19 | 3/1/1/1 (6) | 0/4/0/0/0 | 0 |
| RV-359 | SL-256 | 970 | 1 | 0/0/0/0 (0) | 0/0/0/0/0 | 0 |
| RV-358 | SL-238 | 2,176 | 38 | 1/5/4/0 (10) | 2/4/0/0/0 | 1 |
| RV-355 | SL-254 | 1,556 | 22 | 3/2/2/0 (7) | 3/2/0/0/0 | 0 |
| RV-347 | SL-248 | 5,262 | 0* | 0/0/0/0 (0) | 0/0/0/0/0 | 0 |
| RV-344 | SL-244 | 3,459 | 30 | 1/4/5/0 (10) | 0/5/0/0/0 | 0 |

## 2. Flat table of classified findings

| ledger | F-N | severity | class | R | reason |
|---|---|---|---|---|---|
| RV-325 | F-1 | blocker | C |  | sketch double-dispositions a skill range; accounting inconsistent |
| RV-325 | F-2 | blocker | A |  | DEC-103 delivery rule amendment needs owner ratification |
| RV-325 | F-3 | blocker | C |  | 2a/2b discriminator rule cannot reproduce own assignments |
| RV-325 | F-4 | blocker | A |  | shipped skill rule removed before boot replacement lands |
| RV-325 | F-5 | major | E |  | edge-2 asset-shape fork left to implementation |
| RV-325 | F-6 | major | C | R | VA-4 amended criterion still omits Markdown payload |
| RV-325 | F-7 | major | C |  | no pre-registered decision rule for delivery trade |
| RV-325 | F-9 | major | C | R | withdrawn F-3 classifier still states D14 split |
| RV-325 | F-10 | major | C | R | F-7 repair exempts DEC-104 from its own falsifier |
| RV-325 | F-11 | blocker | A | R | DEC-101/PHASE-16 criteria still assign set mode |
| RV-325 | F-12 | major | C | R | F-4 repair reads premature attestation as misclassification proof |
| RV-325 | F-13 | major | C | R | F-12 repair adds a no-op 'stand' outcome |
| RV-325 | F-14 | major | B | R | F-13 control cannot observe incidental context loss |
| RV-325 | F-15 | major | C | R | F-13 repair's 'reword' leaves placement unchanged |
| RV-325 | F-16 | major | C | R | round-8 clarification equates failure-to-invent with no boundary |
| RV-325 | F-17 | major | C | R | round-10 gate not applied to three 2a assignments |
| RV-370 | F-1 | blocker | A |  | new design show surface violates governed CLI grammar |
| RV-370 | F-2 | major | B |  | dispatch-arm deletion has no acyclic route to handler |
| RV-370 | F-3 | major | B |  | unreadable-record guarantee unreachable for scan-pruned records |
| RV-370 | F-4 | major | B |  | full JSON shape incomplete and not single-source |
| RV-370 | F-5 | major | B |  | inspect --transitive --knowledge behaviour undefined |
| RV-370 | F-6 | major | A |  | DEC-260 omits prior carrier; R6 relies on duplicate |
| RV-370 | F-8 | major | B |  | read-only surface inherits worker-mode write refusal |
| RV-370 | F-9 | major | B |  | reclaimed design show flag set undefined at four points |
| RV-370 | F-10 | major | B |  | C1/I1 false of design show; skip golden unsatisfiable |
| RV-370 | F-11 | major | B |  | impact table omits tests bound to retired surfaces |
| RV-370 | F-12 | major | C |  | section 7.2 still states superseded five-key JSON entry |
| RV-370 | F-13 | major | B | R | full JSON entry lacks marker slot after F-4 repair |
| RV-370 | F-14 | major | B |  | in-block unreadable marker's only reachable case untested |
| RV-370 | F-18 | major | B |  | JSON arm specified as shape with no producer |
| RV-370 | F-19 | major | B |  | (withdrawn) marker logic sits on functions without inputs |
| RV-370 | F-20 | major | B |  | empty-state policy sits on functions without marker inputs |
| RV-370 | F-21 | major | B |  | design show flag partition incomplete and inconsistent |
| RV-370 | F-23 | major | B |  | by-design marker has no route past leaf renderers |
| RV-307 | F-1 | blocker | A |  | verified_sha contract omits the attested body |
| RV-307 | F-3 | blocker | B |  | body-first ordering mutates before validation rejects |
| RV-307 | F-4 | blocker | A |  | ADR-013 dependency names no active Revision anchor |
| RV-307 | F-5 | blocker | A |  | SPEC-007 revision inventory omits binding REQ-147 |
| RV-307 | F-6 | blocker | B |  | blanket root exclusions hide legitimately scoped evidence |
| RV-307 | F-8 | major | B |  | title/summary edits leave verification valid by design |
| RV-307 | F-12 | major | B |  | apply_edit stamps updated internally, breaking stamp step |
| RV-307 | F-13 | blocker | B |  | allow-dirty frame excludes claim-relevant corpus dirt |
| RV-307 | F-15 | blocker | B | R | key-form verify matches symlink, restoring F-1 blindness |
| RV-307 | F-16 | major | B | R | F-6 construction table states git mechanics backwards |
| RV-307 | F-18 | blocker | B | R | scope.paths pathspec magic subtracts claim surface |
| RV-307 | F-19 | major | B | R | validate keeps principal-path scope assumptions |
| RV-307 | F-20 | blocker | B |  | tracked symlink contributes while its target is invisible |
| RV-307 | F-21 | major | A |  | 55 current corpus items become unverifiable (migration) |
| RV-307 | F-22 | major | C |  | review history section restates normative rules |
| RV-307 | F-23 | major | B |  | empty scope value becomes whole-index pathspec |
| RV-307 | F-24 | major | B |  | retrieve remains a third raw claim-surface consumer |
| RV-307 | F-25 | blocker | B | R | D10 attestation covers only partial path evidence |
| RV-307 | F-26 | major | B |  | pre-pathspec canonicalisation not total |
| RV-307 | F-27 | blocker | B |  | canonicalisation erases symlink-retarget commits from history |
| RV-307 | F-28 | major | B |  | validate/retrieve discard item-directory provenance |
| RV-307 | F-29 | major | B |  | malformed-surface handling defined only for verify |
| RV-307 | F-31 | blocker | B | R | history discriminator ref-set-dependent, not checkout-stable |
| RV-307 | F-32 | major | B |  | ordered scope algorithm has reachable misclassifications |
| RV-307 | F-33 | blocker | C |  | strong and weak attestation contracts incompatible |
| RV-307 | F-34 | major | C |  | QUE-175 still gates IMP-317 on rejected shared-surface model |
| RV-307 | F-36 | blocker | A |  | validate does not implement DEC-020 corpus sink |
| RV-307 | F-37 | blocker | B |  | unresolved aliases contribute and read clean |
| RV-307 | F-38 | major | B |  | NUL/newline entries escape E11/E13 hostile-input contract |
| RV-307 | F-39 | major | C | R | round-7 stale-text sweep leaves code and title |
| RV-314 | F-1 | blocker | B |  | index-positive expansion drops untracked and HEAD-only deltas |
| RV-314 | F-2 | blocker | A |  | scope.unobservable schema has no producer, API or migration |
| RV-314 | F-3 | blocker | A |  | REQ-146/REQ-155 Revision routing deferred to reconcile |
| RV-314 | F-4 | major | C |  | T49 claims unaffected behaviour while nine rows change |
| RV-314 | F-5 | major | C |  | R-G risk text contradicts unchanged allow-dirty behaviour |
| RV-314 | F-6 | major | B |  | I11 lockstep enforcement is one-directional |
| RV-314 | F-7 | major | B |  | symlink re-expansion bypasses lexical guard; exhaustion unbounded |
| RV-314 | F-8 | major | B |  | -z pathname handling not byte-safe for non-UTF-8 |
| RV-314 | F-10 | blocker | B |  | dirty_under cannot see untracked mandatory uid directory |
| RV-314 | F-11 | major | B |  | dirty_under->bool cannot serve capture_with's three fingerprints |
| RV-314 | F-15 | blocker | B |  | symlink closure remains index-conditioned |
| RV-314 | F-16 | blocker | B | R | committed symlink blob pathspec magic subtracts uid dir |
| RV-314 | F-17 | major | B |  | 18-state enumeration blind to index-flag suppression |
| RV-314 | F-18 | blocker | B | R | memory_uid not bound to storage identity |
| RV-314 | F-19 | blocker | A |  | clean filter/attributes can hide arbitrary content from verify |
| RV-314 | F-20 | major | C |  | undefined coverage predicate survives in surface definition |
| RV-314 | F-21 | blocker | A |  | info/attributes and attributesFile still hide arbitrary bytes |
| RV-314 | F-22 | blocker | A |  | core.fsmonitor blinds all three verify legs |
| RV-314 | F-23 | blocker | A |  | raw-byte flag turns fail-closed merge-driver refusal fail-open |
| RV-314 | F-24 | blocker | B |  | per-repo empty-tree oid cycle has no unflagged route |
| RV-314 | F-25 | major | A |  | CON-002 doctrine-wide floor binds only verify; retrieve degrades |
| RV-314 | F-26 | major | B | R | F-18 identity binding only covers Key route |
| RV-314 | F-27 | major | A |  | DEC-081 MCP tool contract omits field from advertised schema |
| RV-314 | F-28 | major | A |  | checkout_state_id tag not bumped after algorithm change |
| RV-314 | F-29 | major | C | R | F-20 coverage-filter predicate survives in section 5.4 |
| RV-314 | F-30 | major | B |  | unmerged symlink hits unclassified cat-file exit 128 |
| RV-314 | F-31 | major | B |  | non-UTF-8 index targets cannot enter DEC-080 string surface |
| RV-314 | F-33 | blocker | A |  | stat-cache config omits trustctime/checkStat, blinding legs |
| RV-314 | F-34 | blocker | B |  | unmeasurable detector rides claim constructor only |
| RV-314 | F-35 | blocker | B |  | allow-dirty stamps Commit frame over byte-divergent body |
| RV-314 | F-36 | blocker | B | R | single read_link hop accepts nested alias |
| RV-314 | F-37 | blocker | B |  | GIT_WORK_TREE redirects measuring legs from stamped bytes |
| RV-314 | F-38 | blocker | B |  | DEC-071 bound omits info/attributes load-bearing gap |
| RV-314 | F-39 | major | A |  | CON-002 NORMATIVE_FLAGS floor amendment claim false |
| RV-314 | F-40 | major | C |  | DEC-091 not propagated; deleted memory_uid construction remains |
| RV-314 | F-41 | major | B | R | round-4 probe artefacts defective: wrong exit code |
| RV-314 | F-42 | blocker | B | R | F-33 stat-cache repair leaves a worse route open |
| RV-349 | F-1 | blocker | B |  | recovery can apply old acceptance to changed payload |
| RV-349 | F-2 | major | B |  | settle splits one TOML transition into two writes |
| RV-349 | F-3 | major | C |  | P1/P2 do not pin facet ownership multiplicity |
| RV-349 | F-4 | major | B |  | I9 cannot detect wrong wire-key subject mapping |
| RV-349 | F-5 | major | B |  | governance canary passes a still-four-kind spec |
| RV-349 | F-6 | major | B |  | I1 oracle never exercises the new writer |
| RV-349 | F-7 | major | B | R | D9 misses capitalised four-kind framing (F-5 fix gap) |
| RV-349 | F-8 | major | C | R | scope card keeps understated site count |
| RV-349 | F-9 | major | B |  | D9 misclassifies independent use of four |
| RV-349 | F-11 | major | B |  | allowlist presence does not make exemptions expire |
| RV-349 | F-12 | major | C | R | scope statement recreates drift F-9 corrected |
| RV-349 | F-13 | major | C | R | section 13 requires lifecycle work the slice defers |
| RV-349 | F-14 | major | C |  | declared allowlist phrase has count zero |
| RV-349 | F-21 | major | B |  | fingerprint guarantee does not cover licensed claims |
| RV-349 | F-25 | major | C | R | section 14 restates cross-section claims it should point at |
| RV-349 | F-34 | major | C |  | section 14 mirrors the ledger in narrative form |
| RV-358 | F-1 | blocker | B |  | backlog delegation creates unaccounted command cycle |
| RV-358 | F-2 | major | B | R | new root module missing from code-impact set (F-1 repair) |
| RV-358 | F-3 | major | B |  | one probe contract cannot serve listing and inspect |
| RV-358 | F-4 | major | A |  | doctor walk does not establish STD-003 tolerance |
| RV-358 | F-5 | major | B |  | Unavailable reachable through backlog needs CLI |
| RV-358 | F-6 | major | A |  | class_of unlisted departure from DEC-233 |
| RV-365 | F-1 | blocker | B |  | only CheckpointActGroup repaired; sibling act store uncovered |
| RV-365 | F-2 | blocker | B |  | post-mint resolved-ID refusal predicate unreachable |
| RV-365 | F-3 | blocker | B |  | forgotten-roster floor wraps only change-log rows |
| RV-365 | F-4 | major | B |  | RawRow retains no cause for degraded-row disclosure |
| RV-355 | F-1 | blocker | B |  | chosen unbound funnel path has no Spawn writer |
| RV-355 | F-2 | major | B |  | macOS prefix never sets DOCTRINE_WORKER identity |
| RV-355 | F-3 | major | A |  | worker_commit belts replaced by a different prove gate |
| RV-355 | F-4 | blocker | A |  | SPEC-012 revision covers only fork stamp wording |
| RV-355 | F-5 | blocker | A |  | one-arm design deletes ADR-012 Claude synthesis case |
| RV-344 | F-1 | blocker | B |  | Waived edge binds to ReviewPass nothing mints |
| RV-344 | F-2 | major | B |  | AgentActKind payload breaks &'static rule table slot |
| RV-344 | F-3 | major | B |  | Activation::Pending reason falsified by wired acts |
| RV-344 | F-4 | major | B |  | Artefact coverage conjunct can never fail |
| RV-344 | F-5 | major | B |  | ninth promise lacks Cause variant, remedy and test |

## 3. Totals (severe findings, side by side)

| class | Group N (n=123) | Group C (n=20) |
|---|---|---|
| A costly surface | 23 (18.7%) | 5 (25.0%) |
| B mechanism prediction | 71 (57.7%) | 15 (75.0%) |
| C artefact prose | 28 (22.8%) | 0 (0.0%) |
| D missing intent | 0 (0.0%) | 0 (0.0%) |
| E other | 1 (0.8%) | 0 (0.0%) |

## 4. Caveats

1. **Group C rule.** "Concluded" was read as derived status `done`. Only 4 design ledgers carry the explicit `conclude` marker, so the literal marker reading yields 4, not 6. RV-358 is both Group N's SL-238 ledger and the 3rd-most-recent Group C ledger; it is counted in both. Dropping it leaves Group C at 5 ledgers / 14 severe findings (A 3 = 21.4%, B 11 = 78.6%).
2. **RV-307 is design-facet** (target `SL-230`), so the prompt's caveat is resolved. RV-314 is the `SL-232` ledger (SL-232 split from SL-230 at RV-307 round 8). RV-307 and RV-314 are `active`; their open findings are included.
3. **RV-325 reports `rounds 0`** though it has 17 terminal findings (its baton records no transitions); RV-347 also reports 0. Treat both as CLI artefacts, not real round counts.
4. **RV-325 reviews `SL-233@PHASE-08`**; the reviewed artefact is `sketches/thin-adapter.md` (1,719 lines), not `design.md` (781). The design-line column reports the slice `design.md`.
5. **A/B hesitations** (class listed is the one assigned): RV-325 F-2/F-4/F-11 (A vs C); RV-370 F-1/F-6 (A vs B); RV-307 F-1/F-36 (A vs B), F-38 (B vs A); RV-314 F-20 (C vs A), F-25/F-28 (A vs B); RV-358 F-4/F-6 (A vs B); RV-355 F-3 (A vs B). RV-314's security-surface calls (F-19/21/22/23/33) were kept A because the verify gate can be *defeated*, not merely mis-stated.
6. **R is best-effort.** The review TOML carries no per-finding round, so repair-of-repair provenance was inferred from each finding's own text and disposition; R likely undercounts RV-370 and RV-314.
7. **Contradicts the hypothesis.** Group N is not dominated by costly-surface A (18.7%) and carries a large prose/consistency component (C 22.8%); Group C's A share (25.0%) is *higher* than Group N's. Non-convergent ledgers are not distinguished by carrying more costly commitments.
8. **Group C is thin.** Two of its six ledgers (RV-359, RV-347) have zero findings — trivial convergence — so its percentages rest on 20 findings.
9. **Severity totals:** Group N 123 severe of 180 total findings; Group C 20 severe of 23 total (RV-358 double-counted by the overlap in caveat 1).
10. **No ledger was unreadable.** RV-195's `review status` errors (a tracked path is a directory) but it is outside both groups.

## 5. Delegation test (does the design point at records, or restate them?)

Per the added line: for each reviewed slice, does `design.md` delegate its
normative statements to knowledge records (DEC/ASM/EVD/REQ) rather than restate
them? Graded from the current `design.md` (not the reviewed revision — designs
changed mid-review; see the section caveat).

| ledger | slice | delegates? | one-line evidence | C share (severe) | R |
|---|---|---|---|---|---|
| RV-325 | SL-233 | partial | no delegation posture; state machine is operative in `design.md`, DEC-103/104 carry only the classification rules | 11/16 = 68.8% | 10 |
| RV-370 | SL-246 | **yes** | `design.md:1358` "The records are normative; these lines are a map, not a restatement" | 1/18 = 5.6% | 1 |
| RV-307 | SL-230 | partial | `design.md:627` "This section points; it does not restate" — scoped to §10 review history only; mechanisms stay in the design | 4/30 = 13.3% | 7 |
| RV-314 | SL-232 | partial | records cited throughout (§0, §5.6) but no canonical-records statement; the design carries the normative mechanism | 5/37 = 13.5% | 7 |
| RV-349 | SL-249 | partial | §3 "## Binding" names records as binding, then restates them and carries the mechanism (`design.md:107-160`) | 7/16 = 43.8% | 5 |
| RV-358 | SL-238 | partial | one authority record named (`design.md:2017` DEC-236); no delegation posture otherwise | 0/6 = 0.0% | 1 |
| RV-365 | SL-259 | no | restates rather than delegates: `design.md:422` "DEC-250 is restated at the altitude" | 0/4 = 0.0% | 0 |
| RV-359 | SL-256 | no | no delegation statement; `design.md` is the sole normative surface | 0/0 | 0 |
| RV-355 | SL-254 | partial | cites ADR/DEC but no delegation posture; design is operative | 0/5 = 0.0% | 0 |
| RV-347 | SL-248 | **yes** | `design.md:5` "The records hold their own content; this cites and judges, and restates nothing" | 0/0 | 0 |
| RV-344 | SL-244 | **yes** | `design.md:5-6` "The records are canonical ... deliberately restates nothing" | 0/5 = 0.0% | 0 |

**Pooled C share by delegation level** (unique ledgers; severe findings).

| delegation | ledgers | severe | C (all) | C share | C1 copy-smear only | C1 share | R | R share |
|---|---|---|---|---|---|---|---|---|
| yes | RV-370, RV-347, RV-344 | 23 | 1 | 4.3% | 1 | 4.3% | 1 | 4.3% |
| partial | RV-325, RV-307, RV-314, RV-349, RV-358, RV-355 | 110 | 27 | 24.5% | 13 | 11.8% | 30 | 27.3% |
| partial, RV-325 excluded | RV-307, RV-314, RV-349, RV-358, RV-355 | 94 | 16 | 17.0% | 12 | 12.8% | 20 | 21.3% |
| no | RV-365, RV-359 | 4 | 0 | 0.0% | 0 | 0.0% | 0 | 0.0% |

`C1 copy-smear` is the subset of C that is a stale restatement of a fact owned
elsewhere (the four-cost table's first row): RV-325 F-9; RV-370 F-12; RV-307
F-22/F-34/F-39; RV-314 F-4/F-20/F-29/F-40; RV-349 F-8/F-12/F-13/F-25/F-34. The
rest of C is rule-logic/accounting defects (mostly RV-325), review sediment, and
internal contradiction, which typing records does not address.

**Verdict.** Directionally supportive, not decisive. Designs that explicitly
declare records canonical (`SL-246`, `SL-248`, `SL-244`) show a C share an order
of magnitude below partial designs (4.3% vs 24.5%, or 17.0% excluding the
phase-sketch outlier RV-325). Copy-smear-only narrows it to 4.3% vs 11.8-12.8%,
and the R signal (findings against repair text) separates harder still: 4.3% vs
27.3% (21.3% excluding RV-325). That R gap is the more direct read on your
suspicion, because every repair-of-a-repair is bookkeeping cost paid twice. But
`no` designs (SL-259, SL-256) also show 0% C/R, so the relationship is not
monotone; the honest reading is that both costs concentrate in **partially**
delegated designs — exactly where two surfaces coexist — which is what SL-246's
"records are normative where they differ" sentence betrays. The corpus cannot
separate that from review focus: the yes-designs were reviewed by different
passes with different severities, and RV-347/RV-359 are zero-finding.

**Caveat on this section.** Delegation is graded at the current `design.md`,
not the revision RV reviewed; several designs were rewritten during review, so
the posture at review time may differ. `yes` required an explicit
canonical-records statement, not merely DEC citations, which undercounts
designs that delegate silently.

## 6. B1/B2 split (class-B rows only)

B1 = a compile or thin happy-path implementation would expose it; B2 = only a
hostile or edge-case input would expose it (compiles and passes ordinary tests).
Re-judged from each finding's own text; borderline calls listed below.

| ledger | F-N | sev | B-split | reason |
|---|---|---|---|---|
| RV-325 | F-14 | major | B2 | control blind to incidental context loss (only on exhaustion) |
| RV-370 | F-2 | major | B1 | deleted dispatch arm leaves no acyclic route (structural) |
| RV-370 | F-3 | major | B2 | unreachable guarantee needs a scan-pruned record |
| RV-370 | F-4 | major | B2 | incomplete JSON shape shows only on a sparse record |
| RV-370 | F-5 | major | B1 | undefined flag combination hit on ordinary invocation |
| RV-370 | F-8 | major | B1 | read-only call refused (ordinary path fails) |
| RV-370 | F-9 | major | B1 | four flag behaviours undefined (ordinary use) |
| RV-370 | F-10 | major | B1 | unsatisfiable skip golden fails ordinary test run |
| RV-370 | F-11 | major | B1 | retired surfaces break existing tests on ordinary suite |
| RV-370 | F-13 | major | B2 | missing marker slot shows only on a marked record |
| RV-370 | F-14 | major | B2 | test-family gap needs the edge state to trigger |
| RV-370 | F-18 | major | B1 | JSON shape has no producer (nothing to call) |
| RV-370 | F-19 | major | B1 | functions cannot see inputs (type/scope error) |
| RV-370 | F-20 | major | B1 | functions cannot see marker inputs (type/scope error) |
| RV-370 | F-21 | major | B1 | inconsistent flag partition exposed on ordinary use |
| RV-370 | F-23 | major | B2 | marker unreachable at Full needs the marker state |
| RV-307 | F-3 | blocker | B2 | mutation-before-validation needs an invalid argument |
| RV-307 | F-6 | blocker | B2 | blanket exclusion hides evidence only for out-of-root scope |
| RV-307 | F-8 | major | B1 | title/summary edit ordinary; invalidation gap shows there |
| RV-307 | F-12 | major | B1 | body-only edit never stamps updated (ordinary edit) |
| RV-307 | F-13 | blocker | B2 | allow-dirty over a dirty corpus is the edge case |
| RV-307 | F-15 | blocker | B2 | key-form verify blind to symlink target (subtle route) |
| RV-307 | F-16 | major | B1 | backwards git mechanics fail ordinary pathspec use |
| RV-307 | F-18 | blocker | B2 | pathspec injection needs a hostile scope value |
| RV-307 | F-19 | major | B1 | validate keeps wrong scope for multi-path memories |
| RV-307 | F-20 | blocker | B2 | tracked-symlink blindness needs a symlink scope |
| RV-307 | F-23 | major | B2 | empty scope value is the edge input |
| RV-307 | F-24 | major | B2 | third raw consumer exposed by a retrieve-scope test |
| RV-307 | F-25 | blocker | B1 | partial attestation on ordinary declared evidence |
| RV-307 | F-26 | major | B2 | two uncanonicalisable classes are edge inputs |
| RV-307 | F-27 | blocker | B2 | erased retarget commits need symlink history |
| RV-307 | F-28 | major | B1 | discarded provenance shows on ordinary consumers |
| RV-307 | F-29 | major | B2 | malformed-surface handling needs malformed input |
| RV-307 | F-31 | blocker | B2 | ref-set dependence needs a changed ref set |
| RV-307 | F-32 | major | B2 | pattern inputs are the edge class |
| RV-307 | F-37 | blocker | B2 | unresolved aliases are the edge input |
| RV-307 | F-38 | major | B2 | NUL/newline entries are hostile input |
| RV-314 | F-1 | blocker | B1 | untracked/HEAD-only deltas are ordinary dirty-tree cases |
| RV-314 | F-6 | major | B2 | one divergence direction only; needs the untested direction |
| RV-314 | F-7 | major | B2 | lexical-guard bypass needs a symlink |
| RV-314 | F-8 | major | B2 | non-UTF-8 pathnames are hostile/edge input |
| RV-314 | F-10 | blocker | B1 | fresh record is ordinary; untracked uid dir invisible |
| RV-314 | F-11 | major | B1 | bool return cannot serve three-fingerprint caller (type) |
| RV-314 | F-15 | blocker | B2 | index-conditioned closure needs a symlink |
| RV-314 | F-16 | blocker | B2 | pathspec-magic symlink blob is the edge input |
| RV-314 | F-17 | major | B2 | index-flag suppression is the edge state |
| RV-314 | F-18 | blocker | B1 | uid base never bound to storage identity (data flow) |
| RV-314 | F-24 | blocker | B1 | bootstrap cycle blocks the ordinary build route |
| RV-314 | F-26 | major | B2 | Uid/UidPrefix routes are the uncovered edge routes |
| RV-314 | F-30 | major | B2 | unmerged symlink is an edge index state |
| RV-314 | F-31 | major | B2 | non-UTF-8 targets are edge paths |
| RV-314 | F-34 | blocker | B1 | anchor route stays falsely clean on ordinary anchor verify |
| RV-314 | F-35 | blocker | B2 | allow-dirty escape hatch is the edge case |
| RV-314 | F-36 | blocker | B2 | nested alias needs a nested symlink |
| RV-314 | F-37 | blocker | B2 | GIT_WORK_TREE is a hostile environment input |
| RV-314 | F-38 | blocker | B2 | info/attributes is the edge configuration |
| RV-314 | F-41 | major | B1 | probe artefacts fail when the probes are run |
| RV-314 | F-42 | blocker | B2 | stat-cache config is the edge configuration |
| RV-349 | F-1 | blocker | B2 | old acceptance reused only on recovery (edge) |
| RV-349 | F-2 | major | B2 | split transition shows only on a crash between writes |
| RV-349 | F-4 | major | B2 | oracle blind only to a wrong mapping (edge) |
| RV-349 | F-5 | major | B2 | canary passes a bad spec (hostile spec) |
| RV-349 | F-6 | major | B2 | oracle never runs the new writer (coverage gap) |
| RV-349 | F-7 | major | B2 | capitalised framing is the edge input |
| RV-349 | F-9 | major | B2 | independent use of four is the edge input |
| RV-349 | F-11 | major | B2 | exemption expiry is the edge behaviour |
| RV-349 | F-21 | major | B2 | uncovered licensed claims are the edge case |
| RV-358 | F-1 | blocker | B1 | command cycle is structural (compile/architecture) |
| RV-358 | F-2 | major | B1 | missing root module breaks the ordinary build |
| RV-358 | F-3 | major | B1 | one probe contract cannot serve two callers (type) |
| RV-358 | F-5 | major | B1 | Unavailable reachable through ordinary CLI path |
| RV-365 | F-1 | blocker | B1 | second act store uncovered on ordinary replacement |
| RV-365 | F-2 | blocker | B1 | post-mint refusal unreachable on the ordinary path |
| RV-365 | F-3 | blocker | B1 | roster floor wraps ordinary change-log rows |
| RV-365 | F-4 | major | B2 | no cause retained only on a degraded row (edge) |
| RV-355 | F-1 | blocker | B1 | no Spawn writer on the chosen path (structural) |
| RV-355 | F-2 | major | B2 | macOS-only prefix never sets identity (edge platform) |
| RV-344 | F-1 | blocker | B1 | Waived edge binds to unminted ReviewPass (structural) |
| RV-344 | F-2 | major | B1 | payload breaks &'static slot (type error) |
| RV-344 | F-3 | major | B1 | Pending reason falsified by wired acts (logic) |
| RV-344 | F-4 | major | B2 | coverage conjunct can never fail (oracle gap) |
| RV-344 | F-5 | major | B1 | missing Cause variant fails to compile |

**Pooled:** B1 = 35 of 82 class-B findings (42.7%); B2 = 47 (57.3%).

Per ledger, B1 / B2:

- RV-325: 0 / 1
- RV-370: 10 / 5
- RV-307: 6 / 15
- RV-314: 7 / 14
- RV-349: 0 / 9
- RV-358: 4 / 0
- RV-365: 3 / 1
- RV-355: 1 / 1
- RV-344: 4 / 1
### 6.1 What the split does to the headline

Unique severe findings (n=137) re-cut:

| class | n | share |
|---|---|---|
| A costly surface | 26 | 19.0% |
| **B1** compile / thin happy-path | **35** | **25.5%** |
| **B2** hostile / edge only | **47** | **34.3%** |
| C artefact prose | 28 | 20.4% |
| E other | 1 | 0.7% |

By group (B findings only):

| group | B1 | B2 | B1 share |
|---|---|---|---|
| Group N (non-convergent) | 27 | 44 | 38.0% |
| Group C (comparison) | 12 | 3 | 80.0% |

**This is the strongest result in the analysis.** The hypothesis as first
stated ("most findings are cheap to reverse; a thin implementation would settle
them") is only about a quarter right: B1 is 25.5% of severe findings, not
~58%. And the split separates the groups sharply. Non-convergent ledgers'
mechanism findings are 62% B2 — they compile and pass ordinary tests, so a
prototype or thin implementation would not have caught them; the cost sits in
hostile/edge input (symlinks, pathspec magic, non-UTF-8, stat-cache, ref-set
change). Converged ledgers' mechanism findings are 80% B1 — structural, caught
by the compiler or the first happy path. Convergence tracks *catchability*, not
severity.

### 6.2 Borderline B1/B2 calls

`RV-370` F-4 (B2 vs B1: sparse-record dependence); `RV-370` F-5/F-9/F-21
(B1 vs B2: undefined flags exposed only when that flag is exercised);
`RV-307` F-8 (B1 vs B2: ordinary title edit, but arguably a design choice);
`RV-307` F-15 and `RV-307` F-24, `RV-314` F-34, `RV-314` F-41, `RV-349` F-6,
`RV-344` F-4 (B1 vs B2: reachable on ordinary use but only a *discriminating*
test or route reveals it). Class-A rows were not re-opened; a handful of the
A/B hesitations in caveat 5 could move B2→A, which would lower B2 slightly and
raise A.

