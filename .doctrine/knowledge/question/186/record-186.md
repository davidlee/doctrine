# QUE-186: Regression and forward gate re-clearance semantics

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

When a run regresses directly from a later stage such as `reviewing` to
`drafting`, `inquiring`, or `exploring`, which downstream gate evidence becomes
stale? On returning, must it advance through each intervening boundary, may it
jump after cumulatively proving every gate, or can still-fresh evidence be
reused without ceremonial re-approval?
