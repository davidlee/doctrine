// SPDX-License-Identifier: GPL-3.0-only
//! `capacity` — advisory disk-capacity tiering (SL-248 `sec-5` § Capacity,
//! `REQ-461`, `DEC-158`).
//!
//! `REQ-461` asks provisioning to compare available space against a configurable
//! expected capsule size, warn conspicuously below a threshold, halt on
//! exhaustion, and to do **none** of pre-reservation, throughput backpressure,
//! eviction or rescue-archive construction. This module is the whole of the
//! positive half and, structurally, of the negative half too (`EX-20`):
//!
//! * no pre-reservation — nothing here writes or claims anything, and
//!   `CapacityPolicy` holds no reserved figure;
//! * no backpressure — [`assess_capacity`] is a function of one probe and one
//!   policy; it does not know how many capsules exist and nothing queues on it;
//! * no eviction — there is no removal call in this module, or in this phase;
//! * no rescue archive — nothing is copied or compressed on any path.
//!
//! **Capacity is advisory in one direction only** (`sec-5` invariant 5): it may
//! refuse before work starts; it may never permit what another rule refuses.
//!
//! Layering (`ADR-001`): `capacity` is `leaf`, out-edges `{config, host}` — the
//! `host` edge is [`CapacityUnknown`], which lives with the probe that produces
//! it (`D2`).
#![cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "staged ahead of PHASE-06's provision consumer (PHASE-03 D5); \
                  PHASE-06 deletes this line when `provision` lands"
    )
)]

use std::path::{Path, PathBuf};

use crate::config::{ByteCount, CapacityPolicy, KEY_EXPECTED_CAPSULE_SIZE_MIB};
use crate::host::CapacityUnknown;

/// The tier a probed figure falls in, given a policy. Nothing else.
///
/// Deliberately rootless: `EX-14` fixes [`assess_capacity`] at two parameters,
/// neither of which knows where the capsule root is. The root arrives when the
/// verdict is turned into a [`CapacityReport`] by the caller that probed
/// (`F-3`, `D3`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CapacityVerdict {
    /// At or above `expected × multiplier` — provisioning continues silently.
    Ample { available: ByteCount },
    /// At or above expected, below the threshold — warn and continue.
    Low {
        available: ByteCount,
        expected: ByteCount,
        threshold: ByteCount,
    },
    /// Below expected — refuse.
    Insufficient {
        available: ByteCount,
        expected: ByteCount,
    },
    /// The probe could not produce a usable figure — report and continue.
    Unknown { reason: CapacityUnknown },
}

/// **PURE**, given the probe's answer (`EX-14`).
///
/// The threshold multiplication **saturates** at `u64::MAX` while the mebibyte
/// *conversion* in `config` refuses — the two are deliberately different, and
/// both are asserted. A threshold beyond the largest representable disk is a
/// threshold nothing reaches, which is the correct behaviour; a configured size
/// that cannot be represented is an operator error.
pub(crate) fn assess_capacity(
    probe: Result<ByteCount, CapacityUnknown>,
    policy: &CapacityPolicy,
) -> CapacityVerdict {
    let available = match probe {
        Ok(available) => available,
        Err(reason) => return CapacityVerdict::Unknown { reason },
    };

    let expected = policy.expected_capsule_size();
    let threshold = expected.saturating_mul_u32(policy.warn_multiplier());

    if available >= threshold {
        CapacityVerdict::Ample { available }
    } else if available >= expected {
        CapacityVerdict::Low {
            available,
            expected,
            threshold,
        }
    } else {
        CapacityVerdict::Insufficient {
            available,
            expected,
        }
    }
}

/// A verdict paired with the capsule root it was probed against and the key an
/// operator changes — the emitted form (`D3`, `EX-16`).
///
/// **Structured, never a formatted sentence.** `REQ-461` criterion 1 asks for
/// conspicuous, and a formatted line is neither machine-readable nor greppable
/// by an operator triaging a stalled queue. A warning and a refusal about the
/// same condition therefore differ in severity and not in vocabulary. Nothing
/// here prints: `clippy::print_stdout` / `print_stderr` are `deny` and there is
/// no verb in this phase to print from — PHASE-06 owns the emission.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CapacityReport {
    /// Ample — provisioning continues silently.
    Proceed {
        available_bytes: u64,
        capsule_root: PathBuf,
    },
    /// Low — the conspicuous warning, with the five named fields `EX-16` lists.
    /// Provisioning **continues**, and nothing is reserved.
    Warn {
        available_bytes: u64,
        expected_bytes: u64,
        threshold_bytes: u64,
        capsule_root: PathBuf,
        key: &'static str,
    },
    /// Insufficient — provisioning refuses, naming the root and the key.
    Refuse {
        available_bytes: u64,
        expected_bytes: u64,
        capsule_root: PathBuf,
        key: &'static str,
    },
    /// An unusable probe: **reported**, never refused, never read as ample, and
    /// distinguishable from [`CapacityReport::Proceed`] — those two are
    /// identical to a silent implementation and opposite to an operator
    /// (`POL-002` facet 3).
    Report {
        reason: CapacityUnknown,
        capsule_root: PathBuf,
        key: &'static str,
    },
}

