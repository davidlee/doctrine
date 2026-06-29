# IMP-214: Make resolve_runner testable — inject command factory or separate pure which-check from execution

RV-194 F-6 (follow-up): `resolve_runner()` has no test coverage because
the bunx-vs-npx fallback decision executes a real process. Refactor so the
resolution logic is testable — either inject a Command factory, or separate
the pure "which" check from execution. The Runner trait seam exists but
isn't used at the resolution boundary.

