# IMP-181: Replace wildcard agent-def install fallbacks with explicit match arms for N>2 agents

Captured from [[mem_019ed5e7827177f1abb1db081a78ad36]].

Two wildcard fallbacks (`_ → pi`) in `install_agent_def()` work for the current
two-agent world (Claude + pi) but adding a third agent type will require
replacing them with explicit match arms. Not a defect — a natural extension
hazard. Whether there are more wildcards beyond the two identified is unverified.
