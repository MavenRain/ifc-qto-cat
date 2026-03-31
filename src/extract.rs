//! Quantity extraction from decoded IFC entities.
//!
//! Traverses the `IfcRelDefinesByProperties` -> `IfcElementQuantity`
//! -> `IfcQuantity*` chain to collect quantity values for elements.

use std::collections::HashMap;

use ifc_lite_core_cat::attribute::AttributeValue;
use ifc_lite_core_cat::decode::decode_entity;
use ifc_lite_core_cat::scan::scan_next;
use ifc_lite_core_cat::EntityId;

use crate::error::Result;
use crate::quantity::{Quantity, QuantityKind, QuantitySet};

/// A map from element id to the list of property-definition ids
/// attached via `IfcRelDefinesByProperties`.
pub type RelationMap = HashMap<EntityId, Vec<EntityId>>;

/// Index mapping entity id to `(start, end)` byte offsets.
pub type EntityIndex = HashMap<EntityId, (usize, usize)>;

/// Build a [`RelationMap`] by scanning and decoding all
/// `IFCRELDEFINESBYPROPERTIES` entities in the content.
///
/// # IFC structure
///
/// ```text
/// #n = IFCRELDEFINESBYPROPERTIES(guid, owner, name, desc,
///          (related_objects...),   -- attr 4
///          relating_prop_def);     -- attr 5
/// ```
///
/// # Errors
///
/// Propagates decoding errors from the core parser.
pub fn build_relation_map(content: &str, index: &EntityIndex) -> Result<RelationMap> {
    let bytes = content.as_bytes();

    // Collect all IFCRELDEFINESBYPROPERTIES as (related_ids, prop_def_id).
    let pairs: Vec<(Vec<EntityId>, EntityId)> = std::iter::successors(
        scan_next(bytes, 0),
        #[allow(clippy::needless_borrows_for_generic_args)]
        |&(ref _e, pos)| scan_next(bytes, pos),
    )
    .filter(|(entity, _)| entity.ifc_type().name() == "IFCRELDEFINESBYPROPERTIES")
    .filter_map(|(entity, _)| {
        let (start, end) = index.get(&entity.id()).copied()?;
        let decoded = decode_entity(content, start, end).run().ok()?;

        // Attribute 4: RelatedObjects (list of entity refs)
        let related_ids: Vec<EntityId> = decoded
            .get(4)
            .and_then(AttributeValue::as_list)
            .map(|list| list.iter().filter_map(AttributeValue::as_entity_ref).collect())
            .unwrap_or_default();

        // Attribute 5: RelatingPropertyDefinition (entity ref)
        let prop_def_id = decoded.get_ref(5)?;

        Some((related_ids, prop_def_id))
    })
    .collect();

    // Group: element_id -> Vec<prop_def_id>.
    // fold accumulator mutation is confined to the fold scope.
    #[allow(clippy::needless_pass_by_value)]
    let map = pairs.into_iter().fold(
        HashMap::<EntityId, Vec<EntityId>>::new(),
        |map, (related_ids, prop_def_id)| {
            related_ids.into_iter().fold(map, |m, elem_id| {
                insert_grouped(m, elem_id, prop_def_id)
            })
        },
    );

    Ok(map)
}

/// Insert `value` into the `Vec` at `key`, returning the updated map.
///
/// Consumes and returns ownership to avoid external mutation.
fn insert_grouped(
    map: HashMap<EntityId, Vec<EntityId>>,
    key: EntityId,
    value: EntityId,
) -> HashMap<EntityId, Vec<EntityId>> {
    let entry = map.get(&key).cloned().unwrap_or_default();
    let updated = entry.into_iter().chain(std::iter::once(value)).collect();
    let replaced: HashMap<EntityId, Vec<EntityId>> = map
        .into_iter()
        .filter(|(k, _)| *k != key)
        .chain(std::iter::once((key, updated)))
        .collect();
    replaced
}

/// Extract all [`QuantitySet`]s attached to a given element.
///
/// Follows the `RelationMap` to find `IfcElementQuantity` entities,
/// then decodes each quantity within the set.
#[must_use]
pub fn extract_element_quantities(
    content: &str,
    index: &EntityIndex,
    rel_map: &RelationMap,
    element_id: EntityId,
) -> Vec<QuantitySet> {
    rel_map
        .get(&element_id)
        .map(|prop_def_ids| {
            prop_def_ids
                .iter()
                .filter_map(|&prop_def_id| decode_quantity_set(content, index, prop_def_id))
                .collect()
        })
        .unwrap_or_default()
}

