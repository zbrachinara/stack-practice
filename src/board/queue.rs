use std::{collections::VecDeque, iter::repeat_with};

use bevy::{ecs::component::Component, utils::default};
use rand::{rngs::StdRng, seq::SliceRandom, thread_rng, SeedableRng};
use tap::Tap;

use super::MinoKind;

#[derive(Component)]
pub struct PieceQueue {
    window: VecDeque<MinoKind>,
    window_size: usize,
    rng: StdRng,
}

impl Default for PieceQueue {
    fn default() -> Self {
        Self {
            window: default(),
            window_size: 5,
            rng: StdRng::from_rng(thread_rng()).expect("could not construct an rng"),
        }
        .tap_mut(|a| a.refill_window())
    }
}

impl PieceQueue {
    pub fn window(&self) -> &VecDeque<MinoKind> {
        &self.window
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
