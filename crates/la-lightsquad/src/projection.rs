//! Subset projection: [`GateDimension`] → [`Pillar`] (7-variant webshell UI subset).
//!
//! The TypeScript webshell uses a 7-variant `Pillar` type for its UI surfaces.
//! Three canonical dimensions — Canon [C], Knowledge [K], Research [R] — have no
//! current UI counterpart and project to `None`. `Custom` dimensions also project
//! to `None`.
//!
//! # Drift protection
//!
//! [`From<&GateDimension> for Option<Pillar>`] uses an exhaustive match **without
//! a wildcard arm**. Adding a new variant to [`GateDimension`] will produce a
//! compile error here, preventing silent drift between the canonical 10-dimension
//! framework and the UI projection.

use crate::GateDimension;
use serde::{Deserialize, Serialize};

/// Subset of [`GateDimension`] rendered by the webshell Cockpit UI.
///
/// Maps to the TypeScript union `type Pillar = 'ARCH' | 'SEC' | 'QUAL' | 'PERF' |
/// 'TEST' | 'DOC' | 'OPS'`.
///
/// Three canonical gate dimensions have no `Pillar` counterpart yet:
/// Canon [C], Knowledge [K], and Research [R]. When the Cockpit UI gains those
/// panels, add variants here and update the TS type in `types.ts`.
///
/// Serializes as uppercase strings (`"ARCH"`, `"SEC"`, etc.) to match the TS wire format.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum Pillar {
    /// [A] Architecture — maps to [`GateDimension::Architecture`].
    Arch,
    /// [S] Security — maps to [`GateDimension::Security`].
    Sec,
    /// [Q] Quality — maps to [`GateDimension::Quality`].
    Qual,
    /// [P] Performance — maps to [`GateDimension::Performance`].
    Perf,
    /// [T] Testing — maps to [`GateDimension::Testing`].
    Test,
    /// [D] Documentation — maps to [`GateDimension::Documentation`].
    Doc,
    /// [O] Operations — maps to [`GateDimension::Operations`].
    Ops,
}

impl Pillar {
    /// The canonical [`GateDimension`] this `Pillar` corresponds to.
    ///
    /// This is the left-inverse of `From<&GateDimension> for Option<Pillar>`:
    /// `pillar.canonical_dimension().into() == Some(pillar)` always holds.
    #[must_use]
    pub fn canonical_dimension(self) -> GateDimension {
        match self {
            Self::Arch => GateDimension::Architecture,
            Self::Sec => GateDimension::Security,
            Self::Qual => GateDimension::Quality,
            Self::Perf => GateDimension::Performance,
            Self::Test => GateDimension::Testing,
            Self::Doc => GateDimension::Documentation,
            Self::Ops => GateDimension::Operations,
        }
    }
}

impl From<&GateDimension> for Option<Pillar> {
    /// Project a [`GateDimension`] onto the 7-variant webshell UI subset.
    ///
    /// Returns `None` for Canon, Knowledge, Research, and Custom — these have
    /// no counterpart in the current UI surface.
    ///
    /// The match is **exhaustive without wildcards**: any new `GateDimension`
    /// variant must be explicitly assigned a `Pillar` or `None` here.
    fn from(dim: &GateDimension) -> Self {
        match dim {
            GateDimension::Architecture => Some(Pillar::Arch),
            GateDimension::Security => Some(Pillar::Sec),
            GateDimension::Quality => Some(Pillar::Qual),
            GateDimension::Performance => Some(Pillar::Perf),
            GateDimension::Testing => Some(Pillar::Test),
            GateDimension::Documentation => Some(Pillar::Doc),
            GateDimension::Operations => Some(Pillar::Ops),
            // No Pillar counterpart yet — add variants above when the UI gains these panels.
            GateDimension::Canon | GateDimension::Knowledge | GateDimension::Research => None,
            GateDimension::Custom(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seven_dimensions_project_to_some() {
        let projectable = [
            GateDimension::Architecture,
            GateDimension::Security,
            GateDimension::Quality,
            GateDimension::Performance,
            GateDimension::Testing,
            GateDimension::Documentation,
            GateDimension::Operations,
        ];
        for dim in &projectable {
            assert!(
                Option::<Pillar>::from(dim).is_some(),
                "{dim:?} should project to Some(Pillar)"
            );
        }
    }

    #[test]
    fn three_plus_custom_project_to_none() {
        let non_projectable = [
            GateDimension::Canon,
            GateDimension::Knowledge,
            GateDimension::Research,
            GateDimension::Custom("bespoke".to_owned()),
        ];
        for dim in &non_projectable {
            assert!(
                Option::<Pillar>::from(dim).is_none(),
                "{dim:?} should project to None"
            );
        }
    }

    #[test]
    fn canonical_dimension_round_trips() {
        let pillars = [
            Pillar::Arch,
            Pillar::Sec,
            Pillar::Qual,
            Pillar::Perf,
            Pillar::Test,
            Pillar::Doc,
            Pillar::Ops,
        ];
        for pillar in pillars {
            let dim = pillar.canonical_dimension();
            let back = Option::<Pillar>::from(&dim);
            assert_eq!(
                back,
                Some(pillar),
                "{pillar:?} → canonical_dimension() → Pillar should round-trip"
            );
        }
    }

    #[test]
    fn pillar_serializes_screaming_snake_case() {
        assert_eq!(serde_json::to_string(&Pillar::Arch).unwrap(), r#""ARCH""#);
        assert_eq!(serde_json::to_string(&Pillar::Sec).unwrap(), r#""SEC""#);
        assert_eq!(serde_json::to_string(&Pillar::Qual).unwrap(), r#""QUAL""#);
        assert_eq!(serde_json::to_string(&Pillar::Perf).unwrap(), r#""PERF""#);
        assert_eq!(serde_json::to_string(&Pillar::Test).unwrap(), r#""TEST""#);
        assert_eq!(serde_json::to_string(&Pillar::Doc).unwrap(), r#""DOC""#);
        assert_eq!(serde_json::to_string(&Pillar::Ops).unwrap(), r#""OPS""#);
    }

    #[test]
    fn pillar_deserializes_screaming_snake_case() {
        let p: Pillar = serde_json::from_str(r#""ARCH""#).unwrap();
        assert_eq!(p, Pillar::Arch);
        let p: Pillar = serde_json::from_str(r#""OPS""#).unwrap();
        assert_eq!(p, Pillar::Ops);
    }
}
