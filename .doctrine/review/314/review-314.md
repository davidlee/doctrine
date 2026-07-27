# Review RV-314 — design of SL-232

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Arraigned:** `.doctrine/slice/232/design.md` — the design intent alone. Not the
scope, not a plan (none exists), not code.

**Provenance of the accused.** This document is a *rewrite*, not an increment. It
replaces the SL-230-era text wholesale after DEC-027 split the gate out at RV-307
round 8. It therefore inherits 29 findings' worth of settled terrain and claims
to answer two round-8 blockers (RV-307 F-36, F-37) and absorb one issue
(ISS-257 via RV-313 F-2). **A rewrite is where inherited guarantees go to die
unnoticed** — the presumption of guilt is strongest exactly here.

### Lines of interrogation

1. **The inheritance audit.** § 10 declares 21 findings "verified, do not
   re-litigate". A rewrite that deletes the mechanism a finding was verified
   *against* silently un-verifies it. Every claim of "answered at the root" is a
   charge to be tested, not a fact to be accepted.
2. **DEC-053's index-first constructor (§ 5.2).** Does the five-step rule actually
   discharge what it claims, and is step 4's symlink re-expansion bounded,
   cycle-safe, and honest about E15/R-H? Does *anything* survive that still reads
   the filesystem where it claims to read the index?
3. **The two-question gate (§ 5.4).** Are `corpus_excludes` and `claim_pathspecs`
   jointly total? Can a dirt state exist that neither question sees? Is I4's
   `--allow-dirty` re-capture actually free of an extra `capture()`?
4. **I8 / the injection surface.** Nothing declared may subtract from what is
   measured. Prove it holds across `unobservable`, the magic prefixes, and the
   `-z`/`quotePath` route.
5. **Objective 7's blast radius.** ISS-257's tri-state touches a corpus-wide
   seam. Does R-G's "one-time backlog" claim survive, and is the
   behaviour-preservation gate (T49) actually a gate or a wish?
6. **The lockstep I11.** `git_facts` and `retrieve::staleness` branch 1 must widen
   together. Is that an invariant with teeth or a comment?
