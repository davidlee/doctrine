# Review RV-318 — reconciliation of SL-231

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Surface reviewed (F-2 of the dispatch caveat).** SL-231 was driven by
`/dispatch`, so `review/231` and `phase/231-01..05` are immutable evidence refs.
This audit reviewed the **candidate interaction branch**
`candidate/231/review-001`, created at `152ec3c4f` by
`dispatch candidate create --slice 231 --role review_surface --payload
impl_bundle --base refs/heads/main`, in the linked worktree
`.doctrine/state/dispatch/candidate/cand-231-review-001`. The merge onto `main`
was conflict-free and `git diff --name-only main...HEAD` is 29 files — the
honest implementation surface. The four `fix-now` repairs raised here landed on
that branch at `f1686831e`.

**Lines of attack.**

1. *Does the delivered CLI/MCP surface match the locked design?* §3.1 enumerates
   what the record verb "also accepts"; §2.3 defines what field origin means;
   §3.3 bounds the MCP adapter. Read the code against the prose rather than
   against the phase sheets, which are the author's own account.
2. *Do the VT criteria cover their own `expects` text?* RV-317 F-4 established
   that this slice had already shipped two criteria that passed while asserting
   nothing. Assume the class was not exhausted — check the `expects` prose
   against the keyword list, not the keyword list against the code.
3. *Is the conformance algebra trustworthy?* Treat the report as a lead, never a
   verdict. Re-derive undeclared/undelivered by hand against the true bundle
   before attributing scope creep to anyone.
4. *Independently re-read VA-1.* It asserts five surfaces tell one capability-aware
   story and was authored and self-checked in one session by one agent. Open all
   five; the claim is only as good as its weakest surface.
5. *Adjudicate the two PHASE-04 UNCERTAIN items* left open by the worker — facet
   `schema_version` defaulting, and caller-supplied `*_origin` riding through as
   sent — against the code, resolving each to a finding or to nothing.
6. *Treat the handover packet as hypothesis.* Its own generalised caveat: "a
   handover recommendation is a hypothesis to re-derive against the code, not a
   finding to act on." Applied to the packet itself, this cost one false lead
   (see Synthesis) and confirmed two real ones.

**Invariants held to.** ADR-001 layering direction and the observation leaf's
purity; STD-001 single-source constants; the storage rule (authored vs runtime
vs derived); design §2.3 origin semantics; EN-2/STOP-3 retention of the
`case-notes.md` historical corpus; EX-4's enumerated exclusions.

## Synthesis

**The slice delivers what it set out to deliver, and it is good work.** A typed
observation core with a UUID-native envelope and five optional facets; a
no-clobber publication primitive extracted from the entity engine without
disturbing its suite; a UUID-sharded store with replay and collision semantics;
a six-verb CLI; a bounded friction-only MCP adapter whose confinement is
structural rather than validated (there is no path field to abuse); and a
dogfood activation that replaced a `cat >>`-into-a-shared-file instrument with a
queryable corpus. `doctrine check gate` is green on the candidate surface — exit
0, 103 test binaries, 0 failures, only the pre-existing corpus warnings
(RFC-035, POL-003, terminal-slice notices) — and `slice verify-vt 231` reports
17/17 PASS there. The four `fix-now` repairs did not disturb either.

**The one thing that should not close unreconciled is F-1.** Design §3.1 says
the record verb "also accepts" repeatable typed facet fields and a complete
request from stdin or a file. PHASE-03 EX-1 restates both as "typed facets,
structured input". Neither exists: `FrictionRecordArgs` carries summary, detail,
uid, no-enrich, path, and `run_record` passes a hard-coded `None` for explicit
facets — so §3.1's "explicit caller values win" is structurally unreachable from
the CLI, and the two adapters are not at parity. What makes this more than an
omission is *how it survived*: PHASE-03 VT-1's `expects` names "stdin/file
input" and "explicit-over-automatic enrichment", and none of its four keywords
touches either clause. The criterion reported PASS while two clauses of its own
text were undelivered. That is precisely the RV-317 F-4 defect class, recurring
in a criterion the RV-317 remediation did not re-examine — which suggests the
remediation swept the instances it was pointed at rather than the class. The
standing risk to carry forward is not "the CLI is missing two flags"; it is that
this slice's VT keyword lists have twice been narrower than their own `expects`
prose, and nothing mechanical compares the two.

