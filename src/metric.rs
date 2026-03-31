//! Metric definitions for data-driven quantity takeoff.
//!
//! A [`MetricDefinition`] describes *what* to measure, *from which*
//! elements, and *how* to aggregate.  Define metrics in code (or
//! deserialise from YAML/JSON) and pass them to
//! [`run_takeoff`](crate::takeoff::run_takeoff).

/// How to aggregate individual quantity values into a single result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Aggregation {
    /// Sum all values.
    Sum,
    /// Count matching elements (ignores quantity values).
    Count,
    /// Arithmetic mean.
    Average,
    /// Minimum value.
    Min,
    /// Maximum value.
    Max,
}

/// A declarative description of one quantity to extract and aggregate.
///
/// # Examples
///
/// ```
/// use ifc_qto_cat::{MetricDefinition, Aggregation};
///
/// let metric = MetricDefinition::new(
///     "Total Wall Volume".into(),
///     vec!["IFCWALL".into(), "IFCWALLSTANDARDCASE".into()],
///     "Qto_WallBaseQuantities".into(),
///     "NetVolume".into(),
///     Aggregation::Sum,
/// );
/// assert_eq!(metric.name(), "Total Wall Volume");
/// ```
#[derive(Debug, Clone)]
pub struct MetricDefinition {
    name: String,
    target_types: Vec<String>,
    qset_name: String,
    quantity_name: String,
    aggregation: Aggregation,
}

impl MetricDefinition {
    /// Construct a new metric definition.
    #[must_use]
    pub fn new(
        name: String,
        target_types: Vec<String>,
        qset_name: String,
        quantity_name: String,
        aggregation: Aggregation,
    ) -> Self {
        Self {
            name,
            target_types,
            qset_name,
            quantity_name,
            aggregation,
        }
    }

    /// Human-readable metric name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// IFC type names to include (e.g. `["IFCWALL", "IFCWALLSTANDARDCASE"]`).
    #[must_use]
    pub fn target_types(&self) -> &[String] {
        &self.target_types
    }

    /// Quantity set name (e.g. `"Qto_WallBaseQuantities"`).
    #[must_use]
    pub fn qset_name(&self) -> &str {
        &self.qset_name
    }

    /// Quantity name within the set (e.g. `"NetVolume"`).
    #[must_use]
    pub fn quantity_name(&self) -> &str {
        &self.quantity_name
    }

    /// Aggregation method.
    #[must_use]
    pub fn aggregation(&self) -> Aggregation {
        self.aggregation
    }
}

// ── Predefined metrics ──────────────────────────────────────────────

/// Common predefined metrics for typical QTO workflows.
pub mod presets {
    use super::{Aggregation, MetricDefinition};

    /// Total net volume of all walls.
    #[must_use]
    pub fn wall_net_volume() -> MetricDefinition {
        MetricDefinition::new(
            "Wall Net Volume".into(),
            vec!["IFCWALL".into(), "IFCWALLSTANDARDCASE".into()],
            "Qto_WallBaseQuantities".into(),
            "NetVolume".into(),
            Aggregation::Sum,
        )
    }

    /// Total gross volume of all walls.
    #[must_use]
    pub fn wall_gross_volume() -> MetricDefinition {
        MetricDefinition::new(
            "Wall Gross Volume".into(),
            vec!["IFCWALL".into(), "IFCWALLSTANDARDCASE".into()],
            "Qto_WallBaseQuantities".into(),
            "GrossVolume".into(),
            Aggregation::Sum,
        )
    }

    /// Total net volume of all slabs.
    #[must_use]
    pub fn slab_net_volume() -> MetricDefinition {
        MetricDefinition::new(
            "Slab Net Volume".into(),
            vec!["IFCSLAB".into()],
            "Qto_SlabBaseQuantities".into(),
            "NetVolume".into(),
            Aggregation::Sum,
        )
    }

    /// Total net floor area of all spaces.
    #[must_use]
    pub fn space_net_floor_area() -> MetricDefinition {
        MetricDefinition::new(
            "Net Floor Area".into(),
            vec!["IFCSPACE".into()],
            "Qto_SpaceBaseQuantities".into(),
            "NetFloorArea".into(),
            Aggregation::Sum,
        )
    }

    /// Total gross floor area of all spaces.
    #[must_use]
    pub fn space_gross_floor_area() -> MetricDefinition {
        MetricDefinition::new(
            "Gross Floor Area".into(),
            vec!["IFCSPACE".into()],
            "Qto_SpaceBaseQuantities".into(),
            "GrossFloorArea".into(),
            Aggregation::Sum,
        )
    }

    /// Count of all doors.
    #[must_use]
    pub fn door_count() -> MetricDefinition {
        MetricDefinition::new(
            "Door Count".into(),
            vec!["IFCDOOR".into()],
            String::new(),
            String::new(),
            Aggregation::Count,
        )
    }

    /// Count of all windows.
    #[must_use]
    pub fn window_count() -> MetricDefinition {
        MetricDefinition::new(
            "Window Count".into(),
            vec!["IFCWINDOW".into()],
            String::new(),
            String::new(),
            Aggregation::Count,
        )
    }

    /// Total door area.
    #[must_use]
    pub fn door_total_area() -> MetricDefinition {
        MetricDefinition::new(
            "Door Total Area".into(),
            vec!["IFCDOOR".into()],
            "Qto_DoorBaseQuantities".into(),
            "Area".into(),
            Aggregation::Sum,
        )
    }

    /// Total column net volume.
    #[must_use]
    pub fn column_net_volume() -> MetricDefinition {
        MetricDefinition::new(
            "Column Net Volume".into(),
            vec!["IFCCOLUMN".into()],
            "Qto_ColumnBaseQuantities".into(),
            "NetVolume".into(),
            Aggregation::Sum,
        )
    }
}
