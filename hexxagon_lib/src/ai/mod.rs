pub mod move_generation;
use game_ai::Evaluator;

use crate::{
    game::{rules::HexxagonRules, GameState},
    hexgrid::AxialVector,
};

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct HexxagonMove {
    pub src: AxialVector,
    pub dst: AxialVector,
}

pub struct HexxagonEvaluator {}

impl Evaluator for HexxagonEvaluator {
    type Rules = HexxagonRules;

    fn value(&self, state: &GameState) -> f32 {
        let scores = state.scores();
        // Rubies: Zero, maximizing
        scores.rubies as f32 - scores.pearls as f32
    }
}
