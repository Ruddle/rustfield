use crate::field::{CellPos, Field};
use std::collections::HashSet;

pub const MAX_INTEGRATION: usize = GRID_SIZE * 10 * 10;

pub const GRID_SIZE: usize = 32;
pub const CELLS: usize = GRID_SIZE * GRID_SIZE;

#[derive(PartialEq)]
pub enum FlowFieldState {
    Created,
    ComputingIntegration,
    Ready,
}

pub struct FlowField {
    pub cost: Field<u8>,
    pub integration: Field<i32>,
    pub flow: Field<i8>,
    objective: CellPos,
    pub to_visit: Vec<CellPos>,
    pub state: FlowFieldState,
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
            let neighbors = crate::field::neighbors_with_distance(visit, GRID_SIZE, GRID_SIZE);

            for (neighbor, distance) in &neighbors {
                let old_integration = self.integration.get(neighbor);
                let cost_of_neighbor = self.cost.get(neighbor);
                let current_integration = self.integration.get(visit);
                let new_integration =
                    current_integration + cost_of_neighbor as i32 * distance.clone() as i32;
                if new_integration < MAX_INTEGRATION as i32 && new_integration < old_integration {
                    self.integration.set(neighbor, new_integration);
                    self.to_visit.push(neighbor.clone());
                }
            }
        }
    }
}
