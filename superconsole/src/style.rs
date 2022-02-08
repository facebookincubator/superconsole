//! Provides stylization for Strings.
//! - Create a styled string using `style`.
//! - Set the foreground or background color of the string using the `Color` enum.
//! - Set the attribute (bold, italic, underlined, etc) using the `Attribute` enum.

/// Re-export crossterm ideas about stylization to users of `superconsole`.
pub use crossterm::style::{style, Attribute, Color, ContentStyle, StyledContent, Stylize};
