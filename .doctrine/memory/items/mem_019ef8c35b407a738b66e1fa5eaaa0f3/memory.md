# Stale test binary embeds the OLD fixture corpus; a vocabulary change reads as a logic regression

**Symptom (SL-149 P05).** After the relation-vocabulary change (removing the
`Specs`/`Requirements` labels, adding `references`+role), an integration test
asserted on an edge count and failed `1 != 2` — looking exactly like a logic
regression in the migration or the parser.

**Real cause.** The compiled integration-test binary **embeds the fixture corpus**
(test data baked into the binary at compile time). The vocabulary change made the
*fresh* `doctrine` binary stop parsing the pre-change labels, but the **not-rebuilt
test binary still carried the OLD fixture** — so the assertion compared new-parser
behaviour against stale embedded data. The defect was build staleness, not logic.

This is a distinct angle from the stale-*doctrine*-binary cluster
([[mem_019ea4e7c18f78d192ddd738ff6052bf]] and siblings): there the *spawned/installed*
binary is stale; here the staleness is **fixture data compiled into the test binary
itself**. Same root mechanism as a worktree compiling the wrong corpus
([[mem_019ec5bdca9c76219f4e18f37a2ae0b7]]).

**Fix.** `touch` the changed sources **including `tests/*.rs`** before
`cargo test` / `just check`, then rebuild — the incremental build then re-embeds the
current fixture. (Shared `CARGO_TARGET_DIR` in the jail makes this worse:
[[mem_019edefff21776a296a5ba6a4b84c4dc]], [[mem_019ea4e7c18f78d192ddd738ff6052bf]].)

**Secondary footgun that hid it.** `just gate 2>&1 | tail` reports exit **0** even
when `just` fails — the pipeline's exit status is `tail`'s, not `just`'s. A red gate
read as green. Check the command's own exit code directly (`just gate; echo $?`), or
run each check in its own command; never judge pass/fail through `| tail`.
