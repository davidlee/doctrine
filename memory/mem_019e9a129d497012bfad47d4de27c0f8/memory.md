# Doctrine TDD red green refactor loop

Every phase is built **red → green → REFACTOR**. This is non-negotiable inside
`/execute` — the skill drives it; this points at the discipline.

1. **Red.** Write a failing test first. It must fail for the right reason (run it,
   watch it fail) — a test that was never red proves nothing.
2. **Green.** Write the minimal code that makes it pass. No more.
3. **Refactor.** Now clean up — dedupe, rename, extract — with the test as your
   safety net. This third step is the one agents skip; don't.

**Test behaviour, not trivial implementation.** Assert on observable outcomes, not
private wiring — that keeps tests from going brittle when you refactor. Build and
improve test helpers and fixtures as you go; they are first-class.

**End every phase green.** A phase closes with the suite passing — tests clean,
lint clean, formatted. Don't flip a phase `completed` over a red bar.

**The existing suites are the proof.** When you touch shared machinery, the already-
green tests are the behaviour-preservation gate — they must stay green *unchanged*
(see [[pattern.doctrine.conventions]]). Adding behaviour means adding tests, not
rewriting old ones to fit.

`/execute` owns the per-phase mechanics (flip `in_progress`, implement, end green,
flip `completed`); this loop is step 5 of that skill, and the inner engine of the
broader [[pattern.doctrine.core-loop]].
