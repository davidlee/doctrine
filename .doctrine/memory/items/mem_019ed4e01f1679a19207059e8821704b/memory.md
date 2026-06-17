# Memory write path labels are free-form; RELATION_RULES vocabulary is closed to memory edges

Memory [[relation]] rows use raw label strings (CatalogEdgeLabel::Raw). The link/unlink write path for memory relations therefore forks at the label type from the numbered-entity path — RelationLabel is vocabulary-bound; memory labels are user-chosen free text. Any future unified-label work must reconcile these two write paths.