7. **Governance conformance.** POL-002 (no host-layout assumption), STD-001 (no
   magic strings), ADR-001 (leaf ← engine ← command), ADR-002 (E14's exemption),
   ADR-013 (REV-034/REV-041 routing), SPEC-007's re-taken inventory (§ 5.6) —
   including whether REQ-146/REQ-155 belong in REV-034 or a second revision.
8. **The test matrix (§ 9).** Rebuilt, not edited. Does every new invariant,
   edge case and decision have a test, and does every test have a
   *discriminating* half — one that fails if the mechanism is absent?
9. **Overclaim and underclaim.** § 4 forbids stating value by what is fixed
   today. Every "total by construction" is a totality claim over a domain; find
   the domain it under-enumerates.
10. **Executability.** Is this design implementable without a further design
    round — signatures, altitudes, ownership, and the plan's phase seams?

### Method

External adversarial pass via the codex MCP reviewer (GPT-5.5), read-only on the
working tree, then adjudicated here. Findings are raised under the `inquisitor`
posture on the `design` facet; RV-307 remains attached to SL-230 and is not
appended to.

## Synthesis

### Judgement

**The document is not heretical. Its method is sound and its honesty is
conspicuous.** It carries its failed falsifiers (`candidate.sh` FAL-4 and FAL-5)
as failures, it strikes retired ids rather than reusing them, it re-measures its
own corpus figures against a stamped HEAD, and it states R-H as a capability the
inherited design had and this one does not. That is the conduct of a design that
wants to be caught. It was — thirteen times.

**But it must not proceed to implementation.** Four blockers stand, and three of
them share one root: *the design's two gate questions do not observe a joint
domain*. DEC-053 moved the claim surface onto the **index**; the anchor leg and
the attestation both live at **HEAD**; and `dirty_under` is specified only by a
signature. Between those three planes sit states that neither question sees.

### The one structural defect

**Index-membership, HEAD-membership and worktree-presence are three different
predicates, and the design treats them as one.** Reproduced on git 2.54.0:

- **F-10** — a freshly recorded uid directory is untracked. `git diff-index
  --quiet HEAD -- .doctrine/memory/items/mem_new` exits **0**. Only
  `ls-files --others` sees it. The anchor leg excludes `.doctrine`, so neither
  question fires and `verify` would stamp a HEAD that does not contain the
  attested prose — **RV-307 F-1's exact false stamp**, which § 5.4:379-384 and T8
  promise to refuse.
- **F-1** — a claimed path detached from the index (`git rm --cached`) while
  modified on disk vanishes from the expander (`ls-files -s` empty ⇒ step 5 calls
  it *non-contributing*), yet remains real evidence in HEAD. Under an excluded
  root, all three anchor probes and the claim leg read clean. **Verified:** HEAD
  content `a`, disk content `a\nTAMPERED CONTENT`, both gate questions pass. The
  inherited `realpath` rule caught this (`diff HEAD -- <path>` reports it,
  `diff-index --quiet` exits 1); index-first drops the entry before anything can
  probe it. This is a **regression the design does not name** — § 5.2's "what this
  retires" and R-H, the sole acknowledged capability loss, both omit it.
- **F-11** — and the primitive that would have to close them cannot be built as
  factored: `dirty_under -> bool` discards the three fingerprints
  `capture_with`'s CheckoutState branch needs (`src/git.rs:2230-2255`).

§ 5.2 asserts, on `shapes.sh`, that resolution and contribution are "uncorrelated
**in both directions**". It enumerates only the harmless direction — resolves but
does not contribute (E15/R-H, population 0). The harmful direction — **real,
committed, claimed evidence the index does not carry** — is the one that produces
a false attestation, and it is unenumerated. By § 4's own principle: *a tool
property is a claim needing a falsifier, not a premise.* `ls-files` was taken as a
premise.

### Ordered penance

1. **F-10, F-1, F-11 together — reopen § 5.2/§ 5.4's observation domain.** These
   are one repair, not three. State explicitly which probes `dirty_under`
   comprises (tracked diff, index-vs-HEAD, untracked), and decide whether the
   claim surface is index-derived, HEAD-derived, or the union. *Verification:* T8
   with a discriminating half (an untracked-only probe leg that fails if absent);
   a new test for the index-detached-and-modified route; T11 re-asserted against
   the widened primitive.
2. **F-3 — settle the REQ-146/REQ-155 Revision routing now.** The scope
   (`slice-232.md:232-239`) says this "is settled during design, but it is not
   deferrable"; § 5.6:607-618 defers it to `/reconcile`. The design contradicts
   its own charter and ADR-013. *Verification:* the revision rows exist before
   `/plan`.
3. **F-2 — give objective 3 a producer.** A *declared* boundary with no authoring
   verb is not declarable. Name the CLI flag, the MCP `EditParams` field
   (`src/mcp_server/tools.rs:1029-1068`), replace-vs-append semantics, and the
   absent-field default. *Verification:* T52 extended to the write path.
4. **F-7, F-8 — close the constructor's two open joints.** Step 4 manufactures
   absolute-outside inputs that I10 calls unreachable (proven: exit 128); and the
   index's legal domain includes non-UTF-8 pathnames the `(root, entry, magic)`
   signature cannot round-trip.
5. **F-4, F-5 — correct two claims that cannot hold.** T49 demands byte-identical
   output from nine rows this design deliberately changes; R-G calls a state
   recurring `--allow-dirty` keeps producing a "one-time backlog".
6. **F-6, F-9, F-12, F-13 — the reporting and coverage tail.** I11's inverse
   direction, the E8/E9/V3/V4 gap, the `"{uid}: "` attribution contract, and
   R-E's missing boundary test.

### Standing risks, consciously left

R8, R-E, R-F, R-I, OQ-3/QUE-173, OQ-5, IMP-317 limb (b), IMP-318, IMP-325 and
ISS-258 remain open **by the design's own declaration** and are not charged here.
OQ-5 in particular is named by the design as a live tension rather than hidden —
that is correct conduct and earns no penalty.

### What was not examined

R-F (case-insensitive filesystem collision) could not be probed — the jail
exposes ext4 only, and the design already carries it as unmeasured. The 21
findings § 10 lists as "verified, do not re-litigate" were audited for survival
under DEC-053, not re-tried on their merits.

### Disposition posture

**No charge is disposed.** Three of the four blockers admit more than one lawful
remedy (F-1: widen the domain vs narrow the claim and pin the boundary; F-2:
schema and verb shape; F-3: an added REV-034 row vs a second revision), and
`review-ledger.md` § 4 forbids improvising a sentence where the route is
ambiguous. Those are the author's calls, not the Inquisitor's. The ledger stands
**active with four unresolved blockers**, which correctly refuses SL-232's
advance until the penance is done.

> **HERESIS URITOR; DOCTRINA MANET**
