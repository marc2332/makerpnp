use crate::pnp::part::Part;
use crate::part_mapper::criteria::PlacementMappingCriteria;

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug)]
pub struct PartMapping<'part>
{
    pub part: &'part Part,
    pub criteria: Vec<Box<dyn PlacementMappingCriteria>>,
}

impl<'part> PartMapping<'part> {
    pub fn new(part: &'part Part, criteria: Vec<Box<dyn PlacementMappingCriteria>>) -> Self {
        Self {
            part,
            criteria
        }
    }
}