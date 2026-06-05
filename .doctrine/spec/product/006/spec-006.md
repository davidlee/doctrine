# PRD-006: Install

## 1. Intent

Doctrine governs a project from a set of working files that live inside that
project — the scaffolding the rest of the system reads, writes, and reconciles
against. Before any of that machinery can run, those files have to *exist*, in the
right place, in a project that knew nothing about Doctrine a moment ago. Asking an
operator to place them by hand is fragile: files land in the wrong directory, the
ignore boundary is forgotten, and a second person sets it up subtly differently.

The need is to **bring a project under Doctrine's governance with one deliberate,
reviewable, repeatable act** — and to keep doing so safely as Doctrine evolves.
The value is twofold. First, adoption is trivial and uniform: any project, from
any working directory inside it, ends up with the same correct layout without the
operator having to know where things belong. Second, the act is *safe to repeat*:
when Doctrine grows new files, re-running provisioning tops up what is missing
without ever disturbing the edits the operator has since made to what was already
there. The operator can see the full effect before committing to it, so trust is
never asked for blind.

## 2. Scope

In scope:

- Provisioning Doctrine's working files into a target project — creating the
  directories it needs and writing the files it ships.
- Determining where the target project's root is when the operator has not said,
  so provisioning is correct from anywhere inside the project.
- Presenting the complete set of intended changes for review before any of them
  are applied, and offering a way to preview without applying.
- Maintaining the project's version-control ignore boundary so Doctrine's
  disposable, private, and derived files are not committed by accident.
- Repeated provisioning that converges: adding what is absent without redoing or
  undoing what is present.

Out of scope:

- What the provisioned files mean or do once in place — that belongs to the
  capabilities those files serve, not to provisioning.
- Upgrading, migrating, or reconciling operator-modified files toward a newer
  shipped version — provisioning adds the missing, it does not merge the changed.
- Removing, relocating, or de-provisioning Doctrine from a project.
- Choosing or enforcing a particular version-control system; provisioning is
  VCS-agnostic and only touches the ignore boundary as a plain file.

Boundary: provisioning owns *getting Doctrine's files correctly into a project and
keeping that placement convergent and non-destructive*. It does not own the
behaviour of those files, nor their evolution once an operator has touched them.

## 3. Principles

- **Preview precedes change.** Provisioning never alters a project without first
  being able to state, in full, what it would do; applying blind is not a mode.
- **Existing files are sovereign.** A file already present is never overwritten.
  Provisioning is additive; the operator's edits outrank the shipped default.
- **Repetition converges, it does not accumulate.** Running provisioning again
  reaches the same end state without duplicating, doubling, or undoing prior work.
- **The root is discovered, not assumed.** Correct placement depends on finding the
  project root deterministically; an undiscoverable root is a stop condition, not a
  guess.
- **The ignore boundary is part of correct placement.** Provisioning is not done
  when files are written — it is done when Doctrine's disposable and private files
  are also kept out of version control.

## 4. Requirements

The functional and quality requirements this capability must satisfy are recorded
as requirement entities and appear under the synthesized Requirements section
below. This section carries only the constraints and invariants that bound every
valid implementation.

Constraints:

- Provisioning must be VCS-agnostic: it may detect a project root by any of several
  marker conventions and must not depend on one specific version-control system.
- The set of provisioned files must travel with the distributed tool itself, so a
  project can be brought under governance with no external source to fetch from.
- Configuration of what is provisioned (target location, directories, ignore
  entries, root markers) must have working defaults so the common case needs no
  setup, and every configurable section must be optional.

Invariants:

- A file that already exists at its destination is never overwritten.
- Re-running provisioning on an already-provisioned project changes nothing that is
  already correct — no file is rewritten, no directory re-made, no ignore entry
  duplicated.
- No change is ever applied to a project until its full plan has been made
  available for review (or the operator has explicitly waived the prompt).
- Provisioning either resolves a project root or refuses to act; it never writes to
  an arbitrary or ambiguous location.

## 5. Success Measures

- An operator can bring a fresh project under Doctrine governance in a single
  command, from any directory inside that project, and obtain the same correct
  layout every time.
- Before anything is written, the operator can read the complete list of
  directories to create, files to install, files to skip, and ignore entries to
  append, and can obtain that preview without applying it.
- Running provisioning a second (or tenth) time on the same project reports no new
  changes and leaves every existing file byte-for-byte untouched.
- After provisioning, Doctrine's disposable, private, and derived files are absent
  from version control, and no ignore entry appears more than once.
- A project with no Doctrine knowledge and no special configuration is provisioned
  correctly on defaults alone.

## 6. Behaviour

Primary flow — provision a project: an operator invokes provisioning; the system
resolves the project root, computes the full set of intended actions, presents that
plan, and on confirmation applies it — creating directories, writing absent files,
and appending missing ignore entries — then reports what it did.

Preview flow — plan without applying: an operator asks for the plan only; the
system resolves the root, computes and presents the complete set of intended
actions, and exits having changed nothing.

Unattended flow — apply without prompting: an operator who has reviewed or trusts
the plan waives the confirmation; the system computes and applies the plan in one
step, still surfacing what it did.

Root-resolution flow: when the operator names the project root explicitly, it is
used directly; otherwise the system discovers the root by searching outward from
the current location for a recognised project marker. If it reaches the top of the
filesystem without finding one, it refuses to act rather than guessing.

Convergence guard: on a project that is already provisioned, each intended file
that already exists is reported as skipped and left untouched, each needed
directory that already exists is a no-op, and each ignore entry already present is
not re-added — so a repeat run converges to no effective change.

Edge cases and boundaries: an absent ignore file is created when an entry must be
added; a partially provisioned project is completed without disturbing the part
already in place; a file the operator has edited since a prior run is preserved as
the operator's, never reconciled toward the shipped version.

## 7. Verification

Verification confirms that provisioning places Doctrine's files correctly, that it
is safe to preview and safe to repeat, and that it respects the operator's
sovereignty over existing files — without binding the spec to a particular
implementation.

Root resolution is proven by exercising it directly: an explicit root is honoured
verbatim, a root is discovered by walking outward from a nested location to the
nearest recognised marker, and the absence of any marker up to the filesystem top
is proven to refuse rather than write. The non-destructive invariant is proven by
provisioning over pre-existing files and confirming they are reported skipped and
left byte-for-byte unchanged. Convergence is proven by provisioning twice and
confirming the second run plans and effects no change — including that no ignore
entry is duplicated. The preview obligation is proven by confirming the plan-only
path computes and surfaces the full action set while leaving the project untouched,
and that the applying paths report the actions they took. Default behaviour is
proven by provisioning a project with no configuration present and confirming the
shipped defaults yield a correct layout.

Where a check must reference a specific obligation, cite the durable requirement
entity (REQ-NNN), never a mobile membership label. Coverage of the functional and
quality requirements is tracked against those entities, not duplicated here.

## 8. Open Questions

- When Doctrine ships a newer version of a file an operator has since edited,
  provisioning currently preserves the operator's copy and never surfaces the
  divergence. Should provisioning be able to *detect and report* such drift (short
  of overwriting it)? This blocks any future upgrade or migration capability and
  the success measures it would need.
- Root resolution can match more than one marker convention; in a nested or
  multi-project tree the nearest marker may not be the intended root. What is the
  acceptable disambiguation posture when several plausible roots are in scope? This
  blocks confident provisioning inside monorepos and embedded sub-projects.
