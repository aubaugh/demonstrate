//! Defines the inheritance behavior of `Describe` and `Test` block properties

use crate::block::{BasicBlock, BlockProps, Describe, DescribeProps, Test};

/// The trait and respective function for inheriting the parent `Describe` block's properties
pub(crate) trait Inherit {
    fn inherit(&mut self, parent_props: &DescribeProps);
}

impl Inherit for Describe {
    fn inherit(&mut self, parent_props: &DescribeProps) {
        // Inherit the `BlockProps` shared with `Test` blocks
        self.properties.block_props.inherit(parent_props);

        // Inherit `use` paths from parent
        // TODO: Map each parent_props's use path to `super::last_segment`
        self.properties.uses.extend(parent_props.uses.clone());

        // Inherit `before` code sequences from parent
        if let Some(ref parent_props_before) = &parent_props.before {
            // Prepend parent_props's `before` code sequence
            let before = if let Some(ref self_before) = &self.properties.before {
                parent_props_before
                    .0
                    .iter()
                    .chain(self_before.0.iter())
                    .cloned()
                    .collect()
            } else {
                parent_props_before.0.clone()
            };

            self.properties.before = Some(BasicBlock(before));
        }

        // Inherit `after` code sequences from parent
        if let Some(ref parent_props_after) = &parent_props.after {
            // Append parent_props's `after` code sequence
            if let Some(ref mut self_after) = &mut self.properties.after {
                self_after.0.extend(parent_props_after.0.clone());
            } else {
                self.properties.after = Some(parent_props_after.clone());
            }
        }
    }
}

impl Inherit for Test {
    fn inherit(&mut self, parent_props: &DescribeProps) {
        // Inherit the `BlockProps` shared with `Describe` blocks
        self.properties.inherit(parent_props);

        // Append `before` code sequence from parent
        if let Some(ref parent_props_before) = &parent_props.before {
            self.content = BasicBlock(
                parent_props_before
                    .0
                    .iter()
                    .chain(self.content.0.iter())
                    .cloned()
                    .collect(),
            )
        }

        // Append `after` code sequence from parent
        if let Some(ref parent_props_after) = &parent_props.after {
            self.content.0.extend(parent_props_after.0.clone());
        }
    }
}

impl Inherit for BlockProps {
    fn inherit(&mut self, parent_props: &DescribeProps) {
        // Append attributes from parent
        self.attributes
            .extend(parent_props.block_props.attributes.clone());

        // If parent is async, so is self
        if !self.is_async && parent_props.block_props.is_async {
            self.is_async = true;
        }

        // If self doesn't have a return type, use its parent's
        if self.return_type.is_none() {
            self.return_type = parent_props.block_props.return_type.clone()
        }
    }
}
