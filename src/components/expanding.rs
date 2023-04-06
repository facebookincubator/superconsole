/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under both the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree and the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree.
 */

use std::cell::Cell;
use std::fmt::Debug;

use crate::content::LinesExt;
use crate::Component;
use crate::Dimensions;

/// A `Component` which refuses to shrink below it's previous maximum size.
/// Notably, this component implicitly pads to a rectangle for simplicity.
#[derive(Debug)]
pub struct Expanding<S> {
    child: Box<dyn Component<S>>,
    maximum: Cell<Dimensions>,
}

impl<S> Expanding<S> {
    pub fn new(child: Box<dyn Component<S>>) -> Self {
        Self {
            child,
            maximum: Cell::default(),
        }
    }
}

impl<S: Debug> Component<S> for Expanding<S> {
    fn draw_unchecked<'a>(
        &self,
        state: &'a S,
        dimensions: crate::Dimensions,
        mode: crate::DrawMode,
    ) -> anyhow::Result<Vec<crate::Line>> {
        // Remember the new max at each step and pad out as necessary.
        let mut inner = self.child.draw(state, dimensions, mode)?;
        let cur_max = self.maximum.get();
        let new_max = cur_max.union(inner.dimensions()?);
        self.maximum.set(new_max);
        inner.set_lines_to_exact_dimensions(new_max.intersect(dimensions));

        Ok(inner)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::Echo;
    use crate::content::LinesExt;
    use crate::DrawMode;
    use crate::Line;

    #[test]
    fn test() -> anyhow::Result<()> {
        let expander = Expanding::new(Box::new(Echo::new(false)));
        let dims = Dimensions {
            width: 20,
            height: 20,
        };

        let longest_line: Line = vec!["Hello world"].try_into()?;
        let state = vec![longest_line.clone(), vec!["foobar"].try_into()?];
        let result = expander.draw(&state, dims, DrawMode::Normal)?;
        let expected = {
            let mut expected = state;
            expected.justify();
            expected
        };
        assert_eq!(result, expected);

        // testing horizontal
        let now_longest: Line = vec!["foobar"].try_into()?;
        let state = vec![vec!["H"].try_into()?, now_longest.clone()];
        let result = expander.draw(&state, dims, DrawMode::Normal)?;
        let expected = {
            let mut expected = state;
            expected.pad_lines_right(longest_line.len() - now_longest.len());
            expected
        };
        assert_eq!(result, expected);

        // testing vertical
        let state = vec![now_longest];
        let result = expander.draw(&state, dims, DrawMode::Normal)?;
        let expected = {
            let mut expected = state;
            expected.set_lines_to_exact_dimensions(Dimensions {
                width: longest_line.len(),
                height: 2,
            });
            expected
        };
        assert_eq!(result, expected);

        Ok(())
    }
}
