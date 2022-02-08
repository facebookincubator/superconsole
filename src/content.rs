//! Provides a variety of utilities for working with [`Line`s](Line).
//! In order to work with [`Component`](crate::Component) output, one must import [`LinesExt`](LinesExt)

pub use line::Line;
pub use lines::{
    colored_lines_from_multiline_string, lines_from_multiline_string, Lines, LinesExt,
};
pub use span::Span;

mod line;
mod lines;
mod span;
