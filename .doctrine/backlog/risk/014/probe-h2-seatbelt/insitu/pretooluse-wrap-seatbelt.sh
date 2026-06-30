#!/usr/bin/env bash
# PreToolUse(Bash) WRAPPER — H2 pass 2 (M1-sub, the in-situ nesting gate).
#
# macOS analog of probe-h1/pretooluse-wrap.sh: opaquely rewrites a worktree-
# subagent's Bash command via `updatedInput` to run inside a nested Seatbelt
# write-floor (`sandbox-exec` + generated `.sb`). Reuses the PROVEN profile +
# argv logic from seatbelt-jail.sh (sourced — NO reimplementation; DRY) so the
# wrapper is a faithful in-situ driver of the pass-1 builder, not a fork of it.
#
# This is the leg that pass 1 could NOT exercise: pass 1 drove seatbelt-jail.sh
# from the orchestrator Seatbelt context (M1-orch). Here the SAME builder runs
# from inside a real `isolation:worktree` subagent, where Claude Code's own
# native Seatbelt is active — the true nesting test (design §5.5 ASSUMPTION
# M1-sub; OQ-mac1). It also exercises the live `updatedInput` honour-on-macOS
# question (the H1b analog, proven on Linux only).
#
# FAIL-CLOSED (binding facts mem.pattern.dispatch.claude-worktree-subagent-bwrap-
# confinement): pass through ⟺ agent_id ABSENT (orchestrator/interactive).
# agent_id PRESENT ⇒ cwd MUST resolve to a recognised git worktree distinct from
# the main checkout, else DENY. Never frame as "jail when in a worktree, else
# pass through" — that fails OPEN for an isolation:none subagent (agent_id
# present, cwd == repo root).
#
# CONTROL (F-C, the subagent-context confound): while $PROBE_BASE/DISABLE_WRAP
# exists, a validated worktree-subagent's Bash passes THROUGH un-sandboxed, so a
# bare-write control can run and reveal whether a denial is the permission gate
# or Seatbelt. Remove the sentinel to re-arm the floor. The auto-vs-yolo
# comparison runs the battery with the sentinel both present (control) and
# absent (armed) under each mode.
set -u

# --- where things live ------------------------------------------------------
# PROBE_BASE is the gitignored scratch root; it carries the control sentinel,
# the per-invocation profile, and the wrapper log. Resolved from the hook env
# (exported in settings.local.json). Fail closed if unset.
PROBE_BASE="${PROBE_BASE:-}"
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
JAIL_LIB="$HERE/../seatbelt-jail.sh"           # the proven pass-1 builder
LOG="${PROBE_BASE:+$PROBE_BASE/wrapper.log}"

STDIN="$(cat)"

log() {
  [ -n "${LOG:-}" ] || return 0
  printf '%s\t%s\n' "$(date -u +%s.%N)" "$*" >> "$LOG" 2>&1
}

emit_allow() {  # $1 = rewritten command
  # updatedInput REPLACES the entire tool_input (docs/claude/hooks.md §PreToolUse
  # decision control), so merge over the ORIGINAL tool_input — override only
  # `command`, preserve any other fields (timeout, run_in_background, …).
  printf '%s' "$STDIN" | jq -c --arg c "$1" \
    '{hookSpecificOutput:{hookEventName:"PreToolUse",permissionDecision:"allow",permissionDecisionReason:"seatbelt-jail: wrapped",updatedInput:((.tool_input // {}) + {command:$c})}}'
  exit 0
}

deny() {  # $1 = reason
  log "DENY agent_id=${agent_id:-?} cwd=${cwd:-?} reason=$1 cmd=${cmd:-?}"
  jq -cn --arg r "$1" \
    '{hookSpecificOutput:{hookEventName:"PreToolUse",permissionDecision:"deny",permissionDecisionReason:("seatbelt-jail: "+$r)}}'
  exit 0
}

pass_through() {  # $1 = reason (logged only)
  log "PASS-THROUGH $1 agent_id=${agent_id:-} cwd=${cwd:-?}"
  exit 0
}

agent_id=$(printf '%s' "$STDIN" | jq -r '.agent_id // empty' 2>/dev/null)
cwd=$(printf '%s'      "$STDIN" | jq -r '.cwd // empty' 2>/dev/null)
cmd=$(printf '%s'      "$STDIN" | jq -r '.tool_input.command // empty' 2>/dev/null)
# permission_mode is in stdin (docs/claude/hooks.md) — log it so the auto-vs-yolo
# matrix is read off the wrapper log directly, not inferred.
pmode=$(printf '%s'    "$STDIN" | jq -r '.permission_mode // empty' 2>/dev/null)

# --- binding: orchestrator/interactive → never touch (fail OPEN only here) ---
if [ -z "$agent_id" ]; then
  pass_through "orchestrator"
fi

# --- subagent path: fail CLOSED on every ambiguity --------------------------
[ -n "$PROBE_BASE" ] || deny "PROBE_BASE-unset"   # can't build a profile dir → refuse
[ -n "$cmd" ]        || deny "empty-command"
[ -n "$cwd" ]        || deny "no-cwd"
[ -d "$cwd" ]        || deny "cwd-dir-missing:$cwd"

