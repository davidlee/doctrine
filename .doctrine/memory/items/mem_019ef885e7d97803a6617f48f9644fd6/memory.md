# Lower-module → CLI back-edge closes a layering cycle; inject a fn-pointer instead

A lower module calling back into the CLI layer can close a same-tier import cycle and ratchet the ADR-001 Command tangle_baseline; inject the renderer as a fn-pointer from the layer that already depends downward.