impl CapacityReport {
    /// Pair a verdict with the root the caller probed. The key is always
    /// `expected-capsule-size-mib`: it is the one an operator changes for every
    /// tier, so a warning and a refusal name the same lever.
    pub(crate) fn of(verdict: CapacityVerdict, capsule_root: PathBuf) -> Self {
        let key = KEY_EXPECTED_CAPSULE_SIZE_MIB;
        match verdict {
            CapacityVerdict::Ample { available } => Self::Proceed {
                available_bytes: available.as_u64(),
                capsule_root,
            },
            CapacityVerdict::Low {
                available,
                expected,
                threshold,
            } => Self::Warn {
                available_bytes: available.as_u64(),
                expected_bytes: expected.as_u64(),
                threshold_bytes: threshold.as_u64(),
                capsule_root,
                key,
            },
            CapacityVerdict::Insufficient {
                available,
                expected,
            } => Self::Refuse {
                available_bytes: available.as_u64(),
                expected_bytes: expected.as_u64(),
                capsule_root,
                key,
            },
            CapacityVerdict::Unknown { reason } => Self::Report {
                reason,
                capsule_root,
                key,
            },
        }
    }

    /// Whether provisioning halts. **Only** `Insufficient` halts — an unusable
    /// probe does not, because capacity is advisory under `REQ-461`.
    pub(crate) const fn refuses(&self) -> bool {
        matches!(*self, Self::Refuse { .. })
    }

