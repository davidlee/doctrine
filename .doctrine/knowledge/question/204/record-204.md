# QUE-204: Capsule build-input provisioning

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is never
     structurally parsed (the storage rule). -->

**How should a capsule obtain build inputs it cannot get from git objects?**

Raised from SL-241 PHASE-05 (F-P05-16), where it stopped being hypothetical.

## The shape of the problem

A capsule provisions by **git-object transfer at an OID**, and a fixture by
`git clone`. Both carry **tracked content only, by construction**. Any project
whose build depends on state that is gitignored — built assets, vendored
dependencies, generated code — therefore cannot be built inside a capsule at
all, and its declared `verify:` cannot pass there.

This is not exotic. Doctrine itself is such a project: `web/map/dist/` is a
RustEmbed `#[folder]` root *and* gitignored, so a clone of this repo fails to
compile with `#[derive(RustEmbed)] folder ... does not exist` and an `E0599`
cascade at every `Assets::get` site.

Doctrine already solved the same problem once, elsewhere: `.worktreeinclude`
exists precisely because `worktree fork` hit this wall, and `web/map/dist/**`
is one of its two entries. The interpretation-surface declaration (DEC-099) is
described as the dual of that file, but it is the dual for **interpretation
hazard** — what must not be trusted. There is no dual for **provisioning
need** — what must be carried. That absence is the question.

## Options, none yet chosen

1. **Commit the inputs into the fixture/base.** Simplest; reaches the capsule
   through objects with no new mechanism. Costs: B stops being a faithful commit
   of the source repo, and build artefacts enter a git history.
2. **Teach provisioning a copy-in list** — the missing dual of
   `.worktreeinclude`, recursive-copy rather than commit. Faithful to how
   `worktree fork` already behaves, and keeps artefacts out of git. Costs a new
   mechanism in the provisioning path.
3. **Build on site inside the capsule.** Chosen provisionally in PHASE-05
   (D-P05-7) and *measured to be a poor default*: it makes every cell reaching
   that stage depend on a third-party registry, and both mechanisms were
   observed stalling in `bun install` while the host reached
   `registry.npmjs.org` in 0.17s. A cell that can fail for reasons outside the
   capsule model breaks the standard F-P05-10 set — *"a heavy refusal can be
   believed"*.
4. **Allowlisted network egress.** Acceptable in principle (operator,
   2026-08-02) — the objection is not to egress but to *unbounded* egress. But
   the cost is **building the mechanism, not configuring one**. On this host
   network is granted by *namespace omission*: the jail's `network` combinator
   simply deletes `--unshare-net`, so the sandbox sits in the host network
   namespace with the same routing table and firewall position as the operator.
   There is no egress policy, no seccomp, and `jail.nix` has **no combinator**
   for a middle option. Realising this means either `--unshare-net` plus a
   slirp4netns/pasta userspace stack or a veth net namespace with an nftables
   allowlist — or, for *domain-level* allowlisting that the namespace approach
   cannot express at all, `--unshare-net` plus `HTTPS_PROXY` pointed at a
   host-side filtering proxy with its MITM CA bound into `/etc/ssl`.

   **DNS is not the obstacle, and was measured so.** The capsule's `/etc` holds
   only `resolv.conf` — no `nsswitch.conf`, no `hosts`, no `ssl` — yet lookups
   inside resolve in **~0.5ms** against the host's 36–129ms, because the
   systemd-resolved stub at `127.0.0.53` is reachable over shared loopback and
   answers from cache, and curl's CA bundle comes from the store. A plausible
   "the capsule's resolver is crippled" explanation for slow installs is
   therefore **refuted**; the cost is bun fetching and extracting a few hundred
   packages with no warm cache.

## The direction this is expected to grow toward

Operator, 2026-08-02: provisioning **from capsules at a known HEAD**, taking the
build green-light there too — so that N concurrent phases off the same base
**pay setup once** rather than N times. That reframes the question: build inputs
stop being something each capsule fetches and become something a shared warm
base *already holds*, with per-phase capsules derived from it.

The argument against adopting it now is only sequencing — it is complexity worth
deferring to finish SL-241 rather than a disagreement about direction.

## Bearing on other records

- Feeds [[QUE-200]] (M-A vs M-B): whatever the answer, it applies to both
  ingestion mechanisms, so it does not discriminate between them — but the
  *cost* of provisioning is a comparison input.
- Adjacent to [[QUE-202]] (how the capsule model admits a second result).
- The tension named in SL-241 F-P05-17 is the same one seen from the other
  side: EX-1's visibility floor withholds ambient structure that the project's
  own build and suite depend on. Provisioning and fidelity are one question
  asked twice.
