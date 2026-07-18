# IMP-297: REV introduce is SPEC-only — no revision path to introduce a product (PRD) requirement

## Gap

`doctrine revision change add --action introduce --member-of <X>` refuses a PRD:
`--member-of must name a SPEC, got a PRD`. So there is **no revision path to
introduce a new *product* (PRD) requirement** — only to `modify`/`status` existing
ones. Yet `doctrine spec req add` accepts `PRD-NNN` directly, and PRDs carry
requirements (e.g. PRD-006 REQ-043..048). The REV vehicle can amend product
requirements but cannot originate them.

## Where it surfaced

REV-028 (RFC-021 C1 projection revision). RV-285 F-6 asked for PRD-006 product
requirements enforcing minimal projection. Worked around by folding the intent into
a comprehensive **revision of REQ-043** and pushing the testable decomposition to
SPEC-009 (`introduce` rows), justified by the product/tech altitude split. That
workaround is defensible, but the mechanism *forced* it rather than the author
choosing it — a latent constraint worth an explicit decision.

## Question to settle

Is SPEC-only `introduce` **intentional** (product intent originates at PRD authoring;
revisions only refine it) or an **oversight**? If intentional, document it (and have
the CLI say so). If not, allow `introduce --member-of PRD-NNN`.

Distinct from [[IMP-074]] (auto-landing introduce/create rows vs surfaced-for-manual)
— that is about *apply* mechanics; this is about *which member kinds* `introduce`
accepts at all. Related altitude context: IMP-097 (product-vs-tech requirement
placement).
