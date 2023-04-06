/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under both the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree and the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree.
 */

use std::fmt::Debug;
use std::io;
use std::io::Write;

use crate::components::Component;
use crate::output::BlockingSuperConsoleOutput;
use crate::output::NonBlockingSuperConsoleOutput;
use crate::output::SuperConsoleOutput;
use crate::Dimensions;
use crate::SuperConsole;

/// A builder to create SuperConsole, with more options.
pub struct Builder {
    non_blocking: bool,
    stream: Box<dyn Write + Send + 'static + Sync>,
}

impl Default for Builder {
    fn default() -> Self {
        Builder::new()
    }
}

impl Builder {
    pub fn new() -> Self {
        Self {
            non_blocking: false,
            stream: Box::new(io::stderr()),
        }
    }

    /// Enable non-blocking I/O.
    pub fn non_blocking(&mut self) -> &mut Self {
        self.non_blocking = true;
        self
    }

    /// Write to a different I/O
    pub fn write_to(&mut self, stream: Box<dyn Write + Send + 'static + Sync>) -> &mut Self {
        self.stream = stream;
        self
    }

    /// Build a new SuperConsole if stderr is a TTY.
    pub fn build<S: Debug>(
        self,
        root: Box<dyn Component<S>>,
    ) -> anyhow::Result<Option<SuperConsole<S>>> {
        if !SuperConsole::<S>::compatible() {
            return Ok(None);
        }
        Some(self.build_inner(root, None)).transpose()
    }

    /// Build a new SuperConsole regardless of whether stderr is a TTY.
    pub fn build_forced<S: Debug>(
        self,
        root: Box<dyn Component<S>>,
        fallback_size: Dimensions,
    ) -> anyhow::Result<SuperConsole<S>> {
        self.build_inner(root, Some(fallback_size))
    }

    fn build_inner<S: Debug>(
        self,
        root: Box<dyn Component<S>>,
        fallback_size: Option<Dimensions>,
    ) -> anyhow::Result<SuperConsole<S>> {
        Ok(SuperConsole::new_internal(
            root,
            fallback_size,
            self.output()?,
        ))
    }

    fn output(self) -> anyhow::Result<Box<dyn SuperConsoleOutput>> {
        if self.non_blocking {
            Ok(Box::new(NonBlockingSuperConsoleOutput::new(self.stream)?))
        } else {
            Ok(Box::new(BlockingSuperConsoleOutput::new(self.stream)))
        }
    }
}
