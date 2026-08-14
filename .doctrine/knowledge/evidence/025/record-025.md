# EVD-025: Oubliette host integration and macOS portability costs

## Observation

Oubliette's current Firecracker/microvm.nix formulation requires materially more
host shape than the bubblewrap capsule: a NixOS host module, systemd services,
and host rebuilds for some configuration changes. Transaction startup and disk
measurements do not include that adoption and configuration cost.

The macOS evaluation in `../microvm-spike/docs/eval-macos.md` finds that
microvm.nix can boot an aarch64 NixOS guest through vfkit or QEMU/HVF, but:

- the Linux guest must be built by a Linux builder;
- vfkit supplies NAT rather than the tap/no-default-route perimeter Oubliette
  relies on;
- QEMU on macOS does not restore that tap shape.

`../microvm-spike/docs/plan-b-other-jails.md` identifies two viable macOS
alternatives: a restricted QEMU/UTM Linux VM for a hardened tier, and native
Seatbelt with the same proxy and host-initiated Git perimeter for a low-barrier
tier. It also describes non-NixOS Linux as feasible, but with host-side service,
firewall, KVM, and tap plumbing that the NixOS module currently supplies.

## Bearing

MicroVM runtime cost alone cannot decide backend retention. Portability and host
integration cost support evaluating a casual bwrap/Seatbelt tier separately
from the hardened Firecracker profile.
