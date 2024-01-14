use crate::game::GameRules;

pub trait GameAi<Rules: GameRules> {
    fn determine_next_move(&mut self, gamestate: &Rules::State) -> Rules::Action;
    fn name(&self) -> String;
}

pub trait Evaluator {
    type Rules: GameRules;

    /// Player Zero is maximizing, player one is minimizing value
    fn value(&self, state: &<Self::Rules as GameRules>::State) -> f32;
}
