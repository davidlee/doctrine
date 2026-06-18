---
name: design
description: Use when a slice needs architectural shaping before implementation — decision triage, critical analysis of tradeoffs and solutions, and section-by-section validation of design.md until the decisions lock. Routed to from /route once a slice exists.
---

# Design

You are translating scoped intent into implementable design.

Inputs:
- existing slice folder
- existing `design.md` or equivalent design artifact
- relevant related artifacts, source material, research, etc.

## Workflow State Machine

Complete in order without deviation. Each depends on the preceding stage:

1. **Explore context** — specs, ADRs, memories, files, docs, recent commits. Begin high-level.
2. **Collect decisions.** Survey the design surface for every open decision
   — choices not yet made, tradeoffs not yet resolved, constraints not yet
   accepted. Where a design doc already exists (refinement, not greenfield),
   read it; otherwise work from the slice scope and the Explore output.
   Survey implications: what entity constraints, existing REQs, or ADR clauses
   bear on each question.

   Batch all decisions together. Present them as a structured list with
   options, tradeoffs, and your recommendation for each. Ask the user to
   confirm or correct each one.

   **Guardrail:** Batch ALL decisions before presenting. Do not badger the
   user with each decision — collect first, then present.

   This state is a decision-confirmation loop, not a replacement for the
   exploratory "Ask clarifying questions" loop (state 3). If surfacing a
   decision reveals an unknown the user must resolve, hand to state 3 rather
   than forcing a premature choice.
3. **Ask clarifying questions** — one at a time, understand purpose/guiding principles/constraints/success criteria
4. **Requirements pass.** Before proposing approaches, inventory the
   requirements picture:
   - Which existing canonical REQs (REQ-NNN) are impacted?
   - Which implied requirements does this design's decisions demand that no
     existing REQ captures? These are **orphan requirements** (REQ-DNN).
   - Ask the user: are there requirements they see that aren't captured?

   Record in `design-requirements.toml` (authoritative) and reference from
   `design.md` `## Implied Requirements` with one-line summaries.

   **Guardrail:** Do not record implied requirements as structured fields in
   prose — use `design-requirements.toml` (§2).

   **Guardrail:** If an implied requirement crosses the altitude boundary
   (product vs C4 level, or crosses spec kinds), note the ambiguity in the
   TOML `home_hint` field — don't force a decision here. `/reconcile` handles
   placement.
5. **Propose 2-3 approaches** — identify the next unanswered design question; propose options with trade-offs and your recommendation
6. **Present design** — in sections scaled to their complexity, get user approval after each section.
7. **Write design.md** — save to the slice `design.md` file and commit
8. **Adversarial Review** — perform a hostile review of the design doc, probing for imprecision and flawed reasoning
9. **Integrate Review Feedback** — triage and respond to feedback; integrate into slice / design doc; repeat until the design locks (explicit user approval)
10. **Transition to planning** — record the lifecycle move (`doctrine slice
   status <id> plan` — bare number), then invoke `/plan` to create the
   implementation plan

<Process State Machine>
  <state name="Explore context">
    <transition to="Collect decisions" />
  </state>
  <state name="Collect decisions">
    <transition to="Ask clarifying questions" />
  </state>
  <state name="Ask clarifying questions">
    <transition to="Requirements pass" />
  </state>
  <state name="Requirements pass">
    <transition to="Propose 2-3 approaches" />
  </state>
  <state name="Propose 2-3 approaches">
    <transition to="Present design" />
    <transition to="Ask clarifying questions" />
  </state>
  <state name="Present design">
    <transition to="Write design doc" />
  </state>
  <state name="Write design doc">
    <transition to="Adversarial review" />
  </state>
  <state name="Adversarial review">
    <transition to="Ask clarifying questions" />
    <transition to="Propose 2-3 approaches" />
    <transition to="Present design" />
    <transition to="Write design doc" />
    <transition to="Transition to planning" />
  </state>
  <state name="Transition to planning">
    <transition to="invoke /plan skill" />
  </state>
</Process State Machine>

## design-requirements.toml sidecar

The `design-requirements.toml` file records implied/derived requirements
discovered during design. It is the **authoritative** source; `design.md`
`## Implied Requirements` is a prose cross-reference with one-line summaries
only — never structured fields.

```toml
[[implied]]
handle = "REQ-D01"
statement = "..."
kind = "quality"
home_hint = "..."
descends_from = "..."
```

Each `[[implied]]` entry:
- **`handle`** — provisional id (`REQ-DNN`); assigned by the design skill,
  resolved later via `/reconcile`.
- **`statement`** — what the requirement demands.
- **`kind`** — `functional`, `quality`, `constraint`, or `interface`.
- **`home_hint`** — where the requirement likely lives (product-level, C4
  container, etc.). Leave empty when uncertain; `/reconcile` handles placement.
- **`descends_from`** — what drove this: a design decision, an ADR clause,
  or an existing REQ.

## Process (detail)

### Explore context

