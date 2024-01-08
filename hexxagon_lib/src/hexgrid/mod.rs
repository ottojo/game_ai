use rustc_hash::FxHashMap;

pub trait CellTypeTrait: Clone {}
impl<T: Clone> CellTypeTrait for T {}

#[derive(Clone)]
pub struct HexGrid<CellType: CellTypeTrait> {
    size: i32,
    storage: FxHashMap<AxialVector, CellType>,
}

impl<CellType: CellTypeTrait> HexGrid<CellType> {
    pub fn get_mut(&mut self, coordinate: AxialVector) -> Option<&mut CellType> {
        self.storage.get_mut(&coordinate)
    }

    pub fn get(&self, coordinate: AxialVector) -> Option<&CellType> {
        self.storage.get(&coordinate)
    }

    pub fn tile_iter(&self) -> impl Iterator<Item = (&AxialVector, &CellType)> {
        self.storage.iter()
    }

    /// Creates a new HexGrid of specified size, with all cells set to specified value.
    ///
    /// # Arguments
    ///
    /// * `size` - Radius of circumcircle
    /// * `value` - The initial value of every cell
    pub fn new_fill(size: i32, value: CellType) -> HexGrid<CellType> {
        // TODO: nice storage...
        // r goes from -size to +size, number of rows is 2*size+1
        // Row index "r", a row is from top left to bottom right
        // Row length is (2*size+1)-abs(r)
        // Row starts with item q=max(-size,-size-r)

        let mut map = FxHashMap::default();
        map.insert(AxialVector::new(0, 0), value.clone());
        for ring_radius in 1..size {
            let mut hex: AxialVector = ring_radius * AxialVector::direction(4);
            for edge_direction in 0..6 {
                for _edge_index in 0..ring_radius {
                    map.insert(hex, value.clone());
                    hex = hex + AxialVector::direction(edge_direction);
                }
            }
        }

        assert_eq!(map.len() as i32, 1 + 3 * size * (size - 1));

        HexGrid { size, storage: map }
    }

    pub fn is_in_bounds(&self, coordinate: AxialVector) -> bool {
        coordinate.length() < self.size
    }
}

#[allow(unused)]
#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct AxialVector {
    q: i32,
    r: i32,
}

impl AxialVector {
    pub fn q(&self) -> i32 {
        self.q
    }
    pub fn r(&self) -> i32 {
        self.r
    }
    pub fn s(&self) -> i32 {
        -self.q - self.r
    }
    pub const fn new(q: i32, r: i32) -> AxialVector {
        AxialVector { q, r }
    }
    pub fn round_nearest(frac_q: f32, frac_r: f32) -> AxialVector {
        let mut q = frac_q.round() as i32;
        let mut r = frac_r.round() as i32;
        let frac_s = -frac_q - frac_r;
        let s = frac_s.round() as i32;

        let q_diff = (q as f32 - frac_q).abs();
        let r_diff = (r as f32 - frac_r).abs();
        let s_diff = (s as f32 - frac_s).abs();

        if q_diff > r_diff && q_diff > s_diff {
            q = -r - s
        } else if r_diff > s_diff {
            r = -q - s
        }
        AxialVector::new(q, r)
    }

    pub fn length(&self) -> i32 {
        (self.q.abs() + (self.q + self.r).abs() + self.r.abs()).abs() / 2
    }

    pub fn direction(index: u8) -> AxialVector {
        assert!(index < 6);
        match index {
            0 => AxialVector::new(1, 0),
            1 => AxialVector::new(1, -1),
            2 => AxialVector::new(0, -1),
            3 => AxialVector::new(-1, 0),
            4 => AxialVector::new(-1, 1),
            5 => AxialVector::new(0, 1),
            _ => panic!(),
        }
    }
}

impl std::fmt::Display for AxialVector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.q, self.r)
    }
}

impl std::ops::Add for AxialVector {
    type Output = AxialVector;

    fn add(self, rhs: Self) -> Self::Output {
        AxialVector {
            q: self.q + rhs.q,
            r: self.r + rhs.r,
        }
    }
}

impl std::ops::Sub for AxialVector {
    type Output = AxialVector;

    fn sub(self, rhs: Self) -> Self::Output {
        AxialVector {
            q: self.q - rhs.q,
            r: self.r - rhs.r,
        }
    }
}

impl std::ops::Mul<AxialVector> for i32 {
    type Output = AxialVector;

    fn mul(self, rhs: AxialVector) -> Self::Output {
        AxialVector {
            q: rhs.q * self,
            r: rhs.r * self,
        }
    }
}

impl From<(i32, i32)> for AxialVector {
    fn from(value: (i32, i32)) -> Self {
        AxialVector {
            q: value.0,
            r: value.1,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::hexgrid::AxialVector;

    use super::HexGrid;

    #[test]
    fn axial_vector_length() {
        assert_eq!(AxialVector::new(0, 0).length(), 0);

        assert_eq!(AxialVector::new(0, -1).length(), 1);
        assert_eq!(AxialVector::new(1, -1).length(), 1);
        assert_eq!(AxialVector::new(1, 0).length(), 1);
        assert_eq!(AxialVector::new(0, 1).length(), 1);
        assert_eq!(AxialVector::new(-1, 1).length(), 1);
        assert_eq!(AxialVector::new(-1, 0).length(), 1);

        assert_eq!(AxialVector::new(0, -2).length(), 2);
        assert_eq!(AxialVector::new(1, -2).length(), 2);
        assert_eq!(AxialVector::new(-1, 2).length(), 2);

        assert_eq!(AxialVector::new(0, -4).length(), 4);
        assert_eq!(AxialVector::new(1, -4).length(), 4);
        assert_eq!(AxialVector::new(2, -4).length(), 4);
        assert_eq!(AxialVector::new(4, -2).length(), 4);
        assert_eq!(AxialVector::new(-3, 4).length(), 4);
    }

    #[test]
    fn axial_vector_distance() {
        assert_eq!(
            (AxialVector::new(-3, 0) - AxialVector::new(0, -2)).length(),
            3
        );
        assert_eq!(
            (AxialVector::new(0, -2) - AxialVector::new(-3, 0)).length(),
            3
        );
    }

    #[test]
    fn map_fill() {
        let map = HexGrid::new_fill(4, 0u8);
        assert!(!map.is_in_bounds((2, -4).into()));
        assert!(!map.is_in_bounds((2, -5).into()));

        let map2 = HexGrid::new_fill(5, 0u8);
        assert!(map2.is_in_bounds((-1, 4).into()));
        assert!(map2.get((-1, 4).into()).is_some());
        assert!(!map2.is_in_bounds((-2, 5).into()));
    }
}
