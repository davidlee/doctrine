# bootstap doctrine:

> just --list  # common tasks
> just check   # pre-commit gate

.doctrine/slice/nnn/
- slice-nnn.toml - metadata, relations, progress
- slice-nnn.md   - scope document
- design.md      - canonical technical design
- handover.md    - disposable agent context. gitignored
- notes.md       - durable notes from implementation
- audit.md       - verification check, code review, drift findings

doc/*    - evergreen, authoritative specifications. not yet structurally supported by doctrine.
install/ - sources, copied to .doctrine by installer. plugins / skills handled special.
src/     - rust code 

## core process

> doctrine slice new    # define scope
> doctrine slice design # begin design

design iteration - interview, present decisions, tradeoffs, alternatives; refine open questions
adversarially review; repeat

> doctrine slice plan    # set up implementation plan

plan implementation of design against existing surface

> doctrine slice phases  # set up implementation plan

plan each phase in detail just prior to execution

... (implementation goes here)

/code-review & audit against design -> audit.md 

todo: update this as tooling surface expands

## rules

- no code without an approved plan
- frequent conventional commits
- ask, don't infer
- correctness comes first and last

## environment

nixos; bubblewrap jails (mounted into /workspace/*).
