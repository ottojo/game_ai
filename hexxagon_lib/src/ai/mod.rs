pub mod move_generation;
use crate::hexgrid::AxialVector;

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct HexxagonMove {
    pub src: AxialVector,
    pub dst: AxialVector,
}
