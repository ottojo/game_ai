use crate::{game::GameState, hexgrid::AxialVector};

pub struct MoveWithResult {
    pub move_: HexxagonMove,
    pub result_state: GameState,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct HexxagonMove {
    pub src: AxialVector,
    pub dst: AxialVector,
}
