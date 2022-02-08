//! The superconsole crate provides a handler and building blocks for powerful, yet minimally intrusive TUIs.
//! Built on-top of [`crossterm`](crossterm), it cross-compiles on Windows 7+, Linux, and MacOS.
//!
//! Rendering is handled by [`SuperConsole`](SuperConsole), which draws to [`stdout`](std::io::stdout).
//! The caller is responsible for re-rendering whenever necessary.
//! User input will cause aberrations in output; similarly, one should also not produce output from other sources while superconsole is active.
//!
//! The rendering can be divided into two principle components:
//! * In the *scratch* area, the previous content is overwritten at each render.
//! * In the *emitted* area, lines scroll away above the scratch with various diagnostic output.
//! Components live in the scratch area.
//!
//! [`State`](State) and [`Component`s](Component) are decoupled.  `Component`s are stateless, and `State` is supplied at render time.
//!
//! A set of pre-baked composition and testing oriented components are provided in the [`components`](components) module.

#![cfg_attr(feature = "custom_linter", feature(plugin))]
#![cfg_attr(feature = "custom_linter", allow(deprecated))] // :(
#![cfg_attr(feature = "custom_linter", plugin(gazebo_lint))]
#![feature(box_syntax)]

// re-exports
pub use components::{Component, DrawMode};
pub use content::{Line, Lines, Span};
pub use dimensions::{Dimensions, Direction};
pub use error::Error;
pub use state::State;

pub use crate::superconsole::SuperConsole;

pub mod components;
pub mod content;
mod dimensions;
mod error;
mod state;
pub mod style;
mod superconsole;
