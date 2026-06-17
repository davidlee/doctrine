# Agent-def generalization: wildcard fallback fragile for N>2 agents

When generalizing install_agent_def() from Claude-only to Claude+pi, two wildcard fallbacks (_ → pi) were used: in install_agents_for() for embed asset selection, and in install_agent_def() for link_dir selection. This works correctly for the current two-agent world but adding a third agent type will require replacing the wildcards with explicit match arms. Not a defect — a natural extension point.
