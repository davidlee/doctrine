# Debug-build scale timings run ~10x the release probe numbers; cliff-test bounds must budget for debug

The SL-038 scale harness (`crates/cordage/examples/scale_harness.rs`, debug build)
measured the eviction-fixpoint quadratic at **41s for n=100** (dense_evict 100,100);
the deleted RSK-003 probe reported **3.5s** for the same shape. ~10x. The probe was
almost certainly a release build; `cargo test` / `just check` run **debug**, so a red
sized from release numbers can blow its own loose time bound.

Concrete fallout: design §6.3's quadratic red (dense_evict 100/200, assert `< 120s`)
is unviable in debug — n=200 ≈ 17x·41s ≈ ~700s, far over 120s and ~12min wall-clock.
Measured-ratio reds must pick N pairs that stay well under their bound **in debug**:
the harness pinned quadratic at 50/100 (2.2s/41s, ratio 18.5x) and evaluate at
2000/4000 (1.8s/7.7s, ratio 4.25x, ~9.5s total) for exactly this reason.

**Why:** the cliff is real in both profiles; only the constant differs, but a test
gate's wall-clock and its sanity bound live in debug. **How to apply:** when sizing a
slow characterization red, measure the candidate N in debug (not release, not
extrapolated from a release probe) and leave generous bound headroom. See
[[mem.pattern.testing.black-box-cli-golden]].
