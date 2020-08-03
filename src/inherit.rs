use crate::block::{BasicBlock, BlockProps, DescribeProps, Test};

pub(crate) trait Inherit {
    fn inherit(&mut self, parent_props_props: &DescribeProps);
}

impl Inherit for DescribeProps {
    fn inherit(&mut self, parent_props: &DescribeProps) {
        self.block_props.inherit(parent_props);
        // Inherit parent_props's `use`s
        // TODO: Map each parent_props's use path to `super::last_segment`
        self.uses.extend(parent_props.uses.clone());

        if let Some(ref parent_props_before) = &parent_props.before {
            // Prepend parent_props's `before` code sequence
            let before = if let Some(ref self_before) = &self.before {
                parent_props_before
                    .0
                    .iter()
                    .chain(self_before.0.iter())
                    .cloned()
                    .collect()
            } else {
                parent_props_before.0.clone()
            };

            self.before = Some(BasicBlock(before));
        }

        if let Some(ref parent_props_after) = &parent_props.after {
            // Append parent_props's `after` code sequence
            if let Some(ref mut self_after) = &mut self.after {
                self_after.0.extend(parent_props_after.0.clone());
            } else {
                self.after = Some(parent_props_after.clone());
            }
        }
    }
}

impl Inherit for Test {
    fn inherit(&mut self, parent_props: &DescribeProps) {
        self.properties.inherit(parent_props);

        // Prepend parent_props's `before` code sequence
        if let Some(ref parent_props_before) = &parent_props.before {
            self.content = BasicBlock(
                parent_props_before
                    .0
                    .iter()
                    .chain(self.content.0.iter())
                    .cloned()
                    .collect()
            )
        }
        // Append parent_props's `after` code sequence
        if let Some(ref parent_props_after) = &parent_props.after {
            self.content.0.extend(parent_props_after.0.clone());
        }
    }
}

impl Inherit for BlockProps {
    fn inherit(&mut self, parent_props: &DescribeProps) {
        // Append parent_props's attributes
        self.attributes.extend(parent_props.block_props.attributes.clone());

        // If parent_props is async, so is self
        if !self.is_async && parent_props.block_props.is_async {
            self.is_async = true;
        }

        // If self doesn't have a return type, use its parent_props's
        if self.return_type.is_none() {
            self.return_type = parent_props.block_props.return_type.clone()
        }
    }
}
