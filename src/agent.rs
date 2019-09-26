use crate::field::CellPos;
use crate::flowfield::GRID_SIZE;
use crate::misc::Vector2;
use crate::{pathfinding, GRID_CELL_SIZE};

pub struct Agent {
    pub pos: Vector2,
    pub speed: Vector2,
    pub next_dir: Vector2,
}

impl Agent {
    pub fn grid_pos(&self) -> CellPos {
        (
            (self.pos.x / GRID_CELL_SIZE) as usize,
            (self.pos.y / GRID_CELL_SIZE) as usize,
        )
            .into()
    }

    pub fn new(pos: Vector2) -> Self {
        Agent {
            pos,
            speed: Vector2::new(0.0, 0.0),
            next_dir: Vector2::new(0.0, 0.0),
        }
    }

    pub fn follow(&mut self, path: &pathfinding::Result) {
        let current_zone =
            pathfinding::Zone::from(self.pos.x / GRID_CELL_SIZE, self.pos.y / GRID_CELL_SIZE);

        let cell_pos: CellPos = (
            (self.pos.x / GRID_CELL_SIZE) as usize - current_zone.min_i(),
            (self.pos.y / GRID_CELL_SIZE) as usize - current_zone.min_j(),
        )
            .into();

        let flow_to_use = path
            .flowfields
            .iter()
            .filter_map(|(z, f)| if z == &current_zone { Some(f) } else { None })
            .min_by(|f, f2| {
                f.integration
                    .get(&cell_pos)
                    .cmp(&f2.integration.get(&cell_pos))
            });

        if let Some(flowfield) = flow_to_use {
            let v = flowfield.flow.get(&cell_pos);
            let x = (v % 3) - 1;
            let y = (v / 3) - 1;

            self.next_dir = Vector2::new(x as f32, y as f32) * 1.0;
        } else {
            self.next_dir = Vector2::new(0.0, 0.0);
        }
    }

    pub fn step(&mut self) {
        self.speed = self.speed * 0.8 + self.next_dir * 0.2;
        //        self.speed *= 0.98;
        self.speed.x += rand::prelude::random::<f32>() - 0.5;
        self.speed.y += rand::prelude::random::<f32>() - 0.5;
        self.pos = self.pos + self.speed;
    }
}
