//! # IFC Quantity Takeoff
//!
//! Data-driven quantity takeoffs from IFC models, built on
//! [`comp_cat_rs`] and [`ifc_lite_core_cat`].
//!
//! ## Overview
//!
//! This crate extracts and aggregates standardised IFC quantities
//! (length, area, volume, weight, count) from building elements.
//! Define metrics declaratively and run them against any IFC file.
//!
//! ## Quick Start
//!
//! ```rust
//! use ifc_qto_cat::{run_takeoff, metric::presets};
//!
//! let content = "\
//! #1=IFCWALL('g1',$,$,$,$,$,$,$);
//! #2=IFCELEMENTQUANTITY('g2',$,'Qto_WallBaseQuantities',$,$,(#3));
//! #3=IFCQUANTITYVOLUME('NetVolume',$,$,12.5,$);
//! #5=IFCRELDEFINESBYPROPERTIES('g5',$,$,$,(#1),#2);
//! ".to_string();
//!
//! let report = run_takeoff(content, vec![presets::wall_net_volume()])
//!     .run()
//!     .expect("takeoff failed");
//!
//! let vol = report.get("Wall Net Volume").expect("metric");
//! assert!((vol.value() - 12.5).abs() < 0.001);
//! ```
//!
//! ## Custom Metrics
//!
//! ```rust
//! use ifc_qto_cat::{MetricDefinition, Aggregation};
//!
//! let metric = MetricDefinition::new(
//!     "Slab Total Area".into(),
//!     vec!["IFCSLAB".into()],
//!     "Qto_SlabBaseQuantities".into(),
//!     "NetArea".into(),
//!     Aggregation::Sum,
//! );
//! ```
//!
//! ## Predefined Metrics
//!
//! The [`metric::presets`] module provides ready-made metrics for
//! common quantities:
//!
//! | Preset | Measures |
//! |--------|----------|
//! | `wall_net_volume()` | Net volume of all walls |
//! | `wall_gross_volume()` | Gross volume of all walls |
//! | `slab_net_volume()` | Net volume of all slabs |
//! | `space_net_floor_area()` | Net floor area of all spaces |
//! | `space_gross_floor_area()` | Gross floor area of all spaces |
//! | `door_count()` | Number of doors |
//! | `window_count()` | Number of windows |
//! | `door_total_area()` | Total door area |
//! | `column_net_volume()` | Net volume of all columns |
//!
//! ## Architecture
//!
//! 1. **Scan** the IFC content for entities (via `ifc-lite-core-cat`)
//! 2. **Index** entities by id for O(1) lookup
//! 3. **Map** `IfcRelDefinesByProperties` relations to build
//!    element-to-quantity-set associations
//! 4. **Extract** `IfcElementQuantity` / `IfcQuantity*` values
//! 5. **Aggregate** per metric definition (sum, count, avg, min, max)
//! 6. **Report** results as a structured [`TakeoffReport`]
//!
//! The entire pipeline is wrapped in a single
//! [`Io<Error, TakeoffReport>`](comp_cat_rs::effect::io::Io); call
//! `.run()` only at the boundary.

pub mod error;
pub mod extract;
pub mod metric;
pub mod quantity;
pub mod takeoff;

// ── Convenience re-exports ──────────────────────────────────────────

pub use error::{Error, Result};
pub use metric::{Aggregation, MetricDefinition};
pub use quantity::{Quantity, QuantityKind, QuantitySet};
pub use takeoff::{run_takeoff, TakeoffReport, TakeoffResult};
