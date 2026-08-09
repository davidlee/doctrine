Running the `doctrine-control` test binary directly gives correct timing but a
**false red**.

```
target/debug/deps/doctrine_control-<hash> conformance::tests
# → 92.32s (correct), but:
# no_ignored_test_in_this_crate_stands_in_for_a_claim FAILED
# "the walk did not reach this file, so it read the wrong tree: []"
```

**Cause.** That test (SL-248 PHASE-10 `T11`, criterion `EX-14`) audits the crate
source for `#[ignore]` without an `instrument:` reason. It locates the source
tree via `CARGO_MANIFEST_DIR`, which **cargo sets for a test process and a bare
binary invocation does not**. Confirmed as a probe/control pair on the single
test: env set → `ok`; env unset → `FAILED`.

**Rule.** Shard, filter or time this suite **through `cargo`**
(`cargo test -- <filter>`), or export `CARGO_MANIFEST_DIR` yourself:

```
CARGO_MANIFEST_DIR=$PWD/crates/doctrine-control target/debug/deps/doctrine_control-<hash> ...
```

**Do not "fix" the test.** It is right to depend on the variable and right to
refuse. It fails *loudly on an empty walk* rather than reporting a pass over
zero files — the absence-probe lesson (a probe cannot distinguish "held nothing"
from "read nothing"). Relaxing it to tolerate an empty walk converts it into a
vacuous pass.

Related: `mem.fact.testing.runtime-manifest-dir`.
