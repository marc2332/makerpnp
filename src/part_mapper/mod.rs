pub mod criteria;
pub mod part_mapping;

use std::fmt::{Display, Formatter};
use crate::eda::eda_placement::EdaPlacement;
use crate::part_mapper::part_mapping::PartMapping;
use crate::pnp::load_out_item::LoadOutItem;

pub struct PartMapper {}

impl PartMapper {
    pub fn process<'placement, 'mapping>(
        eda_placements: &'placement Vec<EdaPlacement>,
        part_mappings: &'mapping Vec<PartMapping<'mapping>>,
        load_out_items: &Vec<LoadOutItem>
    ) -> Result<Vec<PlacementPartMappingResult<'placement, 'mapping>>, PartMapperError<'placement, 'mapping>> {

        let mut error_count: usize = 0;
        let mut mappings = vec![];

        for eda_placement in eda_placements.iter() {
            let mut part_mapping_results = vec![];

            for part_mapping in part_mappings.iter() {
                for criteria in part_mapping.criteria.iter() {
                    if criteria.matches(eda_placement) {
                        part_mapping_results.push(PartMappingResult { part_mapping, applied_rule: None });
                    }
                }
            }

            let matching_mappings_result = apply_rules(part_mapping_results, load_out_items)
                .inspect_err(|_|error_count += 1);

            let result = PlacementPartMappingResult { eda_placement, mapping_result: matching_mappings_result };
            mappings.push(result);
        }

        match error_count {
            0 => Ok(mappings),
            1.. => Err(PartMapperError::MappingErrors(mappings))
        }

    }
}

fn apply_rules<'mapping>(mut mapping_result: Vec<PartMappingResult<'mapping>>, load_out_items: &Vec<LoadOutItem>) -> Result<Vec<PartMappingResult<'mapping>>, PartMappingError<'mapping>> {

    let count = mapping_result.len();
    if count == 0 {
        return Err(PartMappingError::NoMappings)
    }

    if count == 1 {
        mapping_result[0].applied_rule = Some(AppliedMappingRule::AutoSelected);
        return Ok(mapping_result)
    }

    let mut mapping_results_matched_to_loadout_items: Vec<(&mut PartMappingResult, &LoadOutItem)> = mapping_result.iter_mut().filter_map(|mapping_result| {
        let thing = load_out_items.iter().find(|item| {
            let mapped_part = mapping_result.part_mapping;
            (item.mpn == mapped_part.part.mpn)
                && (item.manufacturer == mapped_part.part.manufacturer)
        });

        match thing {
            Some(loadout_item) => Some((mapping_result, loadout_item)),
            None => None,
        }
    }).collect();

    match mapping_results_matched_to_loadout_items.len() {
        0 => Err(PartMappingError::MultipleMatchingMappings(mapping_result)),
        1 => {
            let load_out_item = &mapping_results_matched_to_loadout_items[0].1;
            mapping_results_matched_to_loadout_items[0].0.applied_rule = Some(AppliedMappingRule::FoundInLoadOut(load_out_item.reference.clone()));
            Ok(mapping_result)
        },
        2.. => {
            todo!()
        }
    }
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug)]
pub enum PartMapperError<'placement, 'mapping> {
    MappingErrors(Vec<PlacementPartMappingResult<'placement, 'mapping>>),
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug)]
pub enum PartMappingError<'mapping> {
    MultipleMatchingMappings(Vec<PartMappingResult<'mapping>>),
    NoMappings,
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug)]
pub enum AppliedMappingRule {
    AutoSelected,
    FoundInLoadOut(String),
}

impl Display for AppliedMappingRule {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AppliedMappingRule::AutoSelected => write!(f, "Auto-selected"),
            AppliedMappingRule::FoundInLoadOut(reference) => write!(f, "Found in load-out, reference: '{}'", reference),
        }
    }
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug)]
pub struct PartMappingResult<'mapping> {
    pub part_mapping: &'mapping PartMapping<'mapping>,
    pub applied_rule: Option<AppliedMappingRule>,
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug)]
pub struct PlacementPartMappingResult<'placement, 'mapping> {
    pub eda_placement: &'placement EdaPlacement,
    pub mapping_result: Result<Vec<PartMappingResult<'mapping>>, PartMappingError<'mapping>>,
}

#[cfg(test)]
mod tests {
    use EdaPlacementDetails::DipTrace;
    use crate::pnp::part::Part;
    use crate::eda::diptrace::criteria::ExactMatchCriteria;
    use crate::eda::eda_placement::{DipTracePlacementDetails, EdaPlacement, EdaPlacementDetails};
    use crate::part_mapper::part_mapping::PartMapping;
    use crate::part_mapper::{AppliedMappingRule, PartMapper, PartMapperError, PartMappingError, PartMappingResult, PlacementPartMappingResult};

