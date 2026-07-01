#!/usr/bin/env bash
# ── Live-consumer PreToolUse observation shim (SL-183 PHASE-04 / EX-2) ──────────
#
# The SHIPPED consumer is `doctrine worktree pretooluse`. It emits its decision as
# JSON on stdout and writes NO log — so pass-2's `wrapper.log` WRAP-arbiter (S1)
# has no source with the real binary. This shim ONLY OBSERVES: it runs the real
# consumer with the same stdin, tees the request + the consumer's verbatim decision
# to a log (classified WRAP / DENY / EMPTY, with the harness permission_mode), then
# emits the decision UNCHANGED and preserves the consumer's exit code.
#
# It does NOT alter the decision — the decision under test is 100% the shipped
# consumer's. The shim is the live analog of pass-2's wrapper.log, wrapped around
# the real binary instead of a reimplemented shell wrapper.
#
# Wiring: the skill hooks.json Bash matcher is temporarily repointed at THIS shim
# (see arm.sh). CLAUDE_PROJECT_DIR is exported by the harness for the hook.
set -u

REAL_BIN="${DOCTRINE_HOOK_BIN:-/Users/davidlee/.cargo/bin/doctrine}"
LOG="${PROBE_BASE:?set PROBE_BASE}/consumer.log"
mkdir -p "$(dirname "$LOG")"

# Capture the tool-call request (stdin) once; feed it to the real consumer.
REQ="$(cat)"

# Run the SHIPPED consumer verbatim. Its stdout IS the decision the harness acts on.
DECISION="$(printf '%s' "$REQ" | "$REAL_BIN" worktree pretooluse)"
RC=$?

# Classify for the log without touching the decision. jq is not assumed present;
# grep the emitted JSON + the request's permission_mode/tool_name/agent_id.
mode="$(printf '%s' "$REQ"      | sed -n 's/.*"permission_mode"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' | head -1)"
tool="$(printf '%s' "$REQ"      | sed -n 's/.*"tool_name"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' | head -1)"
agent="$(printf '%s' "$REQ"     | sed -n 's/.*"agent_id"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' | head -1)"
cwd="$(printf '%s' "$REQ"       | sed -n 's/.*"cwd"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' | head -1)"

if   printf '%s' "$DECISION" | grep -q '"permissionDecision"[[:space:]]*:[[:space:]]*"allow"'; then verdict=WRAP
elif printf '%s' "$DECISION" | grep -q '"permissionDecision"[[:space:]]*:[[:space:]]*"deny"';  then verdict=DENY
elif [ -z "$DECISION" ]; then verdict=EMPTY  # orchestrator fast-path (no agent_id) OR non-emitting
else verdict=OTHER
fi

reason="$(printf '%s' "$DECISION" | sed -n 's/.*"permissionDecisionReason"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' | head -1)"
# Does the emitted updatedInput.command wrap into sandbox-exec? (the macOS wrap marker)
wrapmark=""
printf '%s' "$DECISION" | grep -q 'sandbox-exec' && wrapmark=" sandbox-exec=yes"

printf '%s verdict=%s mode=%s tool=%s agent=%s cwd=%s rc=%s reason=%q%s\n' \
  "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "$verdict" "${mode:-?}" "${tool:-?}" "${agent:-?}" "${cwd:-?}" "$RC" "${reason:-}" "$wrapmark" >> "$LOG"

# Emit the consumer's decision UNCHANGED, preserve its exit code.
printf '%s' "$DECISION"
exit $RC
