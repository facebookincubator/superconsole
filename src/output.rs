/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under both the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree and the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree.
 */

use std::{
    any::Any,
    io::{self, Write},
    thread::JoinHandle,
};

use anyhow::Context as _;
use crossbeam_channel::{bounded, Sender};

pub trait SuperConsoleOutput: Send + Sync + 'static {
    /// Called before rendering will occur. This has a chance to prevent rendering by returning
    /// false.
    fn should_render(&mut self) -> bool;

    /// Called to produce output. This may be called without should_render if we are finalizing or
    /// clearing. This should flush if possible.
    fn output(&mut self, buffer: Vec<u8>) -> anyhow::Result<()>;

    /// Called when the console has finalized. This must block if necessary. No further output will
    /// be emitted.
    fn finalize(self: Box<Self>) -> anyhow::Result<()>;

    /// Get this Output as an Any. This is used for testing.
    fn as_any(&self) -> &dyn Any;

    /// Get this Output as a mutable Any. This is used for testing.
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub(crate) struct BlockingSuperConsoleOutput;

impl SuperConsoleOutput for BlockingSuperConsoleOutput {
    fn should_render(&mut self) -> bool {
        true
    }

    fn output(&mut self, buffer: Vec<u8>) -> anyhow::Result<()> {
        let stderr = io::stderr();
        let mut handle = stderr.lock();
        handle.write_all(&buffer)?;
        handle.flush()?;
        Ok(())
    }

    fn finalize(self: Box<Self>) -> anyhow::Result<()> {
        Ok(())
    }


    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

pub(crate) struct NonBlockingSuperConsoleOutput {
    sender: Sender<Vec<u8>>,
    handle: JoinHandle<anyhow::Result<()>>,
}

impl NonBlockingSuperConsoleOutput {
    #[allow(unused)]
    pub fn new() -> anyhow::Result<Self> {
        let (sender, receiver) = bounded::<Vec<u8>>(1);

        let handle = std::thread::Builder::new()
            .name("superconsole-io".to_owned())
            .spawn(move || {
                for frame in receiver.into_iter() {
                    let stderr = io::stderr();
                    let mut handle = stderr.lock();
                    handle.write_all(&frame)?;
                    handle.flush()?;
                }
                Ok(())
            })
            .context("Error spawning Superconsole I/O thread")?;

        Ok(Self { sender, handle })
    }
}

impl SuperConsoleOutput for NonBlockingSuperConsoleOutput {
    /// Check if we have free capacity in our channel. Note that if the channel is full, that means
    /// our writer thread already has 2 buffered frames (one in the channel, one it's currently
    /// writing out). In this case, refuse to produce further output.
    fn should_render(&mut self) -> bool {
        !self.sender.is_full()
    }

    /// Attempt to send out a frame. If we called should_render, this won't block. If we didn't,
    /// then it may block.
    fn output(&mut self, buffer: Vec<u8>) -> anyhow::Result<()> {
        self.sender
            .send(buffer)
            .context("Superconsole I/O thread has crashed")?;
        Ok(())
    }

    /// Notify our writer thread that no further writes are expected. Wait for it to flush.
    fn finalize(self: Box<Self>) -> anyhow::Result<()> {
        let Self { sender, handle } = *self;
        drop(sender);
        match handle.join() {
            Ok(res) => res?,
            Err(panic) => std::panic::resume_unwind(panic),
        }
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