    /// The capsule root the figure was probed against — not the repository,
    /// which is a different filesystem on any host separating `/home` from
    /// `/var` (`EX-15`).
    pub(crate) fn capsule_root(&self) -> &Path {
        match self {
            Self::Proceed { capsule_root, .. }
            | Self::Warn { capsule_root, .. }
            | Self::Refuse { capsule_root, .. }
            | Self::Report { capsule_root, .. } => capsule_root,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::{CapacityReport, CapacityVerdict, assess_capacity};
    use crate::config::{ByteCount, CapacityPolicy, KEY_EXPECTED_CAPSULE_SIZE_MIB};
    use crate::host::CapacityUnknown;

    const EXPECTED_BYTES: u64 = 4096;
    const MULTIPLIER: u32 = 2;
    const CAPSULE_ROOT: &str = "/var/lib/doctrine/capsules";

    /// `EXPECTED_BYTES × MULTIPLIER`. A function rather than a `const` because
    /// `u64::from` is not const-stable and `as` casts are banned
    /// (`clippy::as_conversions` is `deny`).
    fn threshold_bytes() -> u64 {
        EXPECTED_BYTES * u64::from(MULTIPLIER)
    }

    fn policy() -> CapacityPolicy {
        CapacityPolicy::new(ByteCount::from_bytes(EXPECTED_BYTES), MULTIPLIER)
    }

    fn assess(available: u64) -> CapacityVerdict {
        assess_capacity(Ok(ByteCount::from_bytes(available)), &policy())
    }

    fn report(available: u64) -> CapacityReport {
        CapacityReport::of(assess(available), PathBuf::from(CAPSULE_ROOT))
    }

    // ── T8 / VT-1: the tiers and saturation ───────────────────────────────

    /// The boundary either side, so the comparison is not off by one.
    #[test]
    fn available_at_the_threshold_is_ample_and_one_byte_below_it_is_low() {
        assert_eq!(
            assess(threshold_bytes()),
            CapacityVerdict::Ample {
                available: ByteCount::from_bytes(threshold_bytes())
            }
        );
        assert_eq!(
            assess(threshold_bytes() - 1),
            CapacityVerdict::Low {
                available: ByteCount::from_bytes(threshold_bytes() - 1),
                expected: ByteCount::from_bytes(EXPECTED_BYTES),
                threshold: ByteCount::from_bytes(threshold_bytes()),
            }
        );
    }

    #[test]
    fn available_at_expected_is_low_and_one_byte_below_it_is_insufficient() {
        assert_eq!(
            assess(EXPECTED_BYTES),
            CapacityVerdict::Low {
                available: ByteCount::from_bytes(EXPECTED_BYTES),
                expected: ByteCount::from_bytes(EXPECTED_BYTES),
                threshold: ByteCount::from_bytes(threshold_bytes()),
            }
        );
        assert_eq!(
            assess(EXPECTED_BYTES - 1),
            CapacityVerdict::Insufficient {
                available: ByteCount::from_bytes(EXPECTED_BYTES - 1),
                expected: ByteCount::from_bytes(EXPECTED_BYTES),
            }
        );
    }

    /// The threshold **saturates** where the mebibyte conversion in `config`
    /// **refuses** (`EX-9`) — the two are deliberately different, and a
    /// saturated threshold is simply one nothing reaches.
    #[test]
    fn threshold_saturates_rather_than_overflowing_on_a_large_multiplier() {
        let huge = CapacityPolicy::new(ByteCount::from_bytes(u64::MAX), u32::MAX);
        let verdict = assess_capacity(Ok(ByteCount::from_bytes(u64::MAX)), &huge);

        // u64::MAX × u32::MAX saturates to u64::MAX, which the available figure
        // exactly meets — so this is Ample and not a wrapped-around Insufficient.
        assert_eq!(
            verdict,
            CapacityVerdict::Ample {
                available: ByteCount::from_bytes(u64::MAX)
            }
        );

        // One byte below the saturated threshold is still above expected only
        // when expected is smaller; with expected at u64::MAX it is Insufficient.
        assert_eq!(
            assess_capacity(Ok(ByteCount::from_bytes(u64::MAX - 1)), &huge),
            CapacityVerdict::Insufficient {
                available: ByteCount::from_bytes(u64::MAX - 1),
                expected: ByteCount::from_bytes(u64::MAX),
            }
        );
    }

    // ── T9 / VT-2: what each verdict does ─────────────────────────────────

    #[test]
    fn insufficient_refuses_and_names_the_capsule_root_and_the_config_key() {
        let report = report(EXPECTED_BYTES - 1);
        assert!(report.refuses());
        assert_eq!(
            report,
            CapacityReport::Refuse {
                available_bytes: EXPECTED_BYTES - 1,
                expected_bytes: EXPECTED_BYTES,
                capsule_root: PathBuf::from(CAPSULE_ROOT),
                key: KEY_EXPECTED_CAPSULE_SIZE_MIB,
            }
        );
        assert_eq!(report.capsule_root(), Path::new(CAPSULE_ROOT));
    }

    /// Five named fields, and provisioning **continues**.
    #[test]
    fn low_warns_with_named_fields_and_provisioning_continues() {
        let report = report(threshold_bytes() - 1);
        assert!(!report.refuses());
        assert_eq!(
            report,
            CapacityReport::Warn {
                available_bytes: threshold_bytes() - 1,
                expected_bytes: EXPECTED_BYTES,
                threshold_bytes: threshold_bytes(),
                capsule_root: PathBuf::from(CAPSULE_ROOT),
                key: KEY_EXPECTED_CAPSULE_SIZE_MIB,
            }
        );
    }

    /// The whole of `REQ-461` criterion 2's no-reservation story: the next
    /// capsule provisioned sees the same free space this one did. Assessing
    /// twice against the same figure must give the same verdict, because
    /// nothing was written, claimed or held at `Low`.
    #[test]
    fn low_reserves_nothing_so_a_second_assessment_sees_the_same_figure() {
        let available = threshold_bytes() - 1;
        let first = assess(available);
        let second = assess(available);

        assert_eq!(first, second);
        assert_eq!(
            first,
            CapacityVerdict::Low {
                available: ByteCount::from_bytes(available),
                expected: ByteCount::from_bytes(EXPECTED_BYTES),
                threshold: ByteCount::from_bytes(threshold_bytes()),
            }
        );
    }

    // ── T10 / VT-3: the unusable probe ────────────────────────────────────

    /// Reported, never refused, never read as ample — and *distinguishable in
    /// the report* from an ample figure, which is the distinction a silent
    /// implementation loses (`POL-002` facet 3).
    #[test]
    fn unknown_is_reported_and_is_not_ample() {
        let verdict = assess_capacity(Err(CapacityUnknown::UnusableFigure), &policy());
        assert_eq!(
            verdict,
            CapacityVerdict::Unknown {
                reason: CapacityUnknown::UnusableFigure
            }
        );

        let unknown = CapacityReport::of(verdict, PathBuf::from(CAPSULE_ROOT));
        assert!(!unknown.refuses());
        assert_ne!(unknown, report(threshold_bytes()));
        assert!(matches!(unknown, CapacityReport::Report { .. }));
    }

    #[test]
    fn unknown_carries_which_of_the_three_reasons_produced_it() {
        for reason in [
            CapacityUnknown::ProbeFailed { errno: 13 },
            CapacityUnknown::UnusableFigure,
            CapacityUnknown::FigureOverflows,
        ] {
            assert_eq!(
                assess_capacity(Err(reason), &policy()),
                CapacityVerdict::Unknown { reason }
            );
            assert_eq!(
                CapacityReport::of(
                    CapacityVerdict::Unknown { reason },
                    PathBuf::from(CAPSULE_ROOT)
                ),
                CapacityReport::Report {
                    reason,
                    capsule_root: PathBuf::from(CAPSULE_ROOT),
                    key: KEY_EXPECTED_CAPSULE_SIZE_MIB,
                }
            );
        }
    }
}
