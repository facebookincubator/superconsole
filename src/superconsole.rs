use std::{cmp, io, io::Write};

use crossterm::{
    queue,
    terminal::{self, Clear, ClearType},
    tty::IsTty,
};

use crate::{
    components::{Canvas, Component, DrawMode},
    content::{Line, LinesExt},
    Dimensions, Lines, State,
};

const MINIMUM_EMIT: usize = 5;
const MAX_GRAPHEME_BUFFER: usize = 1000000;

/// Handles rendering the console using the user-defined [Component](Component)s and emitted messages.
/// A Canvas area at the bottom of the terminal is re-rendered in place at each tick for the components,
/// while a log area of emitted messages is produced above.
/// Producing output from sources other than SuperConsole while break the TUI.
#[derive(Default)]
pub struct SuperConsole {
    root: Canvas,
    to_emit: Vec<Line>,
    // A default screen size to use if the size cannot be fetched
    // from the terminal. This generally is only used for testing
    // situations.
    default_size: Option<Dimensions>,
}

impl SuperConsole {
    /// Build a new SuperConsole with a root component.
    pub fn new(root: Box<dyn Component>) -> Option<Self> {
        Self::compatible().then(|| Self {
            root: Canvas::new(root),
            ..Default::default()
        })
    }

    /// Force a new SuperConsole to be built with a root component, regardless of
    /// whether the tty is compatible
    pub fn forced_new(root: Box<dyn Component>, default_size: Dimensions) -> Self {
        Self {
            root: Canvas::new(root),
            default_size: Some(default_size),
            ..Default::default()
        }
    }

    pub fn compatible() -> bool {
        io::stdout().is_tty() && io::stderr().is_tty()
    }

    /// Render at a given tick.  Draws all components and drains the emitted events buffer.
    /// This will produce any pending emitting events above the Canvas and will re-render the drawing area.
    pub fn render(&mut self, state: &State) -> anyhow::Result<()> {
        // `render_general` refuses to drain more than a single frame, so repeat until done.
        // or until the rendered frame is too large to print anything.
        let mut anything_emitted = true;
        let mut has_rendered = false;
        while !has_rendered || (anything_emitted && !self.to_emit.is_empty()) {
            let last_len = self.to_emit.len();
            self.render_with_mode(state, DrawMode::Normal)?;
            anything_emitted = last_len == self.to_emit.len();
            has_rendered = true;
        }

        Ok(())
    }

    /// Perform a final render.
    /// This time, each component will have a chance to finalize themselves before the terminal is disposed of.
    pub fn finalize(&mut self, state: &State) -> anyhow::Result<()> {
        self.render_with_mode(state, DrawMode::Final)
    }

    /// Convenience method:
    /// - Calls queue_emit to add the lines.
    /// - Next, re-renders the `superconsole`.
    ///
    /// Because this re-renders the console, it requires passed state.
    /// Overuse of this method can cause `superconsole` to use significant CPU.
    pub fn emit_now(&mut self, lines: Lines, state: &State) -> anyhow::Result<()> {
        self.emit(lines);
        self.render(state)
    }

    /// Queues the passed lines to be drawn on the next render.
    /// The lines *will not* appear until the next render is called.
    pub fn emit(&mut self, mut lines: Lines) {
        self.to_emit.append(&mut lines);
    }

    fn size(&self) -> anyhow::Result<Dimensions> {
        match terminal::size() {
            Ok(size) => Ok(size.into()),
            Err(e) => match self.default_size {
                Some(default) => Ok(default),
                None => Err(e.into()),
            },
        }
    }

    /// Clears the canvas portion of the superconsole.
    pub fn clear(&mut self) -> anyhow::Result<()> {
        let mut writer = vec![];
        self.root.clear(&mut writer)?;
        Self::send_to_tty(&writer)
    }

    fn send_to_tty(buffer: &[u8]) -> anyhow::Result<()> {
        // the lock (and the flush) are probably unnecessary, but they don't hurt.
        let stderr = io::stderr();
        let mut handle = stderr.lock();
        handle.write_all(buffer)?;
        handle.flush()?;

        Ok(())
    }