    #[test]
    fn map_parts() {
        // given
        let eda_placement1 = EdaPlacement { ref_des: "R1".to_string(), details: DipTrace(DipTracePlacementDetails { name: "NAME1".to_string(), value: "VALUE1".to_string() }) };
        let eda_placement2 = EdaPlacement { ref_des: "R2".to_string(), details: DipTrace(DipTracePlacementDetails { name: "NAME2".to_string(), value: "VALUE2".to_string() }) };
        let eda_placement3 = EdaPlacement { ref_des: "R3".to_string(), details: DipTrace(DipTracePlacementDetails { name: "NAME3".to_string(), value: "VALUE3".to_string() }) };

        let eda_placements = vec![eda_placement1, eda_placement2, eda_placement3];

        // and
        let part1 = Part::new("MFR1".to_string(), "PART1".to_string());
        let part2 = Part::new("MFR2".to_string(), "PART2".to_string());
        let part3 = Part::new("MFR3".to_string(), "PART3".to_string());

        let parts = vec![part1, part2, part3];

        // and
        let criteria1 = ExactMatchCriteria::new("NAME1".to_string(), "VALUE1".to_string());
        let part_mapping1 = PartMapping::new(&parts[1 - 1], vec![Box::new(criteria1)]);
        let criteria2 = ExactMatchCriteria::new("NAME2".to_string(), "VALUE2".to_string());
        let part_mapping2 = PartMapping::new(&parts[2 - 1], vec![Box::new(criteria2)]);
        let criteria3 = ExactMatchCriteria::new("NAME3".to_string(), "VALUE3".to_string());
        let part_mapping3 = PartMapping::new(&parts[3 - 1], vec![Box::new(criteria3)]);

        let part_mappings = vec![part_mapping1, part_mapping2, part_mapping3];

        // and
        let expected_results = Ok(vec![
            PlacementPartMappingResult { eda_placement: &eda_placements[0], mapping_result: Ok(vec![PartMappingResult { part_mapping: &part_mappings[0], applied_rule: Some(AppliedMappingRule::AutoSelected) }]) },
            PlacementPartMappingResult { eda_placement: &eda_placements[1], mapping_result: Ok(vec![PartMappingResult { part_mapping: &part_mappings[1], applied_rule: Some(AppliedMappingRule::AutoSelected) }]) },
            PlacementPartMappingResult { eda_placement: &eda_placements[2], mapping_result: Ok(vec![PartMappingResult { part_mapping: &part_mappings[2], applied_rule: Some(AppliedMappingRule::AutoSelected) }]) },
        ]);

        // when
        let matched_mappings = PartMapper::process(&eda_placements, &part_mappings, &vec![]);

        // then
        assert_eq!(matched_mappings, expected_results);
    }

    #[test]
    fn map_parts_with_multiple_matching_mappings() {
        // given
        let eda_placement1 = EdaPlacement { ref_des: "R1".to_string(), details: DipTrace(DipTracePlacementDetails { name: "NAME1".to_string(), value: "VALUE1".to_string() }) };

        let eda_placements = vec![eda_placement1];

        // and
        let part1 = Part::new("MFR1".to_string(), "PART1".to_string());
        let part2 = Part::new("MFR2".to_string(), "PART2".to_string());

        let parts = vec![part1, part2];

        // and
        let criteria1 = ExactMatchCriteria::new("NAME1".to_string(), "VALUE1".to_string());
        let part_mapping1 = PartMapping::new(&parts[1 - 1], vec![Box::new(criteria1)]);
        let criteria2 = ExactMatchCriteria::new("NAME1".to_string(), "VALUE1".to_string());
        let part_mapping2 = PartMapping::new(&parts[2 - 1], vec![Box::new(criteria2)]);

        let part_mappings = vec![part_mapping1, part_mapping2];

        // and
        let expected_results = Err(PartMapperError::MappingErrors(vec![
            PlacementPartMappingResult {
                eda_placement: &eda_placements[0],
                mapping_result: Err(PartMappingError::MultipleMatchingMappings(vec![
                    PartMappingResult { part_mapping: &part_mappings[0], applied_rule: None },
                    PartMappingResult { part_mapping: &part_mappings[1], applied_rule: None },
                ]))
            },
        ]));

        // when
        let matched_mappings = PartMapper::process(&eda_placements, &part_mappings, &vec![]);

        // then
        assert_eq!(matched_mappings, expected_results);
    }

    #[test]
    fn map_parts_with_no_part_mappings() {
        // given
        let eda_placement1 = EdaPlacement { ref_des: "R1".to_string(), details: DipTrace(DipTracePlacementDetails { name: "NAME1".to_string(), value: "VALUE1".to_string() }) };

        let eda_placements = vec![eda_placement1];

        let part_mappings = vec![];

        // and
        let expected_results = Err(PartMapperError::MappingErrors(vec![
            PlacementPartMappingResult {
                eda_placement: &eda_placements[0],
                mapping_result: Err(PartMappingError::NoMappings)
            },
        ]));

        // when
        let matched_mappings = PartMapper::process(&eda_placements, &part_mappings, &vec![]);

        // then
        assert_eq!(matched_mappings, expected_results);
    }
}