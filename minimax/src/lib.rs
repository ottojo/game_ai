use game_ai::{Evaluator, GameAi, GameRules, GameStateTrait};

#[derive(Clone)]
pub struct MiniMax<Eval: Evaluator + Clone> {
    evaluator: Eval,
    depth: usize,
}

impl<Eval: Evaluator + Clone> MiniMax<Eval> {
    pub fn new(depth: usize, evaluator: Eval) -> MiniMax<Eval> {
        MiniMax { depth, evaluator }
    }
}

impl<Eval: Evaluator + Clone> GameAi<Eval::Rules> for MiniMax<Eval> {
    fn determine_next_move(
        &mut self,
        gamestate: &<Eval::Rules as GameRules>::State,
    ) -> <Eval::Rules as GameRules>::Action {
        let possible_moves = gamestate.get_actions();
        let mut moves_values = vec![];

        for action in possible_moves {
            let next_state = Eval::Rules::play(gamestate, &action);
            moves_values.push((
                action,
                minimax_value(
                    &next_state,
                    self.depth,
                    f32::NEG_INFINITY,
                    f32::INFINITY,
                    gamestate.next_player().is_maximizing(),
                    &self.evaluator,
                ),
            ));
            assert!(moves_values.last().unwrap().1.is_finite());
        }

        if gamestate.next_player().is_maximizing() {
            moves_values
                .iter()
                .max_by(|(_action1, value1), (_action2, value2)| value1.total_cmp(value2))
                .unwrap()
                .0
                .clone()
        } else {
            moves_values
                .iter()
                .min_by(|(_action1, value1), (_action2, value2)| value1.total_cmp(value2))
                .unwrap()
                .0
                .clone()
        }
    }

    fn name(&self) -> String {
        format!("MiniMax (depth {})", self.depth)
    }
}

fn minimax_value<Rules: GameRules, Eval: Evaluator<Rules = Rules>>(
    state: &Rules::State,
    depth: usize,
    mut alpha: f32, // minimum score that the maximizing player is assured of
    mut beta: f32,  // maximum score that the minimizing player is assured of
    maximizing_player: bool,
    eval: &Eval,
) -> f32 {
    if depth == 0 || state.is_final() {
        return eval.value(state);
    }

    if maximizing_player {
        let mut value = std::f32::NEG_INFINITY;
        let possible_moves = state.get_actions();
        for action in possible_moves {
            let child_state = Rules::play(state, &action);
            value = value.max(minimax_value(
                &child_state,
                depth - 1,
                alpha,
                beta,
                false,
                eval,
            ));
            alpha = alpha.max(value);
            if value >= beta {
                // Other children would only increase this nodes value,
                // but one of our siblings has a lower value already,
                // which would be chosen by the parent state
                break;
            }
        }
        value
    } else {
        let mut value = std::f32::INFINITY;
        let possible_moves = state.get_actions();
        for action in possible_moves {
            let child_state = Rules::play(state, &action);
            value = value.min(minimax_value(
                &child_state,
                depth - 1,
                alpha,
                beta,
                false,
                eval,
            ));
            beta = beta.min(value);
            if value <= alpha {
                // Other children would only reduce this nodes value,
                // but one of our siblings has a higher value already,
                // which would be chosen by the parent state
                break;
            }
        }
        value
    }
}
