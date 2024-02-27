use std::{collections::VecDeque, iter::repeat_with};

use bevy::{ecs::component::Component, utils::default};
use rand::{seq::SliceRandom, thread_rng, SeedableRng};
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};
use tap::Tap;

use super::MinoKind;

#[derive(Component, Clone, Serialize, Deserialize, Debug)]
pub struct PieceQueue {
    window: VecDeque<MinoKind>,
    window_size: usize,
    rng: Pcg32,
}

impl Default for PieceQueue {
    fn default() -> Self {
        Self {
            window: default(),
            window_size: 5,
            rng: Pcg32::from_rng(thread_rng()).expect("could not construct an rng"),
        }
        .tap_mut(|a| a.refill_window())
    }
}

// TODO should not assume that there will be a piece in the queue
impl PieceQueue {
    pub fn window(&self) -> &VecDeque<MinoKind> {
        &self.window
    }

    pub fn peek(&mut self) -> MinoKind {
        *self.window.front().unwrap()
    }

    pub fn take(&mut self) -> MinoKind {
        let ret = self.window.pop_front().unwrap();
        self.refill_window();
        ret
    }

    fn refill_window(&mut self) {
        if self.window_size > self.window.len() {
            let bags_needed = (self.window_size - self.window.len() + 6) / 7;
            use MinoKind::*;
            self.window.extend(
                repeat_with(|| [Z, S, T, L, J, I, O].tap_mut(|s| s.shuffle(&mut self.rng)))
                    .take(bags_needed)
                    .flatten(),
            )
        }
    }
}
