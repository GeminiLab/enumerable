use std::collections::BTreeSet;

use enumerable::Enumerable;

#[derive(Clone, Copy, Debug)]
struct Slope(i64, i64);

impl PartialEq for Slope {
    fn eq(&self, other: &Self) -> bool {
        self.0 * other.1 == self.1 * other.0
    }
}

impl Eq for Slope {}

impl PartialOrd for Slope {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Slope {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.0 * other.1).cmp(&(self.1 * other.0))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Enumerable)]
struct FiniteGrid<T: Default + Copy>
where
    T: Into<i64>,
{
    x: T,
    y: T,
}

impl<T: Default + Copy> FiniteGrid<T>
where
    T: Into<i64>,
{
    /// The zero point of the grid.
    fn zero() -> Self {
        Self {
            x: T::default(),
            y: T::default(),
        }
    }

    /// Calculates the slope between two points. If the points are the same, the slope is (0, 1).
    fn slope(&self, other: &Self) -> Slope {
        fn gcd(a: u64, b: u64) -> u64 {
            if b == 0 {
                a
            } else {
                gcd(b, a % b)
            }
        }

        let dx = (other.x.into() as i64) - (self.x.into() as i64);
        let dy = (other.y.into() as i64) - (self.y.into() as i64);

        if dx == 0 {
            return Slope(0, 1);
        }

        let gcd = gcd(dx.abs() as u64, dy.abs() as u64);
        Slope(dx / gcd as i64, dy / gcd as i64)
    }
}

fn main() {
    let mut set = BTreeSet::<Slope>::new();
    let mut count = 0;
    let zero = FiniteGrid::zero();
    for p in FiniteGrid::<u8>::enumerator() {
        let slope = zero.slope(&p);
        if set.insert(slope) {
            count += 1;
        }
    }

    println!("Number of unique slopes in a u8xu8 grid: {}", count);

    let mut set = BTreeSet::<Slope>::new();
    let mut count = 0;
    let zero = FiniteGrid::zero();
    for p in FiniteGrid::<i8>::enumerator() {
        let slope = zero.slope(&p);
        if set.insert(slope) {
            count += 1;
        }
    }

    println!("Number of unique slopes in a i8xi8 grid: {}", count);
}