1. Read the slice scope + relevant specs, ADRs, prior art first.
2. Run `/canon` before drafting so relevant ADRs, policies, and standards are in view for this design surface.
3. Run `/retrieve-memory` against the files and subsystems you expect to touch, so scope-bound gotchas surface early.
4. Before drafting sections, explicitly generate a list of concerns and then triage the design surface:
   - open questions that must be resolved
   - risks and underspecified areas
   - assumptions you are carrying
   - critical design decisions that shape the rest of the design
   - relevant ADRs, policies, and standards that constrain the design

### Ask clarifying questions

Proceed in a light loop to clarify intent, surface known and unknown unknowns,
and drive towards sufficient clarity to lock a design.

Apply this process first to the slice scope itself if necessary, then to the
technical design.

At each step:

1. Summarize:
   - what's already understood
   - carrying assumptions
   - open questions, risks, concerns, dependencies
2. Work through unresolved design questions one at a time. Ask questions one at a time, choosing the most impactful or most naturally related:
   - consider it carefully (implications, related questions)
   - lightly explore related context if necessary, but keep it bounded
   - suggest 2-3 options, with tradeoffs
   - recommend one, with rationale

Operating principles:

- Prefer multiple choice questions when possible, but open-ended is fine too
- Only one question per message - if a topic needs more exploration, break it
  into multiple questions
- Focus on understanding: purpose, constraints, success criteria, verification
  strategy

Continue in this manner until you have sufficient clarity to begin the design
proper, and the user has accepted your summary.

Once accepted, ensure the slice scope (`slice-nnn.md`) is consistent with and
reflects your current shared understanding before proceeding.

### Present design

1. Draft or revise the design section by section, interactively, rather than dumping a full design at once:
   - Current behavior vs target behavior
   - Code impact summary (paths + intended changes)
   - Verification alignment (what evidence must change/add)
   - Design decisions and remaining open questions
2. When a section shapes later sections, present that section first and treat later sections as provisional until the foundation is coherent.
3. Prefer concrete design detail over hand-wavey prose:
   - Current behavior vs target behavior
   - module responsibility boundaries
   - imports, coupling, cohesion analysis
   - structs/types/interfaces/function signatures
   - data structures & algorithms
   - example data shapes
   - data flow boundaries
   - verification impact
   - invariants & boundary conditions
   - samples of critical code, protocols
   - titles / descriptions of key test cases
   - text c4 diagrams
   - Code impact summary (paths + intended changes)
   - Verification alignment (what evidence must change/add)
   - Impact on design decisions and remaining open questions
4. Perform targeted research if required to ensure fit to implementation surface

### Adversarial review

1. Once the design feels coherent, perform an adversarial self-review before
   treating it as done:
   - attack vague sections, hidden assumptions, weak verification, missing
     code-impact detail, and places where a short sample would remove ambiguity
   - attack missing, misread, or weakly applied ADR/policy/standard constraints
   - ensure doctrinal alignment
   - record the findings in the design doc or companion slice notes as needed
   - "What requirements are implied by these decisions but not captured in
     `design-requirements.toml`?"
2. Review for doctrinal alignment.
   - If the doctrine pass exposes governance conflicts, missing authorities, or
     ambiguous constraints, stop and `/consult` rather than normalizing around
     guesswork.
3. Integrate the feedback before offering next steps.
   - Occasionally this might require revisiting earlier steps.
4. After integrating design feedback, reconcile the owning slice — `slice-nnn.md`
   so scope, risks, acceptance criteria, open questions, and follow-up direction
   still match the revised design, **and** `slice-nnn.toml` (relations, metadata);
   relations move via `doctrine link` and lifecycle status via `doctrine slice
   status` — not hand-edits (`using-doctrine.md` § Relating entities).
5. After the internal adversarial pass is integrated, you MUST offer the user a
   choice:
   1. run a formal hostile pass via `/inquisition`, or print a prompt for an
      external adversarial reviewer.
   2. initiate `/plan` to create the implementation plan and runtime phase sheets.
6. Multiple passes of review & feedback may be required before acceptance. Do
   not presume approval until it is explicitly granted.

If meaningful tradeoffs or uncertainty remain unresolved, stop and `/consult`.

## Guardrails

The design doc is canon for design intent.

- If design and plan conflict, reconcile via the design first.
- Do not present "the whole design" as settled before the foundational sections and decisions have been validated.
- Do not hide unresolved assumptions inside polished prose; name them explicitly.
- Do not confuse detailed design with implementation planning.
- Do not treat a polished full-file rewrite as progress if the hard design questions
  are still unresolved.
- Do not move on to planning while the slice scope still tells an older story than the design.
- Do not treat governance as optional background reading when the design makes architectural or workflow choices.
- Do not record implied requirements as structured fields in prose — use `design-requirements.toml`.
- If an implied requirement crosses altitude boundaries, note ambiguity in `home_hint`; do not force a decision. `/reconcile` handles placement.

## Outcomes

- The design gives a clear, defensible target for implementation.
- Foundational questions are closed or made explicit before downstream planning.
- The design evolves through short feedback loops instead of one large speculative draft.
- Verification impact is explicit before coding starts.
- The author gets an internal adversarial pass and an optional external challenge
  prompt before planning starts.
- Slice scope and design stay aligned before plan/phase work begins.
- Relevant ADRs, policies, and standards shape both the draft and the critical review.