**The two PHASE-04 UNCERTAIN items were both real, and both worth the worker
flagging.** The `*_origin` one (F-2) was the sharper: `merge_explicit_facets`
copied the caller's origin marker verbatim, so a caller could stamp `Automatic`
on a value it had just supplied and forge the only provenance discriminator the
corpus carries. Not a security boundary — the whole capture path is trusted —
but the corpus is RFC-011's measurement input, and any statistic partitioned on
origin was unsound while a caller could set it. The design already establishes
the principle next door ("caller-asserted source metadata cannot open that
gate", pinned by PHASE-02 VT-4) and simply did not extend it here. The
`schema_version` one (F-4) resolved to six write-side literals rather than a
deserialization default; latent until the first version bump, then silent.

Both were *repetition* defects in the same 160-line function, which had 30
hand-written four-line blocks and — notably — no unit test coverage at all, only
one e2e MCP case. Fixing the class rather than the instances made the code
smaller and removed the `#[expect(clippy::assigning_clones)]` the old shape
required. Recorded because it generalises: a function that repeats a rule 30
times will get the rule wrong somewhere, and the tell was visible before the bug
was.

**VA-1 was the finding the process nearly missed, and the reason is structural.**
Four of its five surfaces genuinely tell one story, and the prose is better than
it needed to be — `using-doctrine.md` names the three things local-only storage
forfeits rather than waving at a tradeoff; RFC-011 explains *why* the shared-file
append was retired rather than just retiring it; the manifest entry carries its
own do-not-broaden rationale. The fifth surface, the shipped worker definition,
changed by exactly one line of frontmatter. A confined worker received a
capability with no instruction to use it and no pointer to the routing table —
and the two documents that *do* carry the routing (project governance,
`using-doctrine.md`) are not what a worker subagent reads at spawn. Its own
definition is. So the capability shipped unreachable in practice while every
agent-verified check said the contract was coherent. VA-1's author checked the
five surfaces they had written; the gap was in the surface where "I added the
token" and "I documented the capability" feel like the same act. This is the
argument for a second reader on any VA that asserts agreement *across* surfaces,
and it is worth generalising beyond this slice.

**The conformance report was unusable in both directions, and that cost real
time.** It reported 46 undeclared and 1 undelivered; the truth against the
29-file bundle is 3 and 0. The over-count is ISS-268's spanned `refresh-base`
merge. The under-count is its mirror image, new here: `git diff A..B` excludes
A, and PHASE-01's row *starts at* the commit whose entire content is the
`layering.toml` pre-seed — so EX-5's whole deliverable read as undelivered. That
half is systematic, because the pre-seed is required (workers may not write
`.doctrine/`, and the layering gate refuses an unclassified module), so the
pattern reliably puts authored content in the commit that becomes the fork base.
Two further platform defects compounded it: `slice conformance` resolves the
boundaries registry from the primary worktree but phase status from cwd
(ISS-269), so running it where the audit skill sends you — the candidate
worktree — reports every phase incomplete; and `slice verify-vt` reports FAIL
rather than UNCHECKABLE for a `test_file` absent from the tree (ISS-271), so the
same audit read 16 halting failures from the parent tree and 17/17 PASS from the
candidate. **Every conformance number in this audit was re-derived by hand.**
Nothing in the mechanical signal was trusted, and nothing should be until
ISS-268/269 land.

**Tradeoffs consciously accepted.**

- Four findings were repaired during the audit rather than deferred (F-2, F-3,
  F-4, and the tests for both). They land as one commit on the candidate branch,
  outside any phase boundary row — which is itself an instance of the
  attribution weakness above, and is disclosed here rather than papered over.
  The alternative was shipping a known provenance forgery and a capability no
  worker is told about, which is worse.
- The three platform defects (ISS-269/270/271) are dispositioned follow-up, not
  fix-now. They are outside both surfaces `/reconcile` writes and outside this
  slice's subject; each has a typed home with reproduction and a candidate fix.
- `w/SL-231-p01` remains un-reaped: the landed-oracle cannot certify it and
  `--force` would be a knowing bypass. Left for a deliberate decision.

**What close should not treat as settled.** F-1's deliver-or-narrow fork is the
user's call and is stated as a fork in the brief, not pre-empted. And whichever
way it goes, PHASE-03 VT-1's keyword/`expects` mismatch must be corrected in the
same pass — that is a design/plan escalation, off reconcile's direct-edit
surface, so it needs to be routed deliberately rather than absorbed.

## Reconciliation Brief

Built from every non-aligned, non-tolerated finding that touches design or
governance. The three platform findings (F-6/F-7/F-8/F-9 → ISS-268/269/270/271)
are **not** here: they are owned follow-up work with typed homes, outside both
reconcile write surfaces. The three `fix-now` findings (F-2/F-3/F-4) are **not**
here either: already landed at `f1686831e`.

### Per-slice (direct edit)

- **F-5 — `slice-231.toml`: declare the `src/worktree/allowlist.rs` selector.**
  The load-bearing change is the **selector registry**, not prose:
  `doctrine slice selector add` for the single path
  `src/worktree/allowlist.rs`, intent `design-target`. `slice conformance` reads
  `slice-231.toml`; a `design.md` §7 edit alone leaves conformance red.
  Add the matching §7 touch-set row in the same pass as the human mirror.
  **Exactly one path** — do not widen. The other two undeclared cells
  (`slice-231.toml` itself, `plan.toml`) are the registry and the RV-317 plan
  amendment, neither an implementation touch.

- **F-1 — `design.md` §3.1:169-175: settle the two undelivered CLI bullets.**
  A fork for the user, not a mechanical edit. Either:
  - **deliver** — the two input surfaces (repeatable typed facet fields;
    complete request from stdin or a file) become a follow-on phase or slice,
    bringing the CLI to parity with the MCP adapter, which already accepts
    explicit facets. `design.md` is then correct as written and needs no edit; or
  - **narrow** — drop the two bullets from §3.1 with a recorded rationale, and
    state that explicit facets are an MCP-only capability in V1.

  Whichever is chosen, `design.md` and the shipped CLI must agree at the end.

### Governance/spec (REV)

None. No finding on this ledger reaches an ADR, policy, standard, or spec.
F-4 cites STD-001 but the violation was in the code and is fixed; the standard
itself is correct and unchanged.

### Escalation — NOT a reconcile surface

- **PHASE-03 VT-1 `expects` vs keywords.** The criterion's `expects` names
  "stdin/file input" and "explicit-over-automatic enrichment"; its four keywords
  cover neither. `plan.toml` `EN-/EX-/VT-` ids are immutable-append and
  `plan.toml` is **off** reconcile's direct-edit surface, so this cannot be a
  brief item. Route it deliberately: under **deliver**, append keywords (never
  renumber) so the criterion covers its own text; under **narrow**, correct the
  `expects` prose in the same amendment. Left unrouted, the plan keeps asserting
  a contract nothing checks — which is how F-1 survived five phases and a
  ledgered code review.
