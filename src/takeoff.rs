//! Top-level takeoff pipeline.
//!
//! [`run_takeoff`] accepts IFC content and a list of
//! [`MetricDefinition`]s, then scans, indexes, extracts quantities,
//! and aggregates them into a [`TakeoffReport`] -- all wrapped in a
//! single [`Io`] so `run` is called only at the boundary.

use comp_cat_rs::effect::io::Io;

use ifc_lite_core_cat::scan::{build_entity_index, scan_next};
use ifc_lite_core_cat::EntityId;

use crate::error::Error;
use crate::extract::{build_relation_map, extract_element_quantities};
use crate::metric::{Aggregation, MetricDefinition};
use crate::quantity::{Quantity, QuantityKind};

// ═══════════════════════════════════════════════════════════════════
// TakeoffResult / TakeoffReport
// ═══════════════════════════════════════════════════════════════════

/// The result of evaluating a single metric.
#[derive(Debug, Clone)]
pub struct TakeoffResult {
    metric_name: String,
    value: f64,
    unit: Option<QuantityKind>,
    element_count: usize,
}

impl TakeoffResult {
    /// Metric name.
    #[must_use]
    pub fn metric_name(&self) -> &str {
        &self.metric_name
    }

    /// Aggregated numeric value.
    #[must_use]
    pub fn value(&self) -> f64 {
        self.value
    }

    /// Quantity kind (if applicable; `None` for pure counts).
    #[must_use]
    pub fn unit(&self) -> Option<QuantityKind> {
        self.unit
    }

    /// Number of elements that contributed to the result.
    #[must_use]
    pub fn element_count(&self) -> usize {
        self.element_count
    }
}

impl std::fmt::Display for TakeoffResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let unit_str = self.unit.map_or("", QuantityKind::unit_label);
        write!(
            f,
            "{}: {:.3}{} ({} elements)",
            self.metric_name, self.value, unit_str, self.element_count
        )
    }
}

/// A complete takeoff report containing results for all metrics.
#[derive(Debug, Clone)]
pub struct TakeoffReport {
    results: Vec<TakeoffResult>,
}

impl TakeoffReport {
    /// Construct from a list of results.
    #[must_use]
    pub fn new(results: Vec<TakeoffResult>) -> Self {
        Self { results }
    }

    /// All metric results.
    #[must_use]
    pub fn results(&self) -> &[TakeoffResult] {
        &self.results
    }

    /// Look up a result by metric name.
    #[must_use]
    pub fn get(&self, metric_name: &str) -> Option<&TakeoffResult> {
        self.results.iter().find(|r| r.metric_name() == metric_name)
    }

    /// Number of metrics evaluated.
    #[must_use]
    pub fn len(&self) -> usize {
        self.results.len()
    }

    /// Whether no metrics were evaluated.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.results.is_empty()
    }
}

impl std::fmt::Display for TakeoffReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.results
            .iter()
            .try_for_each(|r| writeln!(f, "{r}"))
    }
}

// ═══════════════════════════════════════════════════════════════════
// Pipeline
// ═══════════════════════════════════════════════════════════════════

/// Run a complete quantity takeoff.
///
/// Scans the IFC content, builds an entity index and relation map,
/// then evaluates each metric.  The entire pipeline is deferred
/// inside an [`Io`]; call `.run()` only at the boundary.
///
/// # Examples
///
/// ```
/// use ifc_qto_cat::{run_takeoff, MetricDefinition, Aggregation};
///
/// let content = "\
/// #1=IFCWALL('g1',$,$,$,$,$,$,$);
/// #2=IFCELEMENTQUANTITY('g2',$,'Qto_WallBaseQuantities',$,$,(#3));
/// #3=IFCQUANTITYVOLUME('NetVolume',$,$,12.5,$);
/// #5=IFCRELDEFINESBYPROPERTIES('g5',$,$,$,(#1),#2);
/// ".to_string();
///
/// let metrics = vec![MetricDefinition::new(
///     "Wall Volume".into(),
///     vec!["IFCWALL".into()],
///     "Qto_WallBaseQuantities".into(),
///     "NetVolume".into(),
///     Aggregation::Sum,
/// )];
///
/// let report = run_takeoff(content, metrics).run().expect("takeoff");
/// let result = report.get("Wall Volume").expect("metric");
/// assert!((result.value() - 12.5).abs() < 0.001);
/// ```
#[must_use]
pub fn run_takeoff(content: String, metrics: Vec<MetricDefinition>) -> Io<Error, TakeoffReport> {
    Io::suspend(move || {
        let index = build_entity_index(&content);
        let rel_map = build_relation_map(&content, &index)?;

        let results: Vec<TakeoffResult> = metrics
            .iter()
            .filter_map(|metric| evaluate_metric(&content, &index, &rel_map, metric))
            .collect();

        Ok(TakeoffReport::new(results))
    })
}

