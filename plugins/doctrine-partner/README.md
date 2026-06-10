# doctrine-partner

The two collaboration skills from the `doctrine` plugin — `pair` (calibrated
adversarial pair programming) and `walkthrough` (guided, expertise-aware
code/artifact comprehension and critique) — packaged on their own, for projects
that want a partner-in-the-loop posture without the full process skill set.

Both skills are agentic — they need no CLI; the `doctrine` binary is only a
dependency of the broader process skills, not these two.

## Install one, not both

In this repository the subset's skills are **symlinks** into the canonical
source — there is one source of truth, so they cannot drift:

- **Canonical source:** `plugins/doctrine/skills/{pair,walkthrough}/`
- `plugins/doctrine-partner/skills/<id>` → `../../doctrine/skills/<id>` (symlink)

Claude Code plugins are self-contained at distribution time, so each symlink is
followed and ships as a real copy in the published artifact. That means a
consumer who installs **both** `doctrine` and `doctrine-partner` gets two skills
of each name, invokable only by their namespaced form. Install `doctrine` **or**
`doctrine-partner`, not both.
