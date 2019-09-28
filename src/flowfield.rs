use crate::field::{CellPos, Field};
use std::collections::{HashMap, HashSet};

pub const MAX_INTEGRATION: i32 = std::i32::MAX / 2; //  GRID_SIZE * 10 * 10 * 10;

pub const GRID_SIZE: usize = 8;
pub const GRID_SIZE_MINUS: usize = GRID_SIZE - 1;
pub const CELLS: usize = GRID_SIZE * GRID_SIZE;

#[derive(PartialEq, Debug, Clone)]
pub enum FlowFieldState {
    Created,
    ComputingIntegration,
    ComputingFlow,
    Ready,
}

#[derive(Debug, Clone)]
pub struct FlowField {
    pub cost: Field<u8>,
    pub integration: Field<i32>,
    pub flow: Field<i8>,
    pub objective: CellPos,
    pub to_visit: Vec<CellPos>,
    pub state: FlowFieldState,
    pub skip_flow: bool,
}

impl FlowField {
    pub fn new(objective: CellPos) -> FlowField {
        FlowField {
            cost: Field::new(1, GRID_SIZE, GRID_SIZE),
            integration: Field::new(MAX_INTEGRATION as i32, GRID_SIZE, GRID_SIZE),
            flow: Field::new(4, GRID_SIZE, GRID_SIZE),
            objective,
            to_visit: Vec::new(),
            state: FlowFieldState::Created,
            skip_flow: false,
        }
    }

    pub fn reset(&mut self) {
        self.cost = Field::new(1, GRID_SIZE, GRID_SIZE);
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
                self.integration = Field::new(MAX_INTEGRATION as i32, GRID_SIZE, GRID_SIZE);
                self.integration.set(&self.objective, 0);
                self.to_visit = vec![self.objective];
                self.state = FlowFieldState::ComputingIntegration;
            }

            FlowFieldState::ComputingIntegration => {
                self.step_integration();
                if self.to_visit.is_empty() {
                    self.state = if self.skip_flow {
                        FlowFieldState::Ready
                    } else {
                        FlowFieldState::ComputingFlow
                    }
                }
            }

            FlowFieldState::ComputingFlow => {
                self.step_flow();
                self.state = FlowFieldState::Ready;
            }

            FlowFieldState::Ready => {}
        }

        self.state == FlowFieldState::Ready
    }

    fn step_flow(&mut self) {
        for i in 0..GRID_SIZE as i32 {
            for j in 0..GRID_SIZE as i32 {
                //                if self.flow.get(&(i, j).into()) == 4
                {
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
    }

    // neighbors =
    // [
    //     (-1,-1),(0,-1),(1,-1),
    //     ......., self ,......,
    //     .......,......,......
    // ]
    pub fn step_flow_with_neighbors(&mut self, neighbors: &HashMap<(i32, i32), &Box<FlowField>>) {
        for i in 0..GRID_SIZE as i32 {
            for j in 0..GRID_SIZE as i32 {
                //                if self.flow.get(&(i, j).into()) == 4
                {
                    let mut lowest = self.integration.get(&(i, j).into());
                    let mut dir = 4;
                    let mut count = 0;
                    for di in -1..=1 {
                        for dj in -1..=1 {
                            if !(di == 0 && dj == 0) {
                                if i + di >= 0
                                    && j + dj >= 0
                                    && i + di < GRID_SIZE as i32
                                    && j + dj < GRID_SIZE as i32
                                {
                                    let current = self.integration.get(&(i + di, j + dj).into());
                                    count += 1;
                                    if current < lowest {
                                        lowest = current;
                                        dir = di + 1 + (dj + 1) * 3;
                                    }
                                } else {
                                    // NEIGHBORHOOD

                                    let needed_neighbor = (
                                        if i + di < 0 {
                                            -1
                                        } else if i + di >= GRID_SIZE as i32 {
                                            1
                                        } else {
                                            0
                                        },
                                        if j + dj < 0 {
                                            -1
                                        } else if j + dj >= GRID_SIZE as i32 {
                                            1
                                        } else {
                                            0
                                        },
                                    );

                                    let factor_i = if (i == 0 && di == -1)
                                        || (i == GRID_SIZE_MINUS as i32 && di == 1)
                                    {
                                        2
                                    } else {
                                        1
                                    };
                                    let factor_j = if (j == 0 && dj == -1)
                                        || (j == GRID_SIZE_MINUS as i32 && dj == 1)
                                    {
                                        2
                                    } else {
                                        1
                                    };
                                    let ri =
                                        (i + di * factor_i + GRID_SIZE as i32) % GRID_SIZE as i32;
                                    let rj =
                                        (j + dj * factor_j + GRID_SIZE as i32) % GRID_SIZE as i32;

                                    let neighbor = neighbors.get(&needed_neighbor);

                                    if let (Some(neighbor)) = neighbor {
                                        let current = neighbor.integration.get(&(ri, rj).into());
                                        count += 1;
                                        if current < lowest {
                                            lowest = current;
                                            dir = di + 1 + (dj + 1) * 3;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    if true || count == 8 {
                        self.flow.set(&(i, j).into(), dir as i8);
                    } else {
                        self.flow.set(&(i, j).into(), 4);
                    }
                }
            }
        }
    }

    fn step_integration(&mut self) {
        let to_visit = std::mem::replace(&mut self.to_visit, Vec::new());

        for visit in &to_visit {
            let neighbors = crate::field::neighbors_with_distance(visit, GRID_SIZE, GRID_SIZE);
            let current_integration = *self.integration.get(visit);

            for (neighbor, distance) in &neighbors {
                let cost_of_neighbor = *self.cost.get(neighbor);

                if cost_of_neighbor == 255 {
                    //                    self.integration.set(neighbor, MAX_INTEGRATION);
                } else {
                    let old_integration = *self.integration.get(neighbor);
                    let new_integration =
                        current_integration + cost_of_neighbor as i32 * distance.clone() as i32;
                    if new_integration < MAX_INTEGRATION as i32 && new_integration < old_integration
                    {
                        self.integration.set(neighbor, new_integration);
                        self.to_visit.push(neighbor.clone());
                    }
                }
            }
        }
    }
}
