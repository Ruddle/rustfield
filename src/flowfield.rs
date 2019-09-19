use crate::GRID_CELL_SIZE;

pub const GRID_SIZE: usize = 100;
pub const CELLS: usize = GRID_SIZE * GRID_SIZE;

#[derive(Copy, Clone, Debug)]
pub struct CellPos {
    pub i: usize,
    pub j: usize,
}

impl CellPos {
    pub fn new() -> CellPos {
        CellPos { i: 0, j: 0 }
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

pub struct Field<T> {
    arr: Vec<T>,
}

impl<T> Field<T> {
    pub fn grow(position: &CellPos) -> Vec<CellPos> {
        let mut neighbors: Vec<CellPos> = Vec::new();
        let (i0, j0) = (position.i as i32, position.j as i32);
        for di in -1..=1 {
            for dj in -1..=1 {
                if i0 + di >= 0
                    && j0 + dj >= 0
                    && i0 + di < GRID_SIZE as i32
                    && j0 + dj < GRID_SIZE as i32
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

    pub fn neighbors_with_distance(position: &CellPos) -> Vec<(CellPos, i8)> {
        let mut neighbors: Vec<(CellPos, i8)> = Vec::new();
        let (i0, j0) = (position.i as i32, position.j as i32);
        for di in -1..=1 {
            for dj in -1..=1 {
                if !(di == 0 && dj == 0)
                    && i0 + di >= 0
                    && j0 + dj >= 0
                    && i0 + di < GRID_SIZE as i32
                    && j0 + dj < GRID_SIZE as i32
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
}

impl<T> Field<T>
where
    T: std::marker::Copy,
{
    pub fn new(initial: T) -> Field<T> {
        let mut v: Vec<T> = Vec::with_capacity(CELLS);
        for i in 0..CELLS {
            v.push(initial);
        }

        Field { arr: v }
    }

    fn index_of(&self, cell_pos: &CellPos) -> usize {
        cell_pos.i + cell_pos.j * GRID_SIZE
    }

    pub fn get(&self, position: &CellPos) -> T {
        self.arr[self.index_of(position)]
    }

    pub fn set(&mut self, position: &CellPos, v: T) {
        let index = self.index_of(position);
        self.arr[index] = v;
    }
}

#[derive(PartialEq)]
pub enum FlowFieldState {
    Created,
    ComputingIntegration,
    Ready,
}

pub struct FlowField {
    pub cost: Field<i32>,
    pub integration: Field<i32>,
    pub flow: Field<i8>,
    objective: CellPos,
    pub to_visit: Vec<CellPos>,
    pub state: FlowFieldState,
}

impl FlowField {
    pub fn new(objective: CellPos) -> FlowField {
        FlowField {
            cost: Field::new(1),
            integration: Field::new(std::i32::MAX),
            flow: Field::new(4),
            objective,
            to_visit: Vec::new(),
            state: FlowFieldState::Created,
        }
    }

    pub fn reset(&mut self) {
        self.cost = Field::new(1);
        self.state = FlowFieldState::Created;
    }

    pub fn set_objective(&mut self, objective: CellPos) {
        self.objective = objective;
        self.state = FlowFieldState::Created;
        self.step();
    }

    pub fn step(&mut self) -> bool {
        match self.state {
            FlowFieldState::Created => {
                //                self.cost.set(&self.objective, 0);
                self.integration = Field::new(std::i32::MAX);
                self.integration.set(&self.objective, 0);
                self.to_visit = vec![self.objective];
                self.state = FlowFieldState::ComputingIntegration;
            }

            FlowFieldState::ComputingIntegration => {
                self.step_integration();
                if self.to_visit.is_empty() {
                    self.step_flow();
                    self.state = FlowFieldState::Ready;
                }
            }

            FlowFieldState::Ready => {}
        }

        self.state == FlowFieldState::Ready
    }

    fn step_flow(&mut self) {
        for i in 0..GRID_SIZE as i32 {
            for j in 0..GRID_SIZE as i32 {
                let mut lowest = self.integration.get(&(i, j).into());
                let mut dir = 4;
                for di in -1..=1 {
                    for dj in -1..=1 {
                        if !(di == 0 && dj == 0)
                            && i + di >= 0
                            && j + dj >= 0
                            && i + di < GRID_SIZE as i32
                            && j + dj < GRID_SIZE as i32
                        {
                            let current = self.integration.get(&(i + di, j + dj).into());
                            if current < lowest {
                                lowest = current;
                                dir = di + 1 + (dj + 1) * 3;
                            }
                        }
                    }
                }
                self.flow.set(&(i, j).into(), dir as i8);
            }
        }
    }

    fn step_integration(&mut self) {
        let to_visit = std::mem::replace(&mut self.to_visit, Vec::new());

        for visit in &to_visit {
            let neighbors = Field::<i32>::neighbors_with_distance(visit);

            for (neighbor, distance) in &neighbors {
                let old_integration = self.integration.get(neighbor);
                let cost_of_neighbor = self.cost.get(neighbor);
                let current_integration = self.integration.get(visit);
                let new_integration =
                    current_integration + cost_of_neighbor * distance.clone() as i32;
                if new_integration < old_integration {
                    self.integration.set(neighbor, new_integration);
                    self.to_visit.push(neighbor.clone());
                }
            }
        }
    }
}
