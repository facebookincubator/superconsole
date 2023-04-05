/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under both the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree and the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree.
 */

use std::fmt::Debug;

use crate::content::LinesExt;
use crate::Component;
use crate::Dimensions;
use crate::DrawMode;
use crate::Lines;

/// Component that ensures its child component has at most `max_size` render space.
#[derive(Debug)]
pub struct Bounded<S> {
    child: Box<dyn Component<S>>,
    max_size: Dimensions,
}

impl<S> Bounded<S> {
    pub fn new(child: Box<dyn Component<S>>, max_x: Option<usize>, max_y: Option<usize>) -> Self {
        Self {
            child,
            max_size: Dimensions {
                width: max_x.unwrap_or(usize::MAX),
                height: max_y.unwrap_or(usize::MAX),
            },
        }
    }
}

impl<S: Debug> Component<S> for Bounded<S> {
    fn draw_unchecked<'a>(
        &self,
        state: &'a S,
        dimensions: Dimensions,
        mode: DrawMode,
    ) -> anyhow::Result<Lines> {
        let mut output = self
            .child
            .draw(state, dimensions.intersect(self.max_size), mode)?;
        output.shrink_lines_to_dimensions(self.max_size.intersect(dimensions));
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use derive_more::AsRef;

    use super::*;
    use crate::components::Echo;
    use crate::Line;
    use crate::Span;

    #[derive(AsRef, Debug)]
    struct Msg(Lines);

    #[test]
    fn test_no_bounding() -> anyhow::Result<()> {
        let test = Bounded::new(Box::new(Echo::<Msg>::new(false)), Some(40), Some(40));
        let msg = Msg(vec![Line::from_iter([Span::new_unstyled("hello world")?])]);
        let output = test.draw(
            &crate::state![&msg],
            Dimensions {
                width: 50,
                height: 50,
            },
            DrawMode::Normal,
        )?;
        let expected = msg.0;

        assert_eq!(output, expected);

        Ok(())
    }

    #[test]
    fn test_bounding() -> anyhow::Result<()> {
        let test = Bounded::new(Box::new(Echo::<Msg>::new(false)), Some(2), Some(1));
        let msg = Msg(vec![
            Line::from_iter([Span::new_unstyled("hello world")?]),
            Line::from_iter([Span::new_unstyled("hello world")?]),
        ]);
        let output = test.draw(
            &crate::state![&msg],
            Dimensions {
                width: 50,
                height: 50,
            },
            DrawMode::Normal,
        )?;
        let expected = vec![Line::from_iter([Span::new_unstyled("he")?])];

        assert_eq!(output, expected);

        Ok(())
    }
}
