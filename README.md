# ifc-qto-cat

Data-driven quantity takeoffs from IFC models, built on
[comp-cat-rs](https://github.com/MavenRain/comp-cat-rs) and
[ifc-lite-core-cat](https://crates.io/crates/ifc-lite-core-cat).

Inspired by [QTO Buccaneer](https://github.com/simondilhas/qto_buccaneer).

## Overview

Extract and aggregate standardised IFC quantities (length, area,
volume, weight, count) from building elements.  Define metrics
declaratively and run them against any IFC file.

## Quick Start

```rust
use ifc_qto_cat::{run_takeoff, metric::presets};

let content = std::fs::read_to_string("model.ifc")
    .expect("failed to read IFC file");

let metrics = vec![
    presets::wall_net_volume(),
    presets::slab_net_volume(),
    presets::space_net_floor_area(),
    presets::door_count(),
    presets::window_count(),
];

let report = run_takeoff(content, metrics)
    .run()
    .expect("takeoff failed");

println!("{report}");
```

## Custom Metrics

```rust
use ifc_qto_cat::{MetricDefinition, Aggregation};

let metric = MetricDefinition::new(
    "Total Beam Length".into(),
    vec!["IFCBEAM".into()],
    "Qto_BeamBaseQuantities".into(),
    "Length".into(),
    Aggregation::Sum,
);
```

## Predefined Metrics

| Preset | Measures |
|--------|----------|
| `wall_net_volume()` | Net volume of all walls |
| `wall_gross_volume()` | Gross volume of all walls |
| `slab_net_volume()` | Net volume of all slabs |
| `space_net_floor_area()` | Net floor area of all spaces |
| `space_gross_floor_area()` | Gross floor area of all spaces |
| `door_count()` | Number of doors |
| `window_count()` | Number of windows |
| `door_total_area()` | Total door area |
| `column_net_volume()` | Net volume of all columns |

## Aggregation Methods

| Method | Description |
|--------|-------------|
| `Sum` | Sum all values |
| `Count` | Count matching elements |
| `Average` | Arithmetic mean |
| `Min` | Minimum value |
| `Max` | Maximum value |

## IFC Quantity Sets

The library understands standard IFC4.3 quantity sets:

- `Qto_WallBaseQuantities` -- Length, Width, Height, GrossFootPrintArea, NetFootPrintArea, GrossVolume, NetVolume
- `Qto_SlabBaseQuantities` -- Length, Width, Depth, Area, NetArea, GrossVolume, NetVolume
- `Qto_ColumnBaseQuantities` -- Length, CrossSectionArea, OuterSurfaceArea, GrossVolume, NetVolume
- `Qto_DoorBaseQuantities` -- Width, Height, Area, Perimeter
- `Qto_SpaceBaseQuantities` -- Height, GrossFloorArea, NetFloorArea, GrossVolume, NetVolume
- `Qto_BuildingBaseQuantities` -- Height, GrossFloorArea, NetFloorArea, GrossVolume

## Architecture

```text
IFC Content
    |
    v
 scan_next()        -- find entities (ifc-lite-core-cat)
    |
    v
 build_entity_index()  -- O(1) lookup by id
    |
    v
 build_relation_map()  -- IfcRelDefinesByProperties chain
    |
    v
 extract_element_quantities()  -- IfcElementQuantity -> IfcQuantity*
    |
    v
 aggregate()           -- sum / count / avg / min / max
    |
    v
 TakeoffReport         -- structured results
```

The entire pipeline is wrapped in `Io<Error, TakeoffReport>`;
call `.run()` only at the boundary.

## License

Licensed under either of

- Apache License, Version 2.0
- MIT license

at your option.
