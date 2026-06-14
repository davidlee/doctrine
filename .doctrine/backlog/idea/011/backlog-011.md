# IDE-011: Structured VH/VA test plan + findings record

## Overlap findings

| Existing surface | Overlap with test plan? | Overlap with findings record? |
|---|---|---|
| **plan.toml** `verification` array | Lists VT/VA/VH criteria + `expects` — but no *how* (setup, procedure) | No |
| **Phase sheet** Tasks | Could host setup steps but unstructured, gitignored | No |
| **Phase sheet** Findings | No | Unstructured prose — gitignored, disposable, inconsistently populated |
| **RV** (review) | No — adversarial, turn-based, baton-gated | Partial — structured findings (severity/title/detail/disposition) but different protocol and purpose |
| **REC** (reconciliation) | No | Partial — `evidence_refs` cite coverage entries but don't host raw findings; only records status transitions |

**Key gap**: VH/VA criteria in `plan.toml` say *what* to check but not *how* (setup, steps, expected observations). Execution results have no durable structured home — phase sheet Findings are gitignored by design. RV has the closest finding shape but is adversarial/turn-based, not a straightforward verification record.

## Design sketch

### Core principle

> Testing artefacts are not claims of quality. They are structured evidence about specific quality questions.

Avoids the lip-service failure mode where a "test plan" becomes a checklist whose real purpose is to let an agent say "verified". Every artefact should answer:

- What risk or behaviour is being interrogated?
- What evidence would increase confidence?
- What evidence was actually observed?
- What remains unproven?

### Recommended entities

Two mandatory, one optional/derived:

- **test_plan** — prospective verification intent (risk/coverage contract)
- **test_observation** — retrospective evidence record
- **test_gap** — optional derived/audit finding (can be a backlog item, risk, or open question initially)

### Test plan: compact but expert-shaped

A test plan is not "steps to execute" — it's a **risk/coverage contract**. Key fields:

- `test_basis` — which requirements, designs, risks, decisions, or other source artefacts derive this plan (steal directly from ISTQB/29119, keep lightweight)
- `intent` — what uncertainty this plan is intended to reduce
- `technique` — scenario, boundary, state, equivalence, regression, exploratory, property, golden, mutation, smoke (nudges away from undifferentiated "tested it")
- `risk_focus` — what risks are addressed (prefer named risks over faux-precise numeric RPN)
- `confidence_goal` — what confidence this plan aims to establish
- `evidence_required` — minimum acceptable evidence for a valid observation (varies by mode: manual vs automated vs agent)
- `non_goals` — explicitly what this plan does NOT prove (prevents "one happy-path test passed, therefore done")

### Test observation: evidence first, interpretation second

Cheaper than a bug report, more structured than a note. Key fields:

- `plan` / `scenario` — traceability back to the plan
- `result` — pass, fail, partial, blocked, inconclusive
- `reproducibility` — once, reproduced, flaky, not-reproducible, not-applicable
- `confidence` — low, medium, high
- `disposition` — none, accepted, proposed-backlog, proposed-risk, invalid, duplicate, superseded

The anti-bullshit core is the body structure:

1. **Expected** — what should have happened
2. **Actual** — what actually happened
3. **Evidence** — command, output, transcript, screenshot, commit, fixture
4. **Interpretation** — what the observation means (separate from raw evidence)
5. **Residual uncertainty** — what this observation does NOT establish (even on passing observations; a pass without this is a confidence smell)

### Anti-bureaucracy design rules

- **Most fields optional, evidence invariants strict.** Require only `test_basis + intent + mode + (risk_focus OR quality_question)` for plans; `result + expected + actual + evidence + residual_uncertainty` for observations.
- **Prefer short enums over prose where automation benefits** (result, mode, level, technique, reproducibility, status, disposition). Prose where judgement matters (intent, expected, actual, interpretation, residual_uncertainty).
- **Require unknowns instead of pretending completeness.** Residual uncertainty exists on every observation.
- **Do not make observations own downstream work.** An observation is evidence; backlog items, risks, and ADRs are interpretations or commitments. Keep them separate.

### Audit automation this enables

- Plan coverage: which accepted specs have no active test plan? Which high-risk areas have only exploratory coverage? Which plans lack test_basis?
- Observation validity: which pass observations lack evidence? Which fail observations lack disposition? Which claim high confidence after one unreproduced run?
- Verification readiness: which slice acceptance gates have no passing observation? Which observations are stale relative to changed source files? Which automated tests are expensive, flaky, or overrepresented as end-to-end checks?

### Recommended stance

> Test plans define evidence strategy. Test observations record bounded evidence. Passing observations increase confidence; they do not certify truth. Failed or inconclusive observations feed backlog, risk, or decision artefacts. Every observation must preserve residual uncertainty.

## Open questions

- Should this be one new entity kind with a `kind` discriminator (`test_plan` / `test_observation`) or two separate kinds?
- `test_gap`: first-class entity or derived from plan→observation gap analysis?
- How does `test_observation` differ from RV findings? RV is adversarial/turn-based with baton coordination; TO is straightforward evidence capture. Could they share a common finding shape?
- Does the `evidence_required` enum need to be project-customisable, or is a closed set sufficient?
- How do observations relate to the existing `plan.toml` verification criteria? Do they replace the phase sheet Findings section, supplement it, or become the structured form of it?
