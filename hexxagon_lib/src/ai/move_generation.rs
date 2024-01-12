use rand::{seq::SliceRandom, Rng};

use crate::{
    ai::HexxagonMove,
    game::{CellState, GameState},
    hexgrid::AxialVector,
};
use smallvec::SmallVec;

static POSSIBLE_MOVES: [AxialVector; 6 + 12] = [
    // Distance 1
    AxialVector::new(-1, 1),
    AxialVector::new(0, 1),
    AxialVector::new(1, 0),
    AxialVector::new(1, -1),
    AxialVector::new(0, -1),
    AxialVector::new(-1, 0),
    // Distance 2
    AxialVector::new(-2, 2),
    AxialVector::new(-1, 2),
    AxialVector::new(0, 2),
    AxialVector::new(1, 1),
    AxialVector::new(2, 0),
    AxialVector::new(2, -1),
    AxialVector::new(2, -2),
    AxialVector::new(1, -2),
    AxialVector::new(0, -2),
    AxialVector::new(-1, -1),
    AxialVector::new(-2, 0),
    AxialVector::new(-2, 1),
];

/// Returns a **random** valid move from the given state
#[allow(unused)]
pub fn sample_valid_move(gamestate: &GameState) -> HexxagonMove {
    assert!(gamestate.result().is_none()); // Ensures a move can be found

    let mut possible_sources: SmallVec<[AxialVector; 32]> = gamestate
        // let mut possible_sources: Vec::<_> = gamestate
        .get_field()
        .tile_iter()
        .filter(|(_pos, state)| **state == CellState::Occupied(gamestate.next_player()))
        .map(|(pos, _state)| *pos)
        .collect();

    let mut rng = rand::thread_rng();

    let mut selected_source = None;
    let mut selected_destination = None;

    loop {
        // Select random source
        let source_index = rng.gen_range(0..possible_sources.len());
        selected_source = Some(possible_sources.swap_remove(source_index));

        // Collect possible destinations
        let mut possible_destinations = SmallVec::<[AxialVector; 18]>::new();

        for move_vec in POSSIBLE_MOVES {
            let hex = selected_source.unwrap() + move_vec;
            if gamestate.get_field().is_in_bounds(hex)
                && *gamestate.get_field().get(hex).unwrap() == CellState::Empty
            {
                possible_destinations.push(hex);
            }
        }

        if possible_destinations.is_empty() {
            // No move possible from this source
            continue;
        }

        // Select random destination
        selected_destination = Some(*possible_destinations.choose(&mut rng).unwrap());
        break;
    }
    assert!(selected_source.is_some());
    assert!(selected_destination.is_some());

    HexxagonMove {
        src: selected_source.unwrap(),
        dst: selected_destination.unwrap(),
    }
}

pub fn all_moves(state: &GameState) -> Vec<HexxagonMove> {
    let mut moves = vec![];
    for (source_pos, source_state) in state.get_field().tile_iter() {
        if *source_state != CellState::Occupied(state.next_player()) {
            continue;
        }

        for move_vec in POSSIBLE_MOVES {
            let target = *source_pos + move_vec;
            if state.get_field().is_in_bounds(target)
                && *state.get_field().get(target).unwrap() == CellState::Empty
            {
                moves.push(HexxagonMove {
                    src: *source_pos,
                    dst: target,
                })
            }
        }
    }

    moves
}
