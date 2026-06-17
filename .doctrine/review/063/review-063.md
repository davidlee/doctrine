# Review RV-063 — plan of SL-091

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

The Inquisitor interrogates the **plan** of SL-091 — the phase breakdown,
verification chain, and tooling scaffold that must guard a TypeScript migration
against the known pathologies of LLM-generated code.

## Lines of attack

### L1 — The Absent Lint Gate (mortal sin)

The plan's verification chain is `tsc --noEmit` + `just gate` + manual smoke.
ESLint is **not mentioned** in any phase, not in any EN/EX/VT, not in any script —
it does not exist in the plan's universe. `tsc` catches type errors but is blind
to `any`-widening, floating promises, import/export drift, `innerHTML` writes,
and stale global patterns. An LLM implementing this plan will produce code that
compiles under `tsc` but ships with five classes of silent defect that only a
lint gate catches. This is the gravest heresy — the plan arms the watchman
(`tsc`) but not the inquisitor (`eslint`).

### L2 — Incomplete tsconfig hardening

The design says "strict, moduleResolution bundler, target es2020." The plan does
not specify the tsconfig at all — it inherits by reference. But the design's own
phrasing is the minimum, not the full set: `noUncheckedIndexedAccess`,
`verbatimModuleSyntax`, `exactOptionalPropertyTypes`, `noImplicitOverride`,
`noImplicitReturns`, `useUnknownInCatchVariables`, and `skipLibCheck: false` are
all necessary to close the holes `strict` alone leaves open. Without these, an
LLM will introduce `undefined` reads on sparse arrays, side-effect imports,
missing `override` keywords, and swallowed catch types.

### L3 — package.json scripts missing the chain

The plan mentions `bun run dev` and `bun run build` but never defines the
scripts. The `build` script must enforce the gate — `tsc --noEmit` → `eslint
--max-warnings=0` → `vite build`. If `build` is `vite build` alone, every check
can be skipped by a busy implementer.

### L4 — Missing devDependencies

No phase adds `eslint`, `typescript-eslint`, `@eslint/js`, or `globals` to
package.json. The scaffold phase (PHASE-00) lists `typescript` and `vite` as
devDeps; eslint packages are absent.

### L5 — Verification gaps

Multiple verification items assert `tsc --noEmit passes` with zero errors. None
asserts `eslint --max-warnings=0 passes` with zero errors. The VT gates are
incomplete — they gate type safety but not architectural conformance, import
hygiene, or async correctness.

### L6 — `just gate` undefined for web/map

The plan references `just gate` in verification but does not define what `just
gate` runs for the frontend. If `just gate` only runs `cargo clippy`, the
TypeScript lint gate is never validated in CI.

### L7 — No restricted-syntax or global bans

The design bans JSX and framework patterns (D6). The plan has no mechanism to
enforce this — no `no-restricted-syntax` rules, no `no-restricted-globals`. An
LLM adding `import React from 'react'` or `window.state = {}` will pass both
`tsc` and the plan's verification, and the heresy will only be caught in manual
smoke.

## Held to

The 13 invariants primed in the domain_map (§2). The plan must answer for every
one.

## Synthesis

**Judgement**: GUILTY — of omission, not commission. The plan is sound in its
leaf→root sequencing, its module boundaries, its verification intent, and its
Rust integration. It is silent where silence is heresy: it arms the type-checker
but not the lint-inquisitor, and thus leaves the migration's architectural
guardrails to the unreliable mercy of manual smoke and LLM self-discipline.

Eight findings were raised, all verified with `fix-now` disposition. The penance
is surgical — eight amendments to the plan artifact, not a plan rewrite:

### Ordered penance (to be applied to plan.toml + plan.md before PHASE-00 begins)

1. **F-1 (blocker)**: Add ESLint as a first-class verification gate. Define the
eslint.config.js in PHASE-00 scaffold, add eslint VTs to every conversion
phase, and adopt the user-provided config (flat config, js.configs.recommended +
tseslint.configs.strictTypeChecked + tseslint.configs.stylisticTypeChecked, with
LLM-damage-containment rules: no-explicit-any, no-unsafe-*, no-floating-promises,
no-misused-promises, switch-exhaustiveness-check, strict-boolean-expressions,
consistent-type-imports, await-thenable, require-await, return-await,
reportUnusedDisableDirectives: error).

2. **F-2 (blocker)**: Harden tsconfig.json beyond the design's minimum. Add
noUncheckedIndexedAccess, verbatimModuleSyntax, exactOptionalPropertyTypes,
noImplicitOverride, noImplicitReturns, useUnknownInCatchVariables, skipLibCheck:
false, noUnusedLocals, noUnusedParameters, isolatedModules, allowJs: false,
checkJs: false.

