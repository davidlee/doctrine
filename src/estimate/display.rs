// SPDX-License-Identifier: GPL-3.0-only

use super::EstimateFacet;

pub(crate) fn format_bound(f: f64) -> String {
    debug_assert!(f.is_finite());

    let rounded = (f * 10.0).round() / 10.0;
    let fractional = (rounded - rounded.trunc()).abs();

    if fractional <= f64::EPSILON {
        format!("{rounded:.0}")
    } else {
        format!("{rounded:.1}")
    }
}

#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "preserved for future verbose display mode (e.g. slice show --detail)"
    )
)]
pub(crate) fn format_estimate_normal(facet: Option<&EstimateFacet>, unit: &str) -> String {
    debug_assert!(!unit.is_empty());

    match facet {
        Some(facet) => format!(
            "Estimate: {}-{} {}",
            format_bound(facet.lower),
            format_bound(facet.upper),
            unit
        ),
        None => "Estimate: none recorded".to_string(),
    }
}

#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "preserved for future verbose display mode (e.g. slice show --detail)"
    )
)]
pub(crate) fn format_estimate_verbose(facet: Option<&EstimateFacet>, unit: &str) -> Vec<String> {
    debug_assert!(!unit.is_empty());

    let Some(facet) = facet else {
        return Vec::new();
    };

    let spread = if facet.lower > 0.0 {
        format!(
            "  Attention spread: {}x",
            format_bound(facet.upper / facet.lower)
        )
    } else {
        "  Attention spread: ratio unavailable".to_string()
    };

    let width = format!(
        "  Attention width: {} {}",
        format_bound(facet.upper - facet.lower),
        unit
    );

    vec![spread, width]
}

/// Render a confidence-framed estimate line for `slice show`.
///
/// Formula: `lower_bound = facet.lower + lower_pct * (facet.upper - facet.lower)`,
/// `upper_bound = facet.lower + upper_pct * (facet.upper - facet.lower)`.
/// Output: `"{:.1}–{:.1} {unit} ({:.0}% confidence)"`
pub(crate) fn format_estimate_confidence(
    facet: &EstimateFacet,
    lower_pct: f64,
    upper_pct: f64,
    unit: &str,
) -> String {
    debug_assert!(!unit.is_empty());
    debug_assert!(lower_pct.is_finite() && upper_pct.is_finite());
    debug_assert!((0.0..=1.0).contains(&lower_pct) && (0.0..=1.0).contains(&upper_pct));
    debug_assert!(lower_pct < upper_pct);

    let width = facet.upper - facet.lower;
    let lo = facet.lower + lower_pct * width;
    let hi = facet.lower + upper_pct * width;
    let pct = (upper_pct - lower_pct) * 100.0;

    format!("estimate: {lo:.1}–{hi:.1} {unit} ({pct:.0}% confidence)")
}

#[cfg(test)]
mod tests {
    use super::{format_bound, format_estimate_normal, format_estimate_verbose};
    use crate::estimate::EstimateFacet;

    #[test]
    fn vt1_format_bound_rows() {
        let cases = [
            (0.0, "0"),
            (2.0, "2"),
            (2.5, "2.5"),
            (3.75, "3.8"),
            (2.33333, "2.3"),
            (0.1, "0.1"),
            (100.0, "100"),
            (2.000000000001, "2"),
        ];

        for (input, expected) in cases {
            assert_eq!(format_bound(input), expected);
        }
    }

    #[test]
    fn vt2_format_estimate_normal_present() {
        let facet = EstimateFacet {
            lower: 2.0,
            upper: 8.0,
        };

        assert_eq!(
            format_estimate_normal(Some(&facet), "espresso_shots"),
            "Estimate: 2-8 espresso_shots"
        );
    }

    #[test]
    fn vt3_format_estimate_normal_absent() {
        assert_eq!(
            format_estimate_normal(None, "espresso_shots"),
            "Estimate: none recorded"
        );
    }

    #[test]
    fn vt4_format_estimate_normal_float() {
        let facet = EstimateFacet {
            lower: 2.5,
            upper: 8.0,
        };

        assert_eq!(
            format_estimate_normal(Some(&facet), "espresso_shots"),
            "Estimate: 2.5-8 espresso_shots"
        );
    }

    #[test]
    fn vt5_format_estimate_verbose_absent() {
        assert!(format_estimate_verbose(None, "espresso_shots").is_empty());
    }

    #[test]
    fn vt6_format_estimate_verbose_normal() {
        let facet = EstimateFacet {
            lower: 2.0,
            upper: 8.0,
        };

        assert_eq!(
            format_estimate_verbose(Some(&facet), "espresso_shots"),
            vec![
                "  Attention spread: 4x".to_string(),
                "  Attention width: 6 espresso_shots".to_string(),
            ]
        );
    }

    #[test]
    fn vt7_format_estimate_verbose_lower_zero() {
        let facet = EstimateFacet {
            lower: 0.0,
            upper: 5.0,
        };

        assert_eq!(
            format_estimate_verbose(Some(&facet), "espresso_shots"),
            vec![
                "  Attention spread: ratio unavailable".to_string(),
                "  Attention width: 5 espresso_shots".to_string(),
            ]
        );
    }

    #[test]
    fn vt8_zero_width() {
        let facet = EstimateFacet {
            lower: 5.0,
            upper: 5.0,
        };

        assert_eq!(
            format_estimate_normal(Some(&facet), "shots"),
            "Estimate: 5-5 shots"
        );
        assert_eq!(
            format_estimate_verbose(Some(&facet), "shots"),
            vec![
                "  Attention spread: 1x".to_string(),
                "  Attention width: 0 shots".to_string(),
            ]
        );
    }
}
