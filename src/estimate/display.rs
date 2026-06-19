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