/// Evaluate a single metric against the indexed content.
fn evaluate_metric(
    content: &str,
    index: &std::collections::HashMap<EntityId, (usize, usize)>,
    rel_map: &std::collections::HashMap<EntityId, Vec<EntityId>>,
    metric: &MetricDefinition,
) -> Option<TakeoffResult> {
    let bytes = content.as_bytes();

    // Find all elements matching the target types.
    let matching_ids: Vec<EntityId> = std::iter::successors(
        scan_next(bytes, 0),
        #[allow(clippy::needless_borrows_for_generic_args)]
        |&(ref _e, pos)| scan_next(bytes, pos),
    )
    .filter(|(entity, _)| metric.target_types().iter().any(|t| t == entity.type_name()))
    .map(|(entity, _)| entity.id())
    .collect();

    let element_count = matching_ids.len();

    // For Count aggregation, just return the count.
    if metric.aggregation() == Aggregation::Count {
        return Some(TakeoffResult {
            metric_name: metric.name().to_string(),
            #[allow(clippy::cast_precision_loss)]
            value: element_count as f64,
            unit: Some(QuantityKind::Count),
            element_count,
        });
    }

    // Extract the requested quantity from each matching element.
    let values: Vec<f64> = matching_ids
        .iter()
        .flat_map(|&elem_id| extract_element_quantities(content, index, rel_map, elem_id))
        .filter(|qset| qset.name() == metric.qset_name())
        .filter_map(|qset| qset.get(metric.quantity_name()).map(Quantity::value))
        .collect();

    (!values.is_empty()).then(|| {
        let (aggregated, kind) = aggregate(&values, metric);
        TakeoffResult {
            metric_name: metric.name().to_string(),
            value: aggregated,
            unit: kind,
            element_count,
        }
    })
}

/// Apply the aggregation function to a slice of values.
fn aggregate(values: &[f64], metric: &MetricDefinition) -> (f64, Option<QuantityKind>) {
    let kind = QuantityKind::from_type_name(
        // Infer kind from qset/quantity naming conventions.
        match metric.quantity_name() {
            n if n.contains("Volume") => "IFCQUANTITYVOLUME",
            n if n.contains("Area") => "IFCQUANTITYAREA",
            n if n.contains("Length") || n.contains("Width") || n.contains("Height") || n.contains("Depth") || n.contains("Perimeter") => "IFCQUANTITYLENGTH",
            n if n.contains("Weight") => "IFCQUANTITYWEIGHT",
            n if n.contains("Count") => "IFCQUANTITYCOUNT",
            n if n.contains("Time") => "IFCQUANTITYTIME",
            _ => "",
        },
    );

    let value = match metric.aggregation() {
        Aggregation::Sum => values.iter().sum(),
        Aggregation::Count => {
            #[allow(clippy::cast_precision_loss)]
            let v = values.len() as f64;
            v
        }
        Aggregation::Average => {
            #[allow(clippy::cast_precision_loss)]
            let avg = values.iter().sum::<f64>() / values.len().max(1) as f64;
            avg
        }
        Aggregation::Min => values.iter().copied().fold(f64::INFINITY, f64::min),
        Aggregation::Max => values.iter().copied().fold(f64::NEG_INFINITY, f64::max),
    };

    (value, kind)
}

// ═══════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metric::presets;

    fn sample_ifc() -> String {
        "\
#1=IFCWALL('g1',$,$,$,$,$,$,$);
#2=IFCWALL('g2',$,$,$,$,$,$,$);
#10=IFCELEMENTQUANTITY('gq1',$,'Qto_WallBaseQuantities',$,$,(#11,#12));
#11=IFCQUANTITYLENGTH('Length',$,$,5.0,$);
#12=IFCQUANTITYVOLUME('NetVolume',$,$,12.5,$);
#20=IFCELEMENTQUANTITY('gq2',$,'Qto_WallBaseQuantities',$,$,(#21,#22));
#21=IFCQUANTITYLENGTH('Length',$,$,3.0,$);
#22=IFCQUANTITYVOLUME('NetVolume',$,$,7.5,$);
#30=IFCRELDEFINESBYPROPERTIES('gr1',$,$,$,(#1),#10);
#31=IFCRELDEFINESBYPROPERTIES('gr2',$,$,$,(#2),#20);
#40=IFCDOOR('gd1',$,$,$,$,$,$,$);
#41=IFCDOOR('gd2',$,$,$,$,$,$,$);
#42=IFCDOOR('gd3',$,$,$,$,$,$,$);
"
        .to_string()
    }

    #[test]
    fn takeoff_sum_wall_volume() {
        let report = run_takeoff(sample_ifc(), vec![presets::wall_net_volume()])
            .run()
            .expect("takeoff");
        let result = report.get("Wall Net Volume").expect("metric");
        assert!((result.value() - 20.0).abs() < 0.001);
        assert_eq!(result.element_count(), 2);
    }

    #[test]
    fn takeoff_count_doors() {
        let report = run_takeoff(sample_ifc(), vec![presets::door_count()])
            .run()
            .expect("takeoff");
        let result = report.get("Door Count").expect("metric");
        assert!((result.value() - 3.0).abs() < 0.001);
        assert_eq!(result.element_count(), 3);
    }

    #[test]
    fn takeoff_multiple_metrics() {
        let metrics = vec![
            presets::wall_net_volume(),
            presets::door_count(),
        ];
        let report = run_takeoff(sample_ifc(), metrics)
            .run()
            .expect("takeoff");
        assert_eq!(report.len(), 2);
    }

    #[test]
    fn takeoff_average_wall_length() {
        let metric = MetricDefinition::new(
            "Avg Wall Length".into(),
            vec!["IFCWALL".into()],
            "Qto_WallBaseQuantities".into(),
            "Length".into(),
            Aggregation::Average,
        );
        let report = run_takeoff(sample_ifc(), vec![metric])
            .run()
            .expect("takeoff");
        let result = report.get("Avg Wall Length").expect("metric");
        assert!((result.value() - 4.0).abs() < 0.001); // (5+3)/2
    }

    #[test]
    fn report_display() {
        let report = run_takeoff(sample_ifc(), vec![presets::wall_net_volume()])
            .run()
            .expect("takeoff");
        let display = format!("{report}");
        assert!(display.contains("Wall Net Volume"));
        assert!(display.contains("20.000"));
    }
}
