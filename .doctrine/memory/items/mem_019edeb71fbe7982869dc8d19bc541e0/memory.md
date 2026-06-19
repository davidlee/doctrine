# SL-109 print_review location pattern: in leaf module for test access

print_review() in review.rs (not main.rs) so golden tests call it. Returns String not writes stdout. Format-only fields (#[serde(skip)]) keep JSON clean for MCP. SL-109.
