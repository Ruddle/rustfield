use crate::astar;
use crate::astar::AStarCompute;
use crate::field::{CellPos, Field};
use crate::flowfield::{FlowField, FlowFieldState};
use crate::flowfield::{GRID_SIZE, GRID_SIZE_MINUS};
use std::cmp::Ordering;
use std::cmp::Ordering::Greater;
use std::collections::{HashMap, HashSet};

pub struct PathComputer {
    pub astars: Vec<AStarCompute>,
    pub full_paths: Vec<FullPathCompute>,
}

impl PathComputer {
    pub fn new() -> Self {
        PathComputer {
            astars: Vec::new(),
            full_paths: Vec::new(),
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct Zone {
    pub zx: usize,
    pub zy: usize,
}

impl Zone {
    pub fn from(x: f32, y: f32) -> Zone {
        Zone {
            zx: x as usize / GRID_SIZE_MINUS,

            zy: y as usize / GRID_SIZE_MINUS,
        }
    }

    pub fn large_cell_pos(&self) -> CellPos {
        (self.zx, self.zy).into()
    }

    pub fn min_i(&self) -> usize {
        self.zx * GRID_SIZE_MINUS
    }
    pub fn min_j(&self) -> usize {
        self.zy * GRID_SIZE_MINUS
    }
    pub fn max_i(&self) -> usize {
        (self.zx + 1) * GRID_SIZE_MINUS
    }
    pub fn max_j(&self) -> usize {
        (self.zy + 1) * GRID_SIZE_MINUS
    }
}

pub struct Result {
    pub computed: Field<Option<Box<FlowField>>>,
}

pub enum FullPathCompute {
    Astar(AStarCompute),
    AstarDoneFieldFinding {
        astar: astar::Result,
    },
    ComputingFlowFields {
        astar: astar::Result,
        zone_to_visit: Vec<Zone>,
        computing_zone: Zone,
        computed: Field<Option<Box<FlowField>>>,
    },
    FlowFieldComputed(Result),
}

impl FullPathCompute {
    fn zone_global_cost_to_local_cost(zone: &Zone, global_cost: &Field<u8>) -> Field<u8> {
        let mut computing_field: Field<u8> = Field::new(255, GRID_SIZE, GRID_SIZE);
        for i in zone.min_i()..=zone.max_i().min(global_cost.width - 1) {
            for j in zone.min_j()..=zone.max_j().min(global_cost.height - 1) {
                let cell_pos = CellPos {
                    i: i - zone.min_i(),
                    j: j - zone.min_j(),
                };
                computing_field.set(&cell_pos, *global_cost.get(&CellPos { i, j }))
            }
        }
        computing_field
    }

    fn compute_junction(
        computed: &Field<Option<Box<FlowField>>>,
        next_zone: &Zone,
        next: &mut FlowField,
    ) {
        for di in -1..=1_i32 {
            for dj in -1..=1_i32 {
                let zx = next_zone.zx as i32 + di;
                let zy = next_zone.zy as i32 + dj;

                if (di != 0 || dj != 0)
                    && zx >= 0
                    && zy >= 0
                    && zx < computed.width as i32
                    && zy < computed.height as i32
                {
                    let last_zone = Zone {
                        zx: zx as usize,
                        zy: zy as usize,
                    };

                    if let Some(last_flowfield) = computed.get(&last_zone.large_cell_pos()) {
                        let mut common_globals: Vec<CellPos> = Vec::new();
                        // last | next
                        if last_zone.zx < next_zone.zx && last_zone.zy == next_zone.zy {
                            for j in last_zone.min_j()..=last_zone.max_j() {
                                common_globals.push((last_zone.max_i(), j).into())
                            }
                        }
                        //  next | last
                        else if last_zone.zx > next_zone.zx && last_zone.zy == next_zone.zy {
                            for j in last_zone.min_j()..=last_zone.max_j() {
                                common_globals.push((last_zone.min_i(), j).into())
                            }
                        }
                        //  last
                        //  ____
                        //  next
                        else if last_zone.zx == next_zone.zx && last_zone.zy < next_zone.zy {
                            for i in last_zone.min_i()..=last_zone.max_i() {
                                common_globals.push((i, last_zone.max_j()).into())
                            }
                        }
                        //  next
                        //  ____
                        //  last
                        else if last_zone.zx == next_zone.zx && last_zone.zy > next_zone.zy {
                            for i in last_zone.min_i()..=last_zone.max_i() {
                                common_globals.push((i, last_zone.min_j()).into())
                            }
                        }
                        //  next
                        //  ____
                        //       last
                        else if last_zone.zx > next_zone.zx && last_zone.zy > next_zone.zy {
                            common_globals.push((last_zone.min_i(), last_zone.min_j()).into())
                        }
                        //  last
                        //  ____
                        //       next
                        else if last_zone.zx < next_zone.zx && last_zone.zy < next_zone.zy {
                            common_globals.push((last_zone.max_i(), last_zone.max_j()).into())
                        }
                        //      next
                        //  ____
                        //  last
                        else if last_zone.zx < next_zone.zx && last_zone.zy > next_zone.zy {
                            common_globals.push((last_zone.max_i(), last_zone.min_j()).into())
                        }
                        //      last
                        //  ____
                        //  next
                        else if last_zone.zx > next_zone.zx && last_zone.zy < next_zone.zy {
                            common_globals.push((last_zone.min_i(), last_zone.max_j()).into())
                        } else {
                            println!("{:?} {:?}", last_zone, next_zone);
                        }

                        for global in common_globals {
                            let local_last = CellPos {
                                i: global.i - last_zone.min_i(),
                                j: global.j - last_zone.min_j(),
                            };

                            let local_next = CellPos {
                                i: global.i - next_zone.min_i(),
                                j: global.j - next_zone.min_j(),
                            };

                            next.integration
                                .set(&local_next, *last_flowfield.integration.get(&local_last));
                            next.to_visit.push(local_next);
                            next.state = FlowFieldState::ComputingIntegration
                        }
                    }
                }
            }
        }
    }

    pub fn step(self) -> Self {
        match self {
            FullPathCompute::Astar(astar_compute) => {
                let astar_compute = astar_compute.step();
                match astar_compute {
                    AStarCompute::Computed(astar::Result {
                        from,
                        to,
                        path,
                        cost,
                    }) => FullPathCompute::AstarDoneFieldFinding {
                        astar: astar::Result {
                            from,
                            to,
                            path,
                            cost,
                        },
                    },
                    _ => FullPathCompute::Astar(astar_compute),
                }
            }

            FullPathCompute::AstarDoneFieldFinding { astar } => {
                let mut zone_traversed_vec: Vec<Zone> = Vec::new();
                for node in astar.path.iter().rev() {
                    let zone = Zone {
                        zx: node.i / GRID_SIZE_MINUS,
                        zy: node.j / GRID_SIZE_MINUS,
                    };

                    match zone_traversed_vec.last() {
                        Some(last) => {
                            if last != &zone {
                                // Direct diagonal traversal is not allowed
                                if last.zx != zone.zx && last.zy != zone.zy {
                                    let indirection = Zone {
                                        zx: last.zx,
                                        zy: zone.zy,
                                    };
                                    zone_traversed_vec.push(indirection);
                                }
                                zone_traversed_vec.push(zone);
                            }
                        }
                        _ => {
                            zone_traversed_vec.push(zone);
                        }
                    }
                }

                //                let mut with_grow = zone_traversed_vec;

                let mut with_grow = Vec::new();
                for zone in &zone_traversed_vec {
                    for i in -1..=1_i32 {
                        for j in -1..=1_i32 {
                            if i == 0 || j == 0 {
                                let zx = zone.zx as i32 + i;
                                let zy = zone.zy as i32 + j;

                                if zx >= 0
                                    && zy >= 0
                                    && (zx as usize) < (astar.cost.width / GRID_SIZE_MINUS + 1)
                                    && (zy as usize) < (astar.cost.height / GRID_SIZE_MINUS + 1)
                                {
                                    let new_zone = Zone {
                                        zx: zx as usize,
                                        zy: zy as usize,
                                    };
                                    if !zone_traversed_vec.contains(&new_zone) {
                                        with_grow.push(new_zone);
                                    }
                                }
                            }
                        }
                    }
                }
                with_grow.extend(zone_traversed_vec);
                let mut r = with_grow.clone();
                with_grow.reverse();
                with_grow.extend(r);

                let first_zone = with_grow.pop().unwrap();
                let mut computing_field_cost =
                    FullPathCompute::zone_global_cost_to_local_cost(&first_zone, &astar.cost);

                let mut computing_field = Box::new(FlowField {
                    objective: CellPos {
                        i: astar.to.i % GRID_SIZE_MINUS,
                        j: astar.to.j % GRID_SIZE_MINUS,
                    },
                    cost: computing_field_cost,
                    integration: Field::new(
                        crate::flowfield::MAX_INTEGRATION as i32,
                        GRID_SIZE,
                        GRID_SIZE,
                    ),
                    flow: Field::new(4, GRID_SIZE, GRID_SIZE),
                    to_visit: Vec::new(),
                    state: FlowFieldState::Created,
                    skip_flow: true,
                });

                let mut computed = Field::new(
                    None,
                    astar.cost.width / GRID_SIZE_MINUS + 1,
                    astar.cost.height / GRID_SIZE_MINUS + 1,
                );

                computed.set(&first_zone.large_cell_pos(), Some(computing_field));

                FullPathCompute::ComputingFlowFields {
                    astar,
                    zone_to_visit: with_grow,
                    computing_zone: first_zone,
                    computed,
                }
            }

            FullPathCompute::ComputingFlowFields {
                astar,
                mut zone_to_visit,
                mut computing_zone,
                mut computed,
            } => {
                let mut computing = computed
                    .get_mut(&computing_zone.large_cell_pos())
                    .as_mut()
                    .unwrap();

                computing.step();

                match computing.state {
                    FlowFieldState::Ready => {
                        if zone_to_visit.is_empty() {
                            let indexes: Vec<usize> = computed
                                .arr
                                .iter()
                                .enumerate()
                                .flat_map(|(index, c)| c.as_ref().map(|c| index))
                                .collect();

                            for index in indexes {
                                let zone = Zone {
                                    zx: index % computed.width,
                                    zy: index / computed.width,
                                };

                                let mut me = std::mem::replace(
                                    computed.get_mut(&zone.large_cell_pos()),
                                    None,
                                );

                                let mut neighborhood = HashMap::new();
                                for i in -1..=1_i32 {
                                    for j in -1..=1_i32 {
                                        if (i != 0 || j != 0)
                                            && i + zone.zx as i32 >= 0
                                            && j + zone.zy as i32 >= 0
                                            && (i + zone.zx as i32) < computed.width as i32
                                            && (j + zone.zy as i32) < computed.height as i32
                                        {
                                            if let Some(f) = computed.get(
                                                &(i + zone.zx as i32, j + zone.zy as i32).into(),
                                            ) {
                                                neighborhood.insert((i, j), f);
                                            }
                                        }
                                    }
                                }

                                for me in &mut me {
                                    me.step_flow_with_neighbors(&neighborhood);
                                }

                                std::mem::replace(computed.get_mut(&zone.large_cell_pos()), me);
                            }

                            FullPathCompute::FlowFieldComputed(Result { computed })
                        } else {
                            let next_zone = zone_to_visit.pop().unwrap();
                            match computed.get(&next_zone.large_cell_pos()) {
                                Some(next) => {
                                    let mut next = std::mem::replace(
                                        computed.get_mut(&next_zone.large_cell_pos()),
                                        None,
                                    )
                                    .unwrap();

                                    FullPathCompute::compute_junction(
                                        &computed, &next_zone, &mut next,
                                    );
                                    computed.set(&next_zone.large_cell_pos(), Some(next));
                                }
                                None => {
                                    let cost = FullPathCompute::zone_global_cost_to_local_cost(
                                        &next_zone,
                                        &astar.cost,
                                    );
                                    let mut next_integration = Field::new(
                                        crate::flowfield::MAX_INTEGRATION as i32,
                                        GRID_SIZE,
                                        GRID_SIZE,
                                    );
                                    let mut next_flow = Field::<i8>::new(4, GRID_SIZE, GRID_SIZE);
                                    let mut next_to_visit = Vec::new();

                                    let mut next = Box::new(FlowField {
                                        objective: CellPos::new(),
                                        cost,
                                        integration: next_integration,
                                        flow: next_flow,
                                        to_visit: next_to_visit,
                                        state: FlowFieldState::ComputingIntegration,
                                        skip_flow: true,
                                    });
                                    FullPathCompute::compute_junction(
                                        &computed, &next_zone, &mut next,
                                    );
                                    computed.set(&next_zone.large_cell_pos(), Some(next));
                                }
                            }

                            FullPathCompute::ComputingFlowFields {
                                astar,
                                zone_to_visit,
                                computing_zone: next_zone,
                                computed,
                            }
                        }
                    }
                    _ => FullPathCompute::ComputingFlowFields {
                        astar,
                        zone_to_visit,
                        computing_zone,
                        computed,
                    },
                }
            }
            _ => self,
        }
    }

    pub fn step_replace(s: &mut FullPathCompute) {
        let old = std::mem::replace(
            s,
            FullPathCompute::Astar(AStarCompute::InitialData {
                from: CellPos::new(),
                to: CellPos::new(),
                cost: Field::new(0, 0, 0),
            }),
        );
        *s = old.step();
    }
}

impl PathComputer {
    pub fn begin_astar(&mut self, from: CellPos, to: CellPos, cost: Field<u8>) {
        self.astars
            .push(AStarCompute::InitialData { from, to, cost });
    }

    pub fn begin_full_path(&mut self, from: CellPos, to: CellPos, cost: Field<u8>) {
        self.full_paths
            .push(FullPathCompute::Astar(AStarCompute::InitialData {
                from,
                to,
                cost,
            }));
    }

    pub fn all_astars(&self) -> impl Iterator<Item = &AStarCompute> {
        let inner_astar = self.full_paths.iter().filter_map(|e| match e {
            FullPathCompute::Astar(a) => Some(a),
            _ => None,
        });

        let all = self.astars.iter().chain(inner_astar);

        all
    }
}
