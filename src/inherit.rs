use crate::block::{BasicBlock, BlockProperties, Describe, Test};

pub(crate) trait Inherit {
    fn inherit(&mut self, parent: &Describe);
}

impl Inherit for Describe {
    fn inherit(&mut self, parent: &Describe) {
        self.properties.inherit(parent);
        // Inherit parent's `use`s
        // TODO: Map each parent's use path to `super::last_segment`
        self.uses.extend(parent.uses.clone());

        if let Some(ref parent_before) = &parent.before {
            // Prepend parent's `before` code sequence
            let before = if let Some(ref self_before) = &self.before {
                parent_before
                    .0
                    .iter()
                    .chain(self_before.0.iter())
                    .cloned()
                    .collect()
            } else {
                parent_before.0.clone()
            };

            self.before = Some(BasicBlock(before));
        }

        if let Some(ref parent_after) = &parent.after {
            // Append parent's `after` code sequence
            if let Some(ref mut self_after) = &mut self.after {
                self_after.0.extend(parent_after.0.clone());
            } else {
                self.after = Some(parent_after.clone());
            }
        }
    }
}

impl Inherit for Test {
    fn inherit(&mut self, parent: &Describe) {
        self.properties.inherit(parent);

        // Prepend parent's `before` code sequence
        if let Some(ref parent_before) = &parent.before {
            self.content = BasicBlock(
                parent_before
                    .0
                    .iter()
                    .chain(self.content.0.iter())
                    .cloned()
                    .collect()
            )
        }
        // Append parent's `after` code sequence
        if let Some(ref parent_after) = &parent.after {
            self.content.0.extend(parent_after.0.clone());
        }
    }
}

impl Inherit for BlockProperties {
    fn inherit(&mut self, parent: &Describe) {
        // Append parent's attributes
        self.attributes.extend(parent.properties.attributes.clone());

        // If parent is async, so is self
        if !self.is_async && parent.properties.is_async {
            self.is_async = true;
        }

        // If self doesn't have a return type, use its parent's
        if self.return_type.is_none() {
            self.return_type = parent.properties.return_type.clone()
        }
    }
}
