# PHASE-05 Triage — spec-driver 86 → doctrine orientation corpus

Disposition of every shipped spec-driver memory under
`/home/david/dev/spec-driver/.spec-driver/memory/`.

- **(a) directly transferable** — same topic, rewrite for doctrine.
- **(b) topic-applicable** — doctrine authors a fresh master on this topic.
- **(c) inapplicable** — spec-driver-internal, stack-specific
  (Python/Typer/Textual/pylint/PyYAML/nix), or a contributor/dev-facing gotcha
  that belongs in doctrine's own `items/`, never in the downstream-driver corpus
  (design D7).

Doctrine target slugs are the masters planned for PHASE-05 (the mint+author list).
A `(b)` master may have several source rows or none (authored fresh from the
OQ-A skeleton).

| # | spec-driver id | a/b/c | doctrine target master slug / drop rationale |
|---|---|---|---|
| 1 | mem.artifact-pattern-sync | c | spec-driver-internal: Python/TS artifact-pattern mirror coupling |
| 2 | mem.concept.spec.assembly-only-taxonomy | c | spec-driver-internal: assembly-vs-unit tech-spec taxonomy |
| 3 | mem.concept.spec-driver.audit | b | pattern.doctrine.core-loop (audit = verify step) |
| 4 | mem.concept.spec-driver.backlog | c | spec-driver-internal: no backlog entity in doctrine |
| 5 | mem.concept.spec-driver.ceremony.pioneer | c | spec-driver-internal: no ceremony modes in doctrine |
| 6 | mem.concept.spec-driver.ceremony.settler | c | spec-driver-internal: no ceremony modes in doctrine |
| 7 | mem.concept.spec-driver.ceremony.town-planner | c | spec-driver-internal: no ceremony modes in doctrine |
| 8 | mem.concept.spec-driver.contract | c | spec-driver-internal: no auto-generated contract artifact in doctrine |
| 9 | mem.concept.spec-driver.delta | b | pattern.doctrine.core-loop (delta≈slice = scope step) |
| 10 | mem.concept.spec-driver.design-revision | b | pattern.doctrine.core-loop (DR≈design step) |
| 11 | mem.concept.spec-driver.philosophy | b | signpost.doctrine.overview |
| 12 | mem.concept.spec-driver.plan | b | pattern.doctrine.core-loop (IP≈plan step) |
| 13 | mem.concept.spec-driver.posture | c | spec-driver-internal: workflow.toml ceremony/posture config absent in doctrine |
| 14 | mem.concept.spec-driver.relations | b | concept.doctrine.entity-engine (relations/edges/traceability) |
| 15 | mem.concept.spec-driver.requirement-lifecycle | c | spec-driver-internal: coverage-derived requirement lifecycle mechanics |
| 16 | mem.concept.spec-driver.revision | c | spec-driver-internal: no RE-* revision entity in doctrine |
| 17 | mem.concept.spec-driver.spec | b | concept.doctrine.entity-engine (specs as peer entities) |
| 18 | mem.concept.spec-driver.truth-model | b | concept.doctrine.storage-model (two-sources-of-truth → the storage rule) |
| 19 | mem.concept.spec-driver.verification | b | pattern.doctrine.core-loop (VT evidence = verify step) |
| 20 | mem.fact.architecture.core-misplaced-modules | c | spec-driver-internal: core/ module placement |
| 21 | mem.fact.architecture.import-linter-supekku-blindspot | c | stack-specific: Python import-linter |
| 22 | mem.fact.autobahn-independence | c | spec-driver-internal: spec-driver/autobahn dependency boundary |
| 23 | mem.fact.backlog.relations-in-frontmatter | c | stack-specific: Python dataclass/frontmatter gotcha |
| 24 | mem.fact.claude-code.context-loading | c | out-of-corpus: general Claude Code context-loading mechanics, not doctrine orientation |
| 25 | mem.fact.cli.clirunner-no-mix-stderr | c | stack-specific: Typer/Click test runner |
| 26 | mem.fact.cli.typer-exit-inherits-runtimeerror | c | stack-specific: Typer |
| 27 | mem.fact.cli.typer-group-subclass | c | stack-specific: Typer |
| 28 | mem.fact.core.markdown-load-error-taxonomy | c | stack-specific: Python error taxonomy |
| 29 | mem.fact.core.pyyaml-problem-mark-guard | c | stack-specific: PyYAML |
| 30 | mem.fact.pi.append-system-md-discovery | c | spec-driver-internal: pi extension mechanics |
| 31 | mem.fact.pi.session-shutdown-hook-timing | c | spec-driver-internal: pi extension hook timing |
| 32 | mem.fact.project.de-011-enum-sources | c | spec-driver-internal: DE-011 enum sourcing |
| 33 | mem.fact.pylint.recursive-test-ignores | c | stack-specific: pylint |
| 34 | mem.fact.skills.source-location | c | doctrine-dev gotcha: edit-source-not-installed (contributor-facing, lives in items/) |
| 35 | mem.fact.spec-driver.coverage-gate | c | spec-driver-internal: delta completion coverage gate |
| 36 | mem.fact.spec-driver.requirement-bundle-files | c | spec-driver-internal: requirements/ bundle files |
| 37 | mem.fact.spec-driver.status-enums | c | spec-driver-internal: status enum schema |
| 38 | mem.fact.tui.widget-selection | c | stack-specific: Textual widgets |
| 39 | mem.fact.validation.audit-gate-test-impact | c | spec-driver-internal: WorkspaceValidator audit gate |
| 40 | mem.fact.validation.cross-project-refs-skipped | c | spec-driver-internal: validator cross-project refs |
| 41 | mem.fact.workflow.disposition-authority-required | c | stack-specific: pydantic FindingDisposition field |
| 42 | mem.fact.yaml.strenum-serialization | c | stack-specific: PyYAML StrEnum |
| 43 | mem.gotcha.migration.sys-modules-registration | c | stack-specific: Python sys.modules/dataclass |
| 44 | mem.gotcha.pydantic.migration | c | stack-specific: pydantic |
| 45 | mem.gotcha.textual.tab-key-handling | c | stack-specific: Textual |
| 46 | mem.pattern.architecture.domain-migration | c | spec-driver-internal: spec_driver.domain migration |
| 47 | mem.pattern.architecture.migration-principles | c | spec-driver-internal: DE-125 migration principles |
| 48 | mem.pattern.cli.skinny | b | pattern.doctrine.conventions (skinny CLI ≈ pure/imperative split) |
| 49 | mem.pattern.dr-authoring-review-loop | b | pattern.doctrine.core-loop (design → adversarial review/inquisition loop) |
| 50 | mem.pattern.events.cli-middleware | c | spec-driver-internal: event-emission middleware |
| 51 | mem.pattern.formatters.soc | c | spec-driver-internal: formatters/ architecture detail |
| 52 | mem.pattern.frontmatter.prettier-compat | c | stack-specific: YAML CompactDumper/prettier |
| 53 | mem.pattern.git.spec-driver-commit-cleanliness | b | pattern.doctrine.conventions (frequent slice-scoped conventional commits) |
| 54 | mem.pattern.installer.boot-architecture | b | fact.doctrine.storage-tiers (managed/authored ownership; boot reference) |
| 55 | mem.pattern.phase.canonical-fields | c | spec-driver-internal: phase frontmatter schema |
| 56 | mem.pattern.phase.contract-vs-progress | b | concept.doctrine.storage-model (contract-vs-progress = the storage rule) |
| 57 | mem.pattern.phase.frontmatter-block-precedence | c | spec-driver-internal: frontmatter/block reading precedence |
| 58 | mem.pattern.project.completion | c | spec-driver-internal: project completion config stub |
| 59 | mem.pattern.project.workflow | c | spec-driver-internal: project workflow config stub |
| 60 | mem.pattern.pylint.summary-workflow | c | stack-specific: pylint |
| 61 | mem.pattern.skills.memory-retrieval-and-wrapup | a | concept.doctrine.memory-model (scoped retrieve + wrap-up capture) |
| 62 | mem.pattern.spec-driver.block-class-data-taxonomy | c | spec-driver-internal: block-class .data/.parse taxonomy |
| 63 | mem.pattern.spec-driver.core-loop | a | pattern.doctrine.core-loop |
| 64 | mem.pattern.spec-driver.create-phase-convention | b | pattern.doctrine.conventions (immutable PHASE-NN ids) |
| 65 | mem.pattern.spec-driver.delta-completion | b | pattern.doctrine.core-loop (delta-completion ≈ /close) |
| 66 | mem.pattern.spec-driver.field-conditional-rules | c | spec-driver-internal: FieldMetadata conditional rules |
| 67 | mem.pattern.spec-driver.frontmatter-compaction | c | spec-driver-internal: frontmatter compaction annotations |
| 68 | mem.pattern.spec-driver.metadata-test-placement | c | spec-driver-internal: metadata test mirror rule |
| 69 | mem.pattern.spec-driver.metadata-validator-strictness | c | spec-driver-internal: MetadataValidator strict-mode |
| 70 | mem.pattern.spec-driver.shared-block-id-patterns | c | spec-driver-internal: block ID regex patterns |
| 71 | mem.pattern.testing.nix-pytest-via-python | c | stack-specific: uv/pytest/nix |
| 72 | mem.pattern.tui.screen-lifecycle | c | stack-specific: Textual screen lifecycle |
| 73 | mem.pattern.typechecking.ty-known-issues | c | stack-specific: ty typechecker |
| 74 | mem.pattern.validation.per-kind-block-wiring | c | spec-driver-internal: WorkspaceValidator block wiring |
| 75 | mem.pattern.validation.warning-triage | c | spec-driver-internal: validate-workspace warning triage |
| 76 | mem.reference.spec-driver.workflow-config | c | spec-driver-internal: workflow.toml config reference |
| 77 | mem.reference.workflow-commands | b | signpost.doctrine.cli-command-map (authored as signpost, not reference — Charge VIII) |
| 78 | mem.signpost.spec-driver.ceremony | c | spec-driver-internal: ceremony-mode selection |
| 79 | mem.signpost.spec-driver.file-map | a | signpost.doctrine.file-map |
| 80 | mem.signpost.spec-driver.lifecycle-start | a | signpost.doctrine.lifecycle-start |
| 81 | mem.signpost.spec-driver.overview | a | signpost.doctrine.overview |
| 82 | mem.signpost.spec-driver.skill-authoring | c | dev-facing: authoring/refining skills is contributor work, not driver orientation |
| 83 | mem.signpost.spec-driver.upgrade.metadata-blocks-0-10 | c | spec-driver-internal: 0.10 metadata-blocks upgrade runbook |
| 84 | mem.system.dispatch.architecture | c | spec-driver-internal: /dispatch internals (doctrine dispatch is an unimplemented placeholder) |
| 85 | mem.system.tui.architecture | c | stack-specific: Textual TUI architecture |
| 86 | mem.thread.de-041 | c | spec-driver-internal: DE-041 working thread |

