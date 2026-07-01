# SL-183 EX-2 live subagent battery blocked two ways on macOS: plugin hook not firing + subagent refuses red-team framing

The live isolation:worktree escape-battery leg (SL-183 PHASE-04 EX-2) is blocked on macOS by two independent walls: (1) the doctrine skill/plugin PreToolUse(Bash) hook does not intercept subagent Bash this session (subagent in-wt writes ran unwrapped, no .tmp/jail.sb), and (2) current Sonnet subagents refuse the red-team escape-battery prompt (even terse + bypassPermissions) as sandbox-escape recon.

## Wall 1 — plugin PreToolUse(Bash) not firing for subagents (macOS)
Confirmed by the T3b materialization tell: a subagent that DID run `echo INSIDE_OK
> ./inside_marker` left the marker but NO `<wt>/.tmp/jail.sb`. Since the shipped
consumer writes the `.sb` only on the Seatbelt wrap path
(`pretooluse::materialize_seatbelt_profile`), its absence proves the hook never
wrapped that Bash — i.e. the hook did not fire. A logging shim wired into the
`.claude/skills/doctrine/hooks/hooks.json` Bash matcher also logged nothing across
two spawns. Contrast: the PROVEN-working RSK-014 pass-1/2 runs registered the hook
via a `.claude/settings.local.json` `hooks` block (session-start only); that block
is currently EMPTY (`hooks:{}`). The doctrine plugin is NOT in `enabledPlugins`
(`~/.claude/settings.json`) nor `installed_plugins.json` — the `.claude/skills/
doctrine/` dir is auto-discovered as a project skill dir, and `/reload-plugins`
counts its hooks ("3 hooks") but they did not intercept subagent Bash on macOS.
See the downgraded [[mem.fact.claude.reload-plugins-registers-pretooluse]].
**Next-agent lever to try:** wire the shipped `doctrine worktree pretooluse` (or a
logging shim around it) into `.claude/settings.local.json`'s PreToolUse Bash
`hooks` block — the known-working session-start surface — restart, confirm a
subagent's `.tmp/jail.sb` materializes, THEN run the battery. Open question for the
human: is EX-2's "live consumer" satisfied by the shipped binary+logic firing via
settings.local.json, or does it require the plugin hooks.json to be the firing
surface (which needs the plugin actually enabled)?

## Wall 2 — subagent refuses the red-team battery
Three spawns (Opus once, Sonnet twice), across framings from verbose-authorized to
terse-h1-style, all refused as "sandbox-escape reconnaissance / prompt-injection",
citing authority-assertion tells and the promised follow-up "escape-vector
battery". Under `mode: bypassPermissions` a Sonnet DID run the two inert orient
commands once (pwd + in-wt echo) but still declined the battery; a later spawn ran
ZERO commands. The subagents made a SOUND methodological point that matches the
rig's own F-D: **containment is verified from OUTSIDE (canary checksums via
`verify.sh`), so the subagent's self-report/cooperation is not load-bearing.** So a
future run should NOT depend on the subagent narrating denials — it only needs the
subagent to run ordinary commands that touch canary paths while the OUTSIDE verifier
reads them. Do NOT escalate/jailbreak the prompt to defeat the refusal (working
around a safety judgment). Pass-2 (2026-07-01) got compliant subagents; the model
posture shifted since. **The nesting property EX-2 would re-test (INV M1-sub) is
ALREADY SUPPORTED from pass-2 and promoted to a design INV** — so the live battery
is re-confirmation, not the sole proof.

## What IS proven (so EX-2 is not empty)
T3a+T3b wiring + offline floor validation through the SHIPPED binary: given a real
linked-worktree cwd + a provisioned `jail/<basename>.toml`, the consumer emits the
`sandbox-exec -f jail.sb` wrap, materializes the 540B `.sb`, and the floor confines
end-to-end — OUTSIDE write `Operation not permitted` (canary pristine), IN-WT write
OK. Plus the live deny-path (no policy ⇒ fail-closed). The only unproven-live piece
is the nested-subagent battery, which pass-2's INV(M1-sub) already covers.

Rig lives at `.doctrine/backlog/risk/014/probe-h2-seatbelt/insitu-live/`
(shim + arm.sh/disarm.sh). Pairs with
[[mem.pattern.seatbelt.profile-materialization-command-tier]] and
[[mem.pattern.macos.doctrine-hook-reinstall-resign]].