# cwd must be a git worktree DISTINCT from the main checkout. We do NOT hardcode
# the macOS worktree path (unknown until pass 2 observes it) — we ask git. A
# real linked worktree has git-dir under <main>/.git/worktrees/<name>, so its
# --git-common-dir resolves to the main repo's .git while its toplevel differs
# from the main checkout. Both probed via realpath to defeat the /tmp alias.
wt_top=$(git -C "$cwd" rev-parse --show-toplevel 2>/dev/null)
[ -n "$wt_top" ] || deny "cwd-not-a-git-worktree:$cwd"
wt_top=$(realpath "$wt_top" 2>/dev/null) || deny "wt-toplevel-unresolved:$cwd"

common=$(git -C "$cwd" rev-parse --git-common-dir 2>/dev/null)
gitdir=$(git -C "$cwd" rev-parse --git-dir 2>/dev/null)
# normalise to absolute — git prints these RELATIVE TO $cwd when not absolute.
case "$common" in /*) : ;; *) common="$cwd/$common" ;; esac
case "$gitdir" in /*) : ;; *) gitdir="$cwd/$gitdir" ;; esac
common=$(realpath "$common" 2>/dev/null) || deny "git-common-dir-unresolved"
gitdir=$(realpath "$gitdir" 2>/dev/null) || deny "git-dir-unresolved"
main_top=$(realpath "$(dirname "$common")" 2>/dev/null)   # <main>/.git → <main>

# A linked worktree: git-dir != common-dir AND toplevel != main checkout. If
# this is the main checkout itself (gitdir == common, top == main_top), it is
# NOT a worktree subagent we may confine to cwd → deny (the isolation:none /
# repo-root case the binding rule warns about).
if [ "$gitdir" = "$common" ] || [ "$wt_top" = "$main_top" ]; then
  deny "cwd-is-main-checkout-not-worktree:$wt_top"
fi

# --- F-C control: un-sandboxed pass-through while the sentinel exists --------
if [ -f "$PROBE_BASE/DISABLE_WRAP" ]; then
  log "CONTROL-BYPASS un-sandboxed mode=${pmode:-?} agent_id=$agent_id cwd=$cwd wt_top=$wt_top cmd=$cmd"
  exit 0   # let the bare command run; the F-C control measures the gate, not Seatbelt
fi

# --- preflight the floor toolchain (fail closed if missing) -----------------
command -v sandbox-exec >/dev/null 2>&1 || deny "sandbox-exec-unavailable"
[ -r "$JAIL_LIB" ]                      || deny "seatbelt-jail-lib-missing:$JAIL_LIB"

# Source the PROVEN profile + builder. We reuse emit_profile + the realpath/-D
# discipline; we only need the *command string* (the hook rewrites, it does not
# exec), so we call a thin string-builder that mirrors seatbelt_run's argv but
# prints instead of running.
# shellcheck source=../seatbelt-jail.sh
source "$JAIL_LIB" || deny "seatbelt-jail-source-failed"

# Build the sandbox-exec command string for `updatedInput`. network=true (egress
# is a non-goal here — design §7 D-mac4), deny_private_tmp=1 (D-mac3). Mirrors
# seatbelt_run but emits an argv-quoted string the harness will run verbatim.
build_wrapped_cmd() {  # $1 = worktree (realpath'd already as wt_top), $2 = raw cmd
  local rwt="$1" raw="$2" rtmp rptmp prof
  mkdir -p "$rwt/.tmp" || { echo ""; return 1; }
  rtmp="$(realpath "$rwt/.tmp")" || { echo ""; return 1; }
  rptmp="$(realpath /tmp)"       || { echo ""; return 1; }
  prof="$PROBE_BASE/floor.$agent_id.sb"

  # emit_profile <network> <deny_ptmp> <n_extra_rw>  (no extra_rw in the probe)
  emit_profile true 1 0 > "$prof" || { echo ""; return 1; }

  # opaque base64 body (single-quote-safe charset); decode+exec inside the floor;
  # TMPDIR redirected into the rw worktree (D-mac3). children inherit (M3).
  local b64; b64="$(printf '%s' "$raw" | base64 | tr -d '\n')"
  printf "sandbox-exec -D 'WT=%s' -D 'TMP=%s' -D 'PTMP=%s' -f '%s' -- /bin/bash -c \"export TMPDIR='%s'; printf %%s '%s' | base64 -d | /bin/bash\"" \
    "$rwt" "$rtmp" "$rptmp" "$prof" "$rtmp" "$b64"
}

wrapped="$(build_wrapped_cmd "$wt_top" "$cmd")"
[ -n "$wrapped" ] || deny "profile-build-failed"

log "WRAP mode=${pmode:-?} agent_id=$agent_id cwd=$cwd wt_top=$wt_top cmd=$cmd"
emit_allow "$wrapped"
