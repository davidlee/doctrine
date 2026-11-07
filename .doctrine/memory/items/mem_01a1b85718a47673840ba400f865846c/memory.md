In a `reviewing` design run, `doctrine design apply` treats a top-level
`acceptance` block as the user's `design-accepted` act — even when the
submission only declares section attestations (`att-N attests sec-N`) and has
no `checkpoint_act`. Observed SL-287, run revision 43: the apply output listed
`act_recorded cpa-design-accepted act=design-accepted`.

There is no verb to retract a recorded act, and a later discharge-only
submission did not make it stale (`design show --format json` still reported
`acts[].current: true`).

**How to apply:** record section attestations without a top-level
`acceptance` unless the user has actually accepted the design (check the
contract for whether attestations need it), or record them in the same
submission as the real lock. If it happens anyway, tell the user at once: the
act stays live until the design or the disposition changes.
