//! Strand activation records.

use serde::{Deserialize, Serialize};

/// Records which personality strand was activated and with what weight.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrandActivation {
    /// The strand name (e.g. "analytical", "candid").
    pub strand: String,
    /// Activation weight in the range `[0.0, 1.0]`.
    pub weight: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_strand_activation() {
        let sa = StrandActivation {
            strand: "analytical".into(),
            weight: 0.8,
        };
        let json = serde_json::to_string(&sa).expect("serialize");
        let back: StrandActivation = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(sa.strand, back.strand);
        assert!((sa.weight - back.weight).abs() < f64::EPSILON);
    }
}
