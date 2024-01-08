use game_ai::{GameRules, GameStateTrait, PlayerIndex, Rewards};

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct TTTAction {
    row: usize,
    col: usize,
}

#[derive(PartialEq, Clone, Debug)]
pub enum TTTPlayer {
    X,
    O,
}
impl TTTPlayer {
    fn winner_to_reward(&self) -> Rewards {
        match self {
            TTTPlayer::X => Rewards {
                player_0: 1.0,
                player_1: 0.0,
            },
            TTTPlayer::O => Rewards {
                player_0: 0.0,
                player_1: 1.0,
            },
        }
    }
}
impl std::fmt::Display for TTTPlayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TTTPlayer::X => write!(f, "X"),
            TTTPlayer::O => write!(f, "O"),
        }
    }
}

impl From<TTTPlayer> for PlayerIndex {
    fn from(value: TTTPlayer) -> Self {
        match value {
            TTTPlayer::X => PlayerIndex::Zero,
            TTTPlayer::O => PlayerIndex::One,
        }
    }
}

#[derive(Default, PartialEq, Clone, Debug)]
enum GridCell {
    #[default]
    Empty,
    Occupied(TTTPlayer),
}
impl std::fmt::Display for GridCell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GridCell::Empty => write!(f, " "),
            GridCell::Occupied(p) => write!(f, "{}", p),
        }
    }
}

#[derive(Clone)]
pub struct TTTState {
    board: [[GridCell; 3]; 3],
    next_player: TTTPlayer,
}

impl std::fmt::Display for TTTState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{{}|{}|{}\n -----\n",
            self.board[0][0], self.board[0][1], self.board[0][2]
        )?;
        write!(
            f,
            " {}|{}|{}\n -----\n",
            self.board[1][0], self.board[1][1], self.board[1][2]
        )?;
        writeln!(
            f,
            " {}|{}|{}",
            self.board[2][0], self.board[2][1], self.board[2][2]
        )?;
        write!(f, " next: {}}}", self.next_player)
    }
}

impl std::fmt::Debug for TTTState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

fn is_winner(a: &GridCell, b: &GridCell, c: &GridCell) -> Option<TTTPlayer> {
    if let GridCell::Occupied(p) = a {
        if a == b && a == c {
            return Some(p.clone());
        }
    }

    None
}

impl TTTState {
    fn winner(&self) -> Option<TTTPlayer> {
        for row in 0..3 {
            if let Some(p) = is_winner(
                &self.board[row][0],
                &self.board[row][1],
                &self.board[row][2],
            ) {
                return Some(p);
            }
        }

        for col in 0..3 {
            if let Some(p) = is_winner(
                &self.board[0][col],
                &self.board[1][col],
                &self.board[2][col],
            ) {
                return Some(p);
            }
        }

        // Top left to down right diagonal
        if let Some(p) = is_winner(&self.board[0][0], &self.board[1][1], &self.board[2][2]) {
            return Some(p);
        }

        // Other diagonal
        if let Some(p) = is_winner(&self.board[0][2], &self.board[1][1], &self.board[2][0]) {
            return Some(p);
        }

        None
    }
}

impl GameStateTrait<TTTAction> for TTTState {
    fn is_final(&self) -> bool {
        if self.winner().is_some() {
            return true;
        }

        for row in 0..3 {
            for col in 0..3 {
                if self.board[row][col] == GridCell::Empty {
                    return false;
                }
            }
        }
        // No winner, and no empty cells
        true
    }

    fn get_actions(&self) -> Vec<TTTAction> {
        let mut empty_fields = vec![];
        for row in 0..3 {
            for col in 0..3 {
                if self.board[row][col] == GridCell::Empty {
                    empty_fields.push(TTTAction { row, col });
                }
            }
        }
        empty_fields
    }

    fn reward(&self) -> Rewards {
        assert!(self.is_final());

        self.winner()
            .map(|w| w.winner_to_reward())
            .unwrap_or(Rewards {
                player_0: 0.5,
                player_1: 0.5,
            })
    }

    fn next_player(&self) -> PlayerIndex {
        PlayerIndex::from(self.next_player.clone())
    }
}

impl Default for TTTState {
    fn default() -> Self {
        Self {
            board: Default::default(),
            next_player: TTTPlayer::X,
        }
    }
}

pub struct TTTRules {}

impl GameRules for TTTRules {
    type Action = TTTAction;

    type State = TTTState;

    const N_PLAYERS: u32 = 2;

    fn play(initial_state: &Self::State, action: &Self::Action) -> Self::State {
        assert!(action.row < 3);
        assert!(action.col < 3);
        assert_eq!(initial_state.board[action.row][action.col], GridCell::Empty);

        let mut new_state = initial_state.clone();
        new_state.board[action.row][action.col] =
            GridCell::Occupied(initial_state.next_player.clone());
        new_state.next_player = match initial_state.next_player {
            TTTPlayer::X => TTTPlayer::O,
            TTTPlayer::O => TTTPlayer::X,
        };
        new_state
    }
}
