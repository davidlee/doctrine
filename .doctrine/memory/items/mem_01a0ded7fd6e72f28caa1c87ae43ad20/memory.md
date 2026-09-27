# A review reads `done` only once concluded — and raise or reopen un-concludes it

Since SL-268 (its design decision D2), `derived_status`
(`src/review_ledger/derive.rs`) is: **`done` ⇔ every finding terminal
(`verified`/`withdrawn`) ∧ the pass concluded**. All-terminal but unconcluded —
an empty ledger included — reads `active · await=raiser`. So `done · await=none`
now implies `concluded`; the old "done is not concluded" trap is gone.

What still bites:

- **Conclude requires `--basis`** (what the pass examined). It is recorded as a
  durable turn. `doctrine review conclude RV-NNN --basis - <<'EOF' … EOF`.
- **A raise or a reopen after conclude clears it.** The ledger drops back to
  `active`, and the raiser must conclude again. Conclude is the pass's *last*
  act.
- **The design run still reads the marker.** `review-disposed {conducted:
  RV-NNN}` is refused over an unconcluded ledger (`read_pass_facts`,
  `src/review_ledger/gate.rs`, reads absence as `false`). A `conducted`
  disposition already recorded stands even if the ledger is later reopened.
- **Open findings do not block conclude** — conclude marks the raiser done
  looking; disposing is the responder's work afterwards. The ledger is simply
  not `done` until those findings are terminal too.

Supersedes `mem.pattern.review.done-is-not-concluded`, true when written (SL-249,
2026-08-08) and made false by D2. See also [[mem.signpost.doctrine.review]],
[[mem.pattern.review.reopen-to-revise-verified]].
