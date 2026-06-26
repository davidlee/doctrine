# IMP-179: Replace string-parsing error-mapping with structured error enums

Captured from [[mem.pattern.error-mapping-by-string-parsing]].

Error-mapping-by-string-parsing is fragile; the codebase should migrate to
structured error enums instead. Scope and affected call sites are unverified.