3. **F-3 (major)**: Define package.json scripts explicitly — `lint` (eslint
--max-warnings=0), `typecheck` (tsc --noEmit), `build` (typecheck → lint → vite
build), `dev` (vite).

4. **F-4 (major)**: Add eslint, typescript-eslint, @eslint/js, globals to
PHASE-00 devDependencies.

5. **F-5 (major)**: Add eslint VT items to every conversion phase (PHASE-00
through PHASE-10).

6. **F-6 (major)**: Specify that `just gate` must include `cd web/map && bun run
lint` alongside cargo clippy.

7. **F-7 (major)**: Include restricted-syntax bans in eslint.config.js: JSXElement,
innerHTML/outerHTML assignment, insertAdjacentHTML calls, eval, new Function,
window.* assignment. Add no-restricted-globals for bare `event`. Per-file override
for render.ts/concept-map.ts (local eslint-disable-next-line, not blanket weakening).

8. **F-8 (minor)**: Specify eslint.config.js flat config format (ESLint v9+,
required by typescript-eslint v8+).

### Standing risks

- **strict-boolean-expressions**: Will cause pain during migration. LLMs will
try to `as any`-cast or add `!= null` guards that mask the real type bug. The
rule makes those moves noisy — each `// eslint-disable-next-line` must be
reviewed. Acceptable cost.

- **innerHTML ban**: render.ts and concept-map.ts currently use innerHTML
liberally. During migration, these sites will need local eslint-disable-next-line
with explicit sanitization rationale. The ban is not weakened; the escape hatches
are auditable.

- **No Prettier**: Formatting normalization is deferred. If formatting churn
becomes a problem during migration, add Prettier then — not now. The config's
job is correctness and architecture enforcement, not aesthetic normalization.

### Second inquisition — testing approach (F-9 through F-13)

A follow-on interrogation of the plan's testing story revealed a **structural
contradiction**: the plan deletes .js files from PHASE-01 onward but only
converts test.html to ES modules in PHASE-10. test.html references the deleted
.js files as globals — between PHASE-01 and PHASE-10, test.html is un-runnable.
This contradicts PHASE-01 VT-2 and PHASE-05 VA-5 which demand it pass.

Five additional findings were raised and verified:

9. **F-9 (blocker)**: Keep old .js files until PHASE-10 (Option A). Change
PHASE-01 through PHASE-09 EX-3 from 'X.js deleted' to 'X.js retained; deletion
deferred to PHASE-10.' Add PHASE-10 EX-6: 'All old .js files deleted.' This
keeps test.html functional throughout the migration.

10. **F-10 (major)**: Specify test.html inline test code migration to ES module
imports. The inline test blocks must add `import { ... } from './src/X.ts'`
statements at the top of the module script. The test logic itself is preserved
unchanged.

11. **F-11 (major)**: Specify test.html serving path at each stage: Stage 1
(PHASE-00 through PHASE-09) — served by Rust map server at /assets/test.html
with old .js globals. Stage 2 (PHASE-10) — served by Vite dev server at
/test.html with ES module imports. Stage 3 (production) — not embedded.

12. **F-12 (major)**: Add verification that test.html pass/fail counts are
identical before and after conversion. Record the baseline with old .js, compare
after PHASE-10 conversion — any difference signals a semantic regression.

13. **F-13 (minor)**: Acknowledge test.html's manual verification model honestly
in the plan: no structured runner, no CI exit code, agent reads the <pre> element
and confirms 0 FAIL lines. This is acceptable — IMP-088 is deferred.

### Updated standing risks

- **test.html fragility**: The inline test code mutates `state.graph` directly
for test isolation. The ES module `state` import is the live singleton — mutations
persist across test blocks. The test blocks already reset state.graph before each
scenario (they call model.normalizeGraph(raw) with small datasets), but this
pattern is fragile. An LLM could add a test that doesn't reset state and pollute
subsequent assertions. Add a note to the test: 'each test block must call
normalizeGraph() with its own data before asserting.'

- **test.html performance**: The tests run in a browser with no timeout, no
parallelism, no suite runner. At 50+ assertions, the runtime is inconsequential.
If it grows, add a note that a test framework (IMP-088) is the right home for
scaling.

### Tolerated taint

None. All 13 findings are fix-now, verified. Both inquisitions are resolved.
The plan is reconcilable — the heresies were omission (no lint gate) and
contradiction (test.html migration gap), not malignancy. The Inquisitor has
burned both paths. The migration may now proceed with a watchman (`tsc`), an
inquisitor (`eslint`), AND a witness (`test.html`) that stays alive throughout.

---

**HERESIS URITOR; DOCTRINA MANET**
