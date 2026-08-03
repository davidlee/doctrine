# IMP-395: Skills hand collaborators their exact invocation

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## The gap

Where a skill spawns or briefs another agent, it names *what* to do and leaves
*how to read* to improvisation. The spawning agent then reconstructs the command
from memory — which is where guessed flags, wrong binaries and stale command
shapes enter.

Two failure modes, both live in this repo:

- **The command shape is guessed.** The standing guardrail is "don't guess ids /
  command shapes / paths — ask the CLI", and it exists because agents do guess.
  A brief that hands over the literal invocation removes the opportunity.
- **The binary is wrong.** `doctrine` on `PATH` is not the same binary as
  `./target/debug/doctrine`, and in a worktree it is reliably the *wrong* one —
  the project rule is to run corpus-inspecting verbs from the coord tree's build,
  and `DOCTRINE_BIN` exists for exactly this. A collaborator spawned without an
  explicit executable path will use whatever it finds, silently, and report on
  stale state.

## Shape of the want

Wherever a skill briefs a collaborator — the reviewer handoff, the research
scripts, dispatch worker briefs — the brief carries:

- the **exact command line** to read the artefact under review, resolved, not
  described;
- the **executable path** where it matters, or the env var that fixes it;
- the small set of read verbs the collaborator may need next, likewise literal;
- the pointer to its orientation asset (`IMP-394`).

Once `IMP-393` lands, the design case becomes concrete: the reviewer brief cites
the design render verb, resolved against the right binary, rather than telling
the reviewer that a design exists somewhere.

Worth deciding whether this is per-skill authoring (each brief writes its own)
or one shared brief-builder the skills call — the second is the DRY answer and
the one that keeps the invocation correct in one place, but it needs a home.

## Origin

Raised during `SL-244`'s design run, alongside `IMP-393` (the reader-facing
design render) and `IMP-394` (collaborator orientation). This is the **handoff**
piece: the artefact and the orientation are useless if the collaborator is never
told how to reach them.