/// Decode an `IfcElementQuantity` and its child quantities.
///
/// Returns `None` if the entity is not an `IfcElementQuantity` or
/// cannot be decoded.
fn decode_quantity_set(
    content: &str,
    index: &EntityIndex,
    entity_id: EntityId,
) -> Option<QuantitySet> {
    let &(start, end) = index.get(&entity_id)?;
    let decoded = decode_entity(content, start, end).run().ok()?;

    // Must be IFCELEMENTQUANTITY.
    (decoded.ifc_type().name() == "IFCELEMENTQUANTITY").then_some(())?;

    // Attribute 2: Name (e.g. "Qto_WallBaseQuantities")
    let set_name = decoded.get_string(2).unwrap_or("").to_string();

    // Attribute 5: Quantities (list of entity refs)
    let quantities: Vec<Quantity> = decoded
        .get(5)
        .and_then(AttributeValue::as_list)
        .map(|list| {
            list.iter()
                .filter_map(AttributeValue::as_entity_ref)
                .filter_map(|q_id| decode_single_quantity(content, index, q_id))
                .collect()
        })
        .unwrap_or_default();

    Some(QuantitySet::new(set_name, entity_id, quantities))
}

/// Decode a single `IfcQuantity*` entity into a [`Quantity`].
///
/// Supports `IfcQuantityLength`, `IfcQuantityArea`,
/// `IfcQuantityVolume`, `IfcQuantityWeight`, `IfcQuantityCount`,
/// `IfcQuantityTime`.
///
/// # IFC structure
///
/// ```text
/// #n = IFCQUANTITYLENGTH('Name', desc, unit, LengthValue, formula);
///                          0       1     2      3            4
/// ```
///
/// The numeric value is always at attribute index 3.
fn decode_single_quantity(
    content: &str,
    index: &EntityIndex,
    entity_id: EntityId,
) -> Option<Quantity> {
    let &(start, end) = index.get(&entity_id)?;
    let decoded = decode_entity(content, start, end).run().ok()?;

    let kind = QuantityKind::from_type_name(decoded.ifc_type().name())?;
    let name = decoded.get_string(0).unwrap_or("").to_string();
    let value = decoded.get_float(3).unwrap_or(0.0);

    Some(Quantity::new(name, kind, value))
}

// ═══════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use ifc_lite_core_cat::scan::build_entity_index;

    fn sample_ifc() -> &'static str {
        "\
#1=IFCWALL('g1',$,$,$,$,$,$,$);
#2=IFCELEMENTQUANTITY('g2',$,'Qto_WallBaseQuantities',$,$,(#3,#4));
#3=IFCQUANTITYLENGTH('Length',$,$,5.0,$);
#4=IFCQUANTITYVOLUME('NetVolume',$,$,12.5,$);
#5=IFCRELDEFINESBYPROPERTIES('g5',$,$,$,(#1),#2);
"
    }

    #[test]
    fn builds_relation_map() {
        let content = sample_ifc();
        let index = build_entity_index(content);
        let rel_map = build_relation_map(content, &index).expect("rel_map");
        let defs = rel_map.get(&EntityId::new(1)).expect("wall entry");
        assert!(defs.contains(&EntityId::new(2)));
    }

    #[test]
    fn extracts_element_quantities() {
        let content = sample_ifc();
        let index = build_entity_index(content);
        let rel_map = build_relation_map(content, &index).expect("rel_map");
        let qsets = extract_element_quantities(content, &index, &rel_map, EntityId::new(1));

        assert_eq!(qsets.len(), 1);
        assert_eq!(qsets[0].name(), "Qto_WallBaseQuantities");
        assert_eq!(qsets[0].quantities().len(), 2);

        let vol = qsets[0].get("NetVolume").expect("NetVolume");
        assert!((vol.value() - 12.5).abs() < 0.001);
        assert_eq!(vol.kind(), QuantityKind::Volume);

        let len = qsets[0].get("Length").expect("Length");
        assert!((len.value() - 5.0).abs() < 0.001);
        assert_eq!(len.kind(), QuantityKind::Length);
    }
}
