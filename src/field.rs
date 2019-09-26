use crate::astar::NodeState;
use std::fmt;
use std::fmt::{Debug, Formatter};

pub const NEIGHBORS_IJ: [(i32, i32); 8] = [
    (-1, -1),
    (0, -1),
    (1, -1),
    (-1, 0),
    (1, 0),
    (-1, 1),
    (0, 1),
    (1, 1),
];

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct CellPos {
    pub i: usize,
    pub j: usize,
}

impl CellPos {
    pub fn new() -> CellPos {
        CellPos { i: 0, j: 0 }
    }

    pub fn distance(&self, other: &CellPos) -> i32 {
        let di = (self.i as i32 - other.i as i32).abs();
        let dj = (self.j as i32 - other.j as i32).abs();

        if di > dj {
            10 * (di - dj) + 14 * (dj)
        } else {
            10 * (dj - di) + 14 * (di)
        }
    }
}

impl From<(usize, usize)> for CellPos {
    fn from(tuple: (usize, usize)) -> Self {
        CellPos {
            i: tuple.0,
            j: tuple.1,
        }
    }
}

impl From<(i32, i32)> for CellPos {
    fn from(tuple: (i32, i32)) -> Self {
        CellPos {
            i: tuple.0 as usize,
            j: tuple.1 as usize,
        }
    }
}

impl From<(f32, f32)> for CellPos {
    fn from(tuple: (f32, f32)) -> Self {
        CellPos {
            i: tuple.0 as usize,
            j: tuple.1 as usize,
        }
    }
}

#[derive(Clone)]
pub struct Field<T: Debug> {
    pub arr: Vec<T>,
    pub width: usize,
    pub height: usize,
}

impl Debug for Field<u8> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Field of u8")
    }
}

impl Debug for Field<bool> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Field of bool")
    }
}

impl Debug for Field<NodeState> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Field of NodeState")
    }
}

pub fn neighbors_with_distance(
    position: &CellPos,
    width: usize,
    height: usize,
) -> Vec<(CellPos, i8)> {
    let mut neighbors: Vec<(CellPos, i8)> = Vec::new();
    let (i0, j0) = (position.i as i32, position.j as i32);
    for di in -1..=1 {
        for dj in -1..=1 {
            if !(di == 0 && dj == 0)
                && i0 + di >= 0
                && j0 + dj >= 0
                && i0 + di < width as i32
                && j0 + dj < height as i32
            {
                neighbors.push((
                    CellPos {
                        i: (i0 + di) as usize,
                        j: (j0 + dj) as usize,
                    },
                    if di != 0 && dj != 0 { 14 } else { 10 },
                ))
            }
        }
    }
    neighbors
}

pub struct NeighborsIter {
    index: usize,
    i0: i32,
    j0: i32,
    width: usize,
    height: usize,
}

impl Iterator for NeighborsIter {
    type Item = (CellPos, i8);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == 8 {
            None
        } else {
            let (di, dj) = NEIGHBORS_IJ[self.index];
            self.index += 1;
            if self.i0 + di >= 0
                && self.j0 + dj >= 0
                && self.i0 + di < self.width as i32
                && self.j0 + dj < self.height as i32
            {
                Some((
                    CellPos {
                        i: (self.i0 + di) as usize,
                        j: (self.j0 + dj) as usize,
                    },
                    if di != 0 && dj != 0 { 14 } else { 10 },
                ))
            } else {
                self.next()
            }
        }
    }
}

pub fn neighbors_with_distance_iter(
    position: &CellPos,
    width: usize,
    height: usize,
) -> NeighborsIter {
    let (i0, j0) = (position.i as i32, position.j as i32);
    NeighborsIter {
        index: 0,
        i0,
        j0,
        width,
        height,
    }
}

impl<T: Debug> Field<T> {
    pub fn neighbors_with_distance(&self, position: &CellPos) -> Vec<(CellPos, i8)> {
        neighbors_with_distance(position, self.width, self.height)
    }

    pub fn grow(&self, position: &CellPos) -> Vec<CellPos> {
        let mut neighbors: Vec<CellPos> = Vec::new();
        let (i0, j0) = (position.i as i32, position.j as i32);
        for di in -1..=1 {
            for dj in -1..=1 {
                if i0 + di >= 0
                    && j0 + dj >= 0
                    && i0 + di < self.width as i32
                    && j0 + dj < self.height as i32
                {
                    neighbors.push(CellPos {
                        i: (i0 + di) as usize,
                        j: (j0 + dj) as usize,
                    })
                }
            }
        }
        neighbors
    }
}

impl<T> Field<T>
where
    T: std::marker::Copy + Debug,
{
    pub fn new(initial: T, width: usize, height: usize) -> Field<T> {
        let total = width * height;

        let mut v = vec![initial; total];

        //        let mut v: Vec<T> = Vec::with_capacity(total);
        //        for _ in 0..total {
        //            v.push(initial);
        //        }

        Field {
            arr: v,
            width,
            height,
        }
    }

    fn index_of(&self, cell_pos: &CellPos) -> usize {
        cell_pos.i + cell_pos.j * self.width
    }

    pub fn get(&self, position: &CellPos) -> T {
        self.arr[self.index_of(position)]
    }

    pub fn set(&mut self, position: &CellPos, v: T) {
        let index = self.index_of(position);
        self.arr[index] = v;
    }
}
