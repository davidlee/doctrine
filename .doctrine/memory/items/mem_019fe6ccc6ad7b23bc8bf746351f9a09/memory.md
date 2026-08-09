## The defect

A mutation battery asks: *does some test fail when I break this?* Under
`-D warnings` a large share of the obvious mutations never get that far — they
leave a binding unused, a constant dead, or a variant unreachable, and the
**compiler** rejects them. A battery that scores "the build did not succeed" as
a conviction credits the guard with evidence it never produced. The assertion
under test may be vacuous and you will not find out.

This has now bitten SL-248 twice in consecutive tasks:

- PHASE-10 `T9`: removing an unread path made a constant dead code, so the
  mutant failed `-D dead-code`.
- PHASE-10 `T10`: three of twelve arms (emptying a rendered list, dropping a
  format argument twice over) left `unproven` / `remedy` unused.

## The rule

**A mutant must compile, or it is not a mutant.** Make the battery say so:
scan the build output for `error[E` / `could not compile` and report
`<did not compile>` as a distinct outcome from a failing test. Then rebuild
the arm in a shape that compiles.

## How to rebuild an arm that will not compile

The repair is almost always to **degrade** the value rather than remove it —
which is the stronger mutant anyway, because a degraded reading defeats an
absence probe as easily as an empty one:

| will not compile | compiles, and is sharper |
|---|---|
| drop the list | truncate it to its first element |
| drop one of two format arguments | transpose the two |
| omit a field | render its first word / first token |
| delete a call | replace it with a constant of the same type |

## Related

- `mem.pattern.tests.absence-probes-must-convict-an-unread-surface` — the
  same family one level down: a probe asserting an absence cannot tell "held
  nothing" from "read nothing".
- Restore mutated source **by copy** from a pristine snapshot and `diff -q`
  before the next arm; never `git checkout <ref> --`, whose empty-pathspec
  fallback is a whole-worktree branch switch.
