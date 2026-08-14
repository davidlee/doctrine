# QUE-217: Which casual capsule backends should complement hardened microVMs

## Question

Which lower-barrier capsule mechanisms should Doctrine qualify and support
alongside a hardened microVM profile?

## Why this remains open

Oubliette demonstrates a strong Linux boundary, but its host integration is not
casual: the current formulation uses a NixOS module and systemd services, and
some configuration changes require a host rebuild. Its measured transaction
startup cost therefore does not capture installation, portability, or host
configuration cost.

The credible option set is now:

- any-Linux bubblewrap as a casual local profile;
- macOS Seatbelt as a casual local profile with a shared proxy/Git perimeter;
- restricted QEMU/UTM Linux VM as a hardened macOS profile;
- another portable microVM formulation that preserves the authority floor with
  less host coupling.

## Decision criteria

The answer should compare host prerequisites, configuration/rebuild burden,
cross-platform reach, confinement fronts, negative-control qualification,
credential exposure, and maintenance cost. It must preserve the distinction
between an invariant authority floor and a published per-mechanism confinement
profile; casual must not silently mean equivalent assurance.

Until answered, the existing bubblewrap capsule work is a candidate foundation
for the casual tier, not presumptively disposable and not the presumed hardened
production backend.
