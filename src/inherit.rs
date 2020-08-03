use crate::block::{BasicBlock, BlockProperties, Describe, Test};

pub(crate) trait Inherit {
    fn inherit(&mut self, parent: &Describe);
}

impl Inherit for Describe {
    fn inherit(&mut self, parent: &Describe) {
        self.properties.inherit(parent);
        // Inherit parent's `use`s
        self.uses = parent
            .uses
            .iter()
            .chain(self.uses.iter())
            .cloned()
            .collect();

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
        let mut content = if let Some(ref parent_before) = &parent.before {
            parent_before.0.clone()
        } else {
            Vec::new()
        };
        // Add test's unique code sequence
        content.extend(self.content.0.clone());
        // Append parent's `after` code sequence
        let parent_after = if let Some(ref parent_after) = &parent.after {
            parent_after.0.clone()
        } else {
            Vec::new()
        };
        content.extend(parent_after);
        self.content = BasicBlock(content)
    }
}

impl Inherit for BlockProperties {
    fn inherit(&mut self, parent: &Describe) {
        // Prepend parent's attributes
        self.attributes = parent
            .properties
            .attributes
            .iter()
            .chain(self.attributes.iter())
            .cloned()
            .collect();

        // If parent is async, so is self
        if parent.properties.is_async {
            self.is_async = true;
        }

        // If self doesn't have a return type, copy its parent's
        if self.return_type.is_none() {
            self.return_type = parent.properties.return_type.clone()
        }
    }
}
