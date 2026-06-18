# SL-094: mousedown/mouseup pan pattern with {once:true}

Drag-to-pan: mousedown on container records origin, wires mousemove+{once:true} mouseup on document. {once:true} prevents stale listeners. Toggle grabbing CSS class. Gate: e.target.closest('.doctrine-node') to avoid intercepting node clicks.
