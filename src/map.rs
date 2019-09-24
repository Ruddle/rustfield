use crate::field::Field;
use crate::flowfield::GRID_SIZE;

#[derive(Clone)]
pub struct Map {
    pub size: usize,
    pub chunks_on_x: u32,
    pub cost: Field<u8>,
}

struct LocalCoordinate {
    chunk_x: u32,
    chunk_y: u32,
    local_x: u32,
    local_y: u32,
}

//impl LocalCoordinate {
//    fn get_chunk(&self, map: &Map) -> Field<u32> {
//        map.chunks[self.chunk_x + self.chunk_y * map.chunks_on_x];
//    }
//}

impl Map {
    pub fn new(size: usize) -> Map {
        let chunks_on_x = (size as f32 / GRID_SIZE as f32).ceil() as u32;
        let total_chunks = chunks_on_x * chunks_on_x;

        Map {
            size,
            chunks_on_x,
            cost: Field::new(1, size, size),
        }
    }

    pub fn reset(&mut self) {
        self.cost = Field::new(1, self.size, self.size);
    }

    fn xy_to_cxcy_lxly(&self, x: u32, y: u32) -> LocalCoordinate {
        LocalCoordinate {
            chunk_x: x / self.chunks_on_x,
            chunk_y: y / self.chunks_on_x,
            local_x: x % self.chunks_on_x,
            local_y: y % self.chunks_on_x,
        }
    }

    //    fn get(&self, x: u32, y: u32, v: u32) {
    //        let lc = self.xy_to_cxcy_lxly(x, y);
    //                let chunk = self.chunks[lc.chunk_x,lc.chunk_y];
    //    }
}
