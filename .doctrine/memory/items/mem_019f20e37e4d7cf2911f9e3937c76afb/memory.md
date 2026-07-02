# parse_resolvable_ref must detect cross-kind id ambiguity

When bare ids are resolved by scanning all entity kinds (parse_resolvable_ref), multiple entities can share the same numeric id across different kinds. The scan must detect ambiguity and reject — first-match-wins is silently wrong. SL-188 shipped without this gate (see RV-212 F-1).
