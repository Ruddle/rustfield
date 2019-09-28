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

        let cell_pos_local: CellPos = (
            (self.pos.x / GRID_CELL_SIZE) as usize - current_zone.min_i(),
            (self.pos.y / GRID_CELL_SIZE) as usize - current_zone.min_j(),
        )
            .into();

        self.next_dir =
            if let Some(Some(flowfield)) = path.computed.get_safe(&current_zone.large_cell_pos()) {
                let v = flowfield.flow.get(&cell_pos_local);
                let x = (v % 3) - 1;
                let y = (v / 3) - 1;

                Vector2::new(x as f32, y as f32) * 1.0
            } else {
                Vector2::new(0.0, 0.0)
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
