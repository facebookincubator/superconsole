use crate::{content::LinesExt, Component, Dimensions, DrawMode, Lines, State};

/// Component that ensures its child component has at most `max_size` render space.
#[derive(Debug)]
pub struct Bounded {
    child: Box<dyn Component>,
    max_size: Dimensions,
}

impl Bounded {
    pub fn new(child: Box<dyn Component>, max_x: Option<usize>, max_y: Option<usize>) -> Self {
        Self {
            child,
            max_size: Dimensions {
                x: max_x.unwrap_or(usize::MAX),
                y: max_y.unwrap_or(usize::MAX),
            },
        }
    }
}

impl Component for Bounded {
    fn draw_unchecked(
        &self,
        state: &State,
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
    use crate::{components::Echo, Span};

    #[derive(AsRef, Debug)]
    struct Msg(Lines);

    #[test]
    fn test_no_bounding() -> anyhow::Result<()> {
        let test = Bounded::new(box Echo::<Msg>::new(false), Some(40), Some(40));
        let msg = Msg(vec![crate::line!(Span::new_unstyled("hello world")?)]);
        let output = test.draw(
            &crate::state![&msg],
            Dimensions { x: 50, y: 50 },
            DrawMode::Normal,
        )?;
        let expected = msg.0;

        assert_eq!(output, expected);

        Ok(())
    }

    #[test]
    fn test_bounding() -> anyhow::Result<()> {
        let test = Bounded::new(box Echo::<Msg>::new(false), Some(2), Some(1));
        let msg = Msg(vec![
            crate::line!(Span::new_unstyled("hello world")?),
            crate::line!(Span::new_unstyled("hello world")?),
        ]);
        let output = test.draw(
            &crate::state![&msg],
            Dimensions { x: 50, y: 50 },
            DrawMode::Normal,
        )?;
        let expected = vec![crate::line!(Span::new_unstyled("he")?)];

        assert_eq!(output, expected);

        Ok(())
    }
}