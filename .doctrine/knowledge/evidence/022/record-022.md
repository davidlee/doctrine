# EVD-022: Pre-split nineteen-row admission baseline

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

`just capsule-verify`, captured at `4662e64eb` — the last commit before any of
`SL-253`'s code lands — inside the project's bubblewrap jail. This is the
artefact `inq-5`'s behaviour-preservation argument is measured against, and it
could only be captured *before* the split.

`ISS-339` records that `backend verify` has never been run off-jail. That is not
a blocker for this baseline: nested `bwrap` works inside the jail
(`bwrap --unshare-all --ro-bind / / …` returns 0), so the run is reproducible on
the same host the split will be developed on.

## Host

```
uname:      Linux 7.1.6 x86_64
bwrap:      bubblewrap 0.11.2
setsid:     util-linux 2.42.2
socat:      socat 1.8.1.3
userns:     user:[4026535221]
pidns:      pid:[4026535229]
netns:      net:[4026531833]
uid_map:     1000 0 1
gid_map:     100 0 1
NoNewPrivs: 1
```

## Verdict

```
backend=bubblewrap os=linux kernel=7.1.6 arch=x86_64 date=2026-08-12
outcome=admitted
row Property(FreshMutableState)=Proven
row Property(BoundedInputSet)=Proven
row Property(DeniedCanonicalStateAndCredentials)=Proven
row Property(BoundedFilesystemVisibility)=Proven
row Property(ExplicitNetworkPosture)=Proven
row Property(DeterministicWorkingDirectory)=Proven
row Property(ProcessTreeTeardown)=Proven
row Property(TrustedTerminationObservation)=Proven
row Property(ImmutableInputSet)=Proven
row Property(ClosedDescriptorSet)=Proven
row Property(ClosedEnvironment)=Proven
row Property(OwnedStandardStreams)=Proven
row Property(MappedCapsuleIdentity)=Proven
row Property(ConfinedCapabilities)=Proven
row Axis(Checkout)=Proven
row Axis(Repository)=Proven
row Axis(Runtime)=Proven
row Axis(TemporaryState)=Proven
row Axis(Process)=Proven
claim sec-4/rewriting the policy inside a capsule does not change the bound policy=Passed
claim sec-3/the clone's object set is exactly the export's=Passed
claim sec-5/the capacity probe reads real space at the path it is given=Passed
claim sec-5/the capacity probe reads the filesystem the capsule root is on=Passed
observation sec-9/the capsule's supplementary groups are unmapped rather than dropped=Read { value: "65534 65534 65534 65534 65534 1000 65534 65534 65534", caveat: None }
observation sec-9/the capsule runs with `no_new_privs` set=Read { value: "1", caveat: Some("the trusted side already reads `no_new_privs` set, so this run cannot tell whether the backend set the bit or the capsule inherited it — the value is right and its provenance is not established here (`EVD-014` `A6`)") }
```

## What is invariant and what is not

**Invariant.** The nineteen row-to-verdict mappings, the four auxiliary claims,
and the two unrowed readings. Every one is `Proven` / `Passed` / `Read`, and
every one must reproduce after the split. `RV-352`'s baseline claim is exactly
this, re-measured.

**Not invariant, by design.** The `outcome=` line (`DEC-191` stops the
collapse), the exit constants (`DEC-194` renames them), and the row *key
spellings* — `Property(ClosedDescriptorSet)` becomes an assurance-profile id
under `DEC-198`, and `Property(DeniedCanonicalStateAndCredentials)` becomes the
floor key. The rendering is `{:?}` over the enum, so the key spelling moves
whenever `RowId` does.

The post-split comparison is therefore not a diff of this text. It is: same
nineteen rows, same verdict each, under a documented key translation — with the
outcome line and exit code enumerated in advance as permitted to differ.