    /// Helper method to share render + finalize behavior by specifying mode.
    fn render_with_mode(&mut self, state: &State, mode: DrawMode) -> anyhow::Result<()> {
        // TODO(cjhopman): We may need to try to keep each write call to be under the pipe buffer
        // size so it can be completed in a single syscall otherwise we might see a partially
        // rendered frame.
        let size = self.size()?;
        let mut buffer = Vec::new();

        self.render_general(&mut buffer, state, mode, size)?;
        Self::send_to_tty(&buffer)
    }

    /// Helper method that makes rendering highly configurable.
    fn render_general(
        &mut self,
        buffer: &mut Vec<u8>,
        state: &State,
        mode: DrawMode,
        size: Dimensions,
    ) -> anyhow::Result<()> {
        /// Heuristic to determine if a buffer is too large to buffer.
        /// Can be tuned, but is currently set to 1000000 graphemes.
        #[allow(clippy::ptr_arg)]
        fn is_big(buf: &Lines) -> bool {
            let len: usize = buf.iter().map(Line::len).sum();
            len > MAX_GRAPHEME_BUFFER
        }

        // Go the beginning of the canvas.
        self.root.move_up(buffer)?;

        // Pre-draw the frame *and then* start rendering emitted messages.
        let mut frame = self.root.draw(state, size, mode)?;
        // Render at most a single frame if this not the last render.
        // Does not buffer if there is a ridiculous amount of data.
        let limit = match mode {
            DrawMode::Normal if !is_big(&self.to_emit) => {
                let limit = (size.y as usize).saturating_sub(frame.len());
                // arbitrary value picked so we don't starve `emit` on small terminal sizes.
                Some(cmp::max(limit, MINIMUM_EMIT))
            }
            _ => None,
        };
        self.to_emit.render(buffer, limit)?;
        frame.render(buffer, None)?;

        // clear any residue from the previous render.
        queue!(buffer, Clear(ClearType::FromCursorDown))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryInto;

    use derive_more::AsRef;

    use super::*;
    use crate::{components::Echo, Lines};

    #[test]
    // Note: this test cannot be run without a terminal.
    fn test_small_buffer() {
        #[derive(AsRef, Debug)]
        struct Msg(Lines);

        let root = box Echo::<Msg>::new(false);
        let mut console = match SuperConsole::new(root) {
            Some(console) => console,
            // Return early if this test is run from CI
            None => return,
        };
        let msg_count = MINIMUM_EMIT + 5;
        console.emit(vec![vec!["line 1"].try_into().unwrap(); msg_count]);
        let msg = Msg(vec![vec!["line"].try_into().unwrap(); msg_count]);
        let state = crate::state![&msg];
        let mut buffer = Vec::new();

        // even though the canvas is larger than the tty
        console
            .render_general(
                &mut buffer,
                &state,
                DrawMode::Normal,
                Dimensions::new(100, 2),
            )
            .unwrap();

        // we should still drain a minimum of 5 messages.
        assert_eq!(console.to_emit.len(), msg_count - MINIMUM_EMIT);
    }

    #[test]
    // Note: this test cannot be run without a terminal.
    fn test_huge_buffer() {
        #[derive(AsRef, Debug)]
        struct Msg(Lines);

        let root = box Echo::<Msg>::new(false);
        let mut console = match SuperConsole::new(root) {
            Some(console) => console,
            // Return early if this test is run from CI
            None => return,
        };
        console.emit(vec![
            vec!["line 1"].try_into().unwrap();
            MAX_GRAPHEME_BUFFER * 2
        ]);
        let msg = Msg(vec![vec!["line"].try_into().unwrap(); 1]);
        let state = crate::state![&msg];
        let mut buffer = Vec::new();

        // Even though we have more messages than fit on the screen in the `to_emit` buffer
        console
            .render_general(
                &mut buffer,
                &state,
                DrawMode::Normal,
                Dimensions::new(100, 20),
            )
            .unwrap();

        // We have so many that we should just drain them all.
        assert!(console.to_emit.is_empty());
    }
}