## Disposition totals

- (a) directly transferable: 5 — rows 61, 63, 79, 80, 81
- (b) topic-applicable: 17 — rows 3, 9, 10, 11, 12, 14, 17, 18, 19, 48, 49, 53, 54, 56, 64, 65, 77
- (c) inapplicable: 64
- **Total: 86**

## Skeleton coverage (every OQ-A topic → ≥1 master)

| OQ-A topic | type | master slug | source rows |
|---|---|---|---|
| overview | signpost | signpost.doctrine.overview | 11, 81 |
| file-map / layout | signpost | signpost.doctrine.file-map | 79 |
| lifecycle-start (route→slice→design→plan→phase→audit→close) | signpost | signpost.doctrine.lifecycle-start | 80 |
| skill / route map | signpost | signpost.doctrine.skill-map | (fresh) |
| CLI command map (as signpost, not reference) | signpost | signpost.doctrine.cli-command-map | 77 |
| storage model + storage rule | concept | concept.doctrine.storage-model | 18, 56 |
| entity engine | concept | concept.doctrine.entity-engine | 14, 17 |
| memory model (capture vs shipped corpus) | concept | concept.doctrine.memory-model | 61 |
| the routing gate | concept | concept.doctrine.routing-gate | (fresh) |
| the core loop | pattern | pattern.doctrine.core-loop | 3, 9, 10, 12, 19, 49, 63, 65 |
| conventions (commits, pure/imperative, behaviour-preservation, immutable ids) | pattern | pattern.doctrine.conventions | 48, 53, 64 |
| TDD red/green/refactor | pattern | pattern.doctrine.tdd-loop | (fresh) |
| CLI-is-source-of-truth | fact | fact.doctrine.cli-source-of-truth | (fresh) |
| authored vs runtime vs derived tiers | fact | fact.doctrine.storage-tiers | 54 |

14 masters; every OQ-A skeleton topic has ≥1 master. No uncovered topic.
