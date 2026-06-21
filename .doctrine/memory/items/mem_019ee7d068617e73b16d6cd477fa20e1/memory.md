# Shared facet projection: EntityFacets { estimate, value, risk, tags } — consume from ScannedEntity scan, feed both SL-132 (display) and SL-133 (scoring) to avoid parallel parsing

SL-132 and SL-133 both need estimate/value/risk/tag data from authored facets. Before either slice grows its own parser, establish a shared EntityFacets projection consumed by format_show, build_priority_graph, and explain.
