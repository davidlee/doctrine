# SL-094: wheel event cross-browser normalization

Wheel event deltaY normalization: use deltaMode to distinguish pixels (0) from lines (1). Chrome emits pixels, Firefox emits lines. Normalize: delta = e.deltaMode === 1 ? e.deltaY * 16 : e.deltaY. Cap |delta| to 40 for trackpad inertial spikes. Scale factor 0.002. Use { passive: false } on wheel listener.
