use core::fmt;

use crate::hexgrid::{AxialVector, HexGrid};

pub mod rules;

#[derive(Clone, PartialEq, Copy, Debug)]
pub enum Player {
    Rubies, // 0
    Pearls, // 1
}

impl Player {
    pub fn opponent(&self) -> Player {
        match self {
            Player::Rubies => Player::Pearls,
            Player::Pearls => Player::Rubies,
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum CellState {
    Empty,
    Occupied(Player),
    Blocked,
}

#[derive(PartialEq, Debug)]
pub enum MoveResult {
    Success,
    Fail,
}

#[derive(PartialEq, Debug)]
pub enum GameResult {
    Tie,
    Win(Player),
}

impl GameResult {
    pub fn winner(&self) -> Option<Player> {
        match self {
            GameResult::Tie => None,
            GameResult::Win(winner) => Some(*winner),
        }
    }
}

pub struct Scores {
    pub rubies: i32,
    pub pearls: i32,
}

#[derive(Clone, PartialEq)]
pub struct GameState {
    next_move: Player,
    field: HexGrid<CellState>,
}

impl fmt::Debug for GameState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GameState")
            .field("next_move", &self.next_move)
            .finish_non_exhaustive()
    }
}

impl GameState {
    pub fn initialize() -> GameState {
        let mut field = HexGrid::new_fill(5, CellState::Empty);
        *field.get_mut((0, -1).into()).unwrap() = CellState::Blocked;
        *field.get_mut((1, 0).into()).unwrap() = CellState::Blocked;
        *field.get_mut((-1, 1).into()).unwrap() = CellState::Blocked;

        *field.get_mut((-4, 0).into()).unwrap() = CellState::Occupied(Player::Rubies);
        *field.get_mut((4, -4).into()).unwrap() = CellState::Occupied(Player::Rubies);
        *field.get_mut((0, 4).into()).unwrap() = CellState::Occupied(Player::Rubies);

        *field.get_mut((0, -4).into()).unwrap() = CellState::Occupied(Player::Pearls);
        *field.get_mut((-4, 4).into()).unwrap() = CellState::Occupied(Player::Pearls);
        *field.get_mut((4, 0).into()).unwrap() = CellState::Occupied(Player::Pearls);

        GameState {
            field,
            next_move: Player::Rubies,
        }
    }

    pub fn get_field(&self) -> &HexGrid<CellState> {
        &self.field
    }

    pub fn next_player(&self) -> Player {
        self.next_move
    }

    pub fn player_move(&mut self, from: AxialVector, to: AxialVector) -> MoveResult {
        let player = self.next_move;

        let move_length = (to - from).length();

        // Test if move is in range
        #[allow(clippy::manual_range_contains)]
        if move_length > 2 || move_length < 1 {
            return MoveResult::Fail;
        }
        if !self.field.is_in_bounds(from) || !self.field.is_in_bounds(to) {
            return MoveResult::Fail;
        }

        // Test if source contains piece
        if *self.field.get(from).unwrap() != CellState::Occupied(player) {
            return MoveResult::Fail;
        }

        // Test if target is empty
        if *self.field.get(to).unwrap() != CellState::Empty {
            return MoveResult::Fail;
        }
        // Place piece at target

        *self.field.get_mut(to).unwrap() = CellState::Occupied(player);

        // Potentially remove piece at source
        if move_length == 2 {
            *self.field.get_mut(from).unwrap() = CellState::Empty;
        }

        // Capture neighbors of target
        for direction in 0..6 {
            let to_capture = to + AxialVector::direction(direction);
            // TODO unwrap fails after bounds check? at -1,4
            if self.field.is_in_bounds(to_capture)
                && *self.field.get(to_capture).unwrap() == CellState::Occupied(player.opponent())
            {
                *self.field.get_mut(to_capture).unwrap() = CellState::Occupied(player);
            }
        }

        self.next_move = self.next_move.opponent();

        MoveResult::Success
    }

    pub fn scores(&self) -> Scores {
        let mut rubies = 0;
        let mut pearls = 0;

        for (_pos, state) in self.field.tile_iter() {
            match state {
                CellState::Occupied(Player::Rubies) => {
                    rubies += 1;
                }
                CellState::Occupied(Player::Pearls) => {
                    pearls += 1;
                }
                _ => {}
            }
        }

        Scores { rubies, pearls }
    }

    pub fn result(&self) -> Option<GameResult> {
        for (empty_cell_vec, tile) in self.field.tile_iter() {
            if *tile == CellState::Empty {
                let mut reachable = false;

                'reachability_check: for ring_radius in 1..3 {
                    let mut hex: AxialVector =
                        ring_radius * AxialVector::direction(4) + *empty_cell_vec;
                    for edge_direction in 0..6 {
                        for _edge_index in 0..ring_radius {
                            if self.field.is_in_bounds(hex)
                                && *self.field.get(hex).unwrap()
                                    == CellState::Occupied(self.next_move)
                            {
                                reachable = true;
                                break 'reachability_check;
                            }

                            hex = hex + AxialVector::direction(edge_direction);
                        }
                    }
                }

                if reachable {
                    // Game not finished yet: There is an empty cell reachable by the current player
                    return None;
                }
            }
        }
        let scores = self.scores();

        match scores.pearls.cmp(&scores.rubies) {
            std::cmp::Ordering::Less => Some(GameResult::Win(Player::Rubies)),
            std::cmp::Ordering::Equal => Some(GameResult::Tie),
            std::cmp::Ordering::Greater => Some(GameResult::Win(Player::Pearls)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::GameState;

    #[test]
    fn initialize() {
        let state = GameState::initialize();
        let score = state.scores();
        assert_eq!(score.pearls, 3);
        assert_eq!(score.rubies, 3);
        assert_eq!(state.result(), None);
    }
}
