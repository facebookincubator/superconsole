/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under both the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree and the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree.
 */

use std::fmt::Debug;
use std::marker::PhantomData;

use crate::components::Dimensions;
use crate::components::DrawMode;
use crate::Component;
use crate::Line;

/// Component that repeats whatever lines are put into it.
/// Mostly useful for testing purposes.
#[derive(Debug)]
pub struct Echo {
    collapse: bool,
    indices: Option<(usize, usize)>,
    _state: PhantomData<Vec<Line>>,
}

impl Echo {
    pub fn new(collapse: bool) -> Self {
        Self {
            collapse,
            indices: None,
            _state: PhantomData,
        }
    }

    pub fn new_with_indices(collapse: bool, indices: (usize, usize)) -> Self {
        Self {
            collapse,
            indices: Some(indices),
            _state: PhantomData,
        }
    }
}

impl Component<Vec<Line>> for Echo {
    fn draw_unchecked(
        &self,
        state: &Vec<Line>,
        _dimensions: Dimensions,
        mode: DrawMode,
    ) -> anyhow::Result<Vec<Line>> {
        match mode {
            DrawMode::Final if self.collapse => Ok(vec![]),
            _ => Ok(if let Some((i, j)) = self.indices {
                state[i..j].to_vec()
            } else {
                state.to_owned()
            }),
        }
    }
}
