# Parallel agent-detection resolvers are a known divergence risk

detect_agents() (install.rs), resolve_agents() (skills.rs), and boot::resolve_harnesses() all detect agents differently. RV-194 caught divergence where detect_agents gained .pi/.agents support but resolve_agents did not. Any agent-detection change must update all three.
