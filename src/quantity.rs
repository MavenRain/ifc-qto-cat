//! Core quantity types for IFC quantity takeoff.
//!
//! Models the six IFC quantity kinds (length, area, volume, weight,
//! count, time) and the quantity sets that group them.

use ifc_lite_core_cat::EntityId;

/// The six measurement kinds defined by IFC4.3.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QuantityKind {
    /// `IfcQuantityLength` -- metres.
    Length,
    /// `IfcQuantityArea` -- square metres.
    Area,
    /// `IfcQuantityVolume` -- cubic metres.
    Volume,
    /// `IfcQuantityWeight` -- kilograms.
    Weight,
    /// `IfcQuantityCount` -- dimensionless count.
    Count,
    /// `IfcQuantityTime` -- seconds.
    Time,
}

impl QuantityKind {
    /// Match an IFC type name to a quantity kind.
    #[must_use]
    pub fn from_type_name(name: &str) -> Option<Self> {
        match name {
            "IFCQUANTITYLENGTH" => Some(Self::Length),
            "IFCQUANTITYAREA" => Some(Self::Area),
            "IFCQUANTITYVOLUME" => Some(Self::Volume),
            "IFCQUANTITYWEIGHT" => Some(Self::Weight),
            "IFCQUANTITYCOUNT" => Some(Self::Count),
            "IFCQUANTITYTIME" => Some(Self::Time),
            _ => None,
        }
    }

    /// Unit label for display.
    #[must_use]
    pub fn unit_label(self) -> &'static str {
        match self {
            Self::Length => "m",
            Self::Area => "m\u{00b2}",
            Self::Volume => "m\u{00b3}",
            Self::Weight => "kg",
            Self::Count => "",
            Self::Time => "s",
        }
    }
}

impl std::fmt::Display for QuantityKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Length => f.write_str("Length"),
            Self::Area => f.write_str("Area"),
            Self::Volume => f.write_str("Volume"),
            Self::Weight => f.write_str("Weight"),
            Self::Count => f.write_str("Count"),
            Self::Time => f.write_str("Time"),
        }
    }
}

/// A single named quantity value extracted from an IFC file.
///
/// # Examples
///
/// ```
/// use ifc_qto_cat::Quantity;
/// use ifc_qto_cat::QuantityKind;
///
/// let q = Quantity::new("NetVolume".into(), QuantityKind::Volume, 4.5);
/// assert_eq!(q.name(), "NetVolume");
/// assert_eq!(q.value(), 4.5);
/// ```
#[derive(Debug, Clone)]
pub struct Quantity {
    name: String,
    kind: QuantityKind,
    value: f64,
}

impl Quantity {
    /// Construct a new quantity.
    #[must_use]
    pub fn new(name: String, kind: QuantityKind, value: f64) -> Self {
        Self { name, kind, value }
    }

    /// Quantity name (e.g. `"NetVolume"`, `"Length"`).
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Measurement kind.
    #[must_use]
    pub fn kind(&self) -> QuantityKind {
        self.kind
    }

    /// Numeric value.
    #[must_use]
    pub fn value(&self) -> f64 {
        self.value
    }
}

/// A named set of quantities attached to an IFC element.
///
/// Corresponds to an `IfcElementQuantity` entity with a name like
/// `"Qto_WallBaseQuantities"`.
#[derive(Debug, Clone)]
pub struct QuantitySet {
    name: String,
    entity_id: EntityId,
    quantities: Vec<Quantity>,
}

impl QuantitySet {
    /// Construct a new quantity set.
    #[must_use]
    pub fn new(name: String, entity_id: EntityId, quantities: Vec<Quantity>) -> Self {
        Self {
            name,
            entity_id,
            quantities,
        }
    }

    /// Set name (e.g. `"Qto_WallBaseQuantities"`).
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Entity id of the `IfcElementQuantity`.
    #[must_use]
    pub fn entity_id(&self) -> EntityId {
        self.entity_id
    }

    /// All quantities in this set.
    #[must_use]
    pub fn quantities(&self) -> &[Quantity] {
        &self.quantities
    }

    /// Look up a quantity by name.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&Quantity> {
        self.quantities.iter().find(|q| q.name() == name)
    }

    /// Sum of all quantities with the given name.
    #[must_use]
    pub fn sum_by_name(&self, name: &str) -> f64 {
        self.quantities
            .iter()
            .filter(|q| q.name() == name)
            .map(Quantity::value)
            .sum()
    }
}
