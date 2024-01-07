use crate::game::GameRules;

pub trait GameAi<Rules: GameRules> {
    fn determine_next_move(&mut self, gamestate: &Rules::State) -> Rules::Action;
    fn name(&self) -> String;
}
