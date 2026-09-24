A design run's `forward.ready` payload mints `advance-<to>-r<revision>` and steps to `-2`, `-3` … past any retained receipt (`envelope.rs`). At the binary level the "already taken" state is unreachable: `run::admit` answers a receipt by id first, and any submission sharing that id at the revision that minted it *advances* the revision — so the next read re-mints at the new revision. A different payload under the same id is refused (`SubmissionReplayed`) and records nothing.

So an e2e test for the squatted-id case (SL-262 PHASE-03 `squatted_id_does_not_break_ready`) must seed the receipt directly. Do it through the wire type, not a TOML splice:

    let mut run = snapshot::parse(&fs::read_to_string(&fixture.snapshot)?)?;
    run.receipts.receipts.push(Receipt { submission, revision: run.run.revision, digest: "sha256:…", delegation: None, delegation_state: None });
    fs::write(&fixture.snapshot, snapshot::to_toml(&run)?)?;

`ReceiptGroup.receipts` is `pub(crate)`, so the `#[path]`-included leaf in a `tests/e2e_design_*.rs` crate reaches it. Keep the seeded revision equal to the run's current revision, or the mint's suffix arithmetic is not what the test thinks it is.

Related, same suite: a stage move batched with the acts that satisfy it **is** admitted (`admit` evaluates the snapshot the batch produces), while the read before it says `blocked` — forward describes stored state, so batching is out of its scope.