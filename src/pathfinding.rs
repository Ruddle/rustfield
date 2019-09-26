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

#[derive(PartialEq, Debug)]
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
    pub flowfields: Vec<(Zone, FlowField)>,
}

pub enum FullPathCompute {
    Astar(AStarCompute),
    AstarDoneFieldFinding {
        astar: astar::Result,
    },
    ComputingFlowFields {
        astar: astar::Result,
        zone_to_visit: Vec<Zone>,
        computing: (Zone, FlowField),
        computed: Vec<(Zone, FlowField)>,
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
                computing_field.set(&cell_pos, global_cost.get(&CellPos { i, j }))
            }
        }
        computing_field
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

                    if match zone_traversed_vec.last() {
                        Some(z) if z == &zone => false,
                        _ => true,
                    } {
                        zone_traversed_vec.push(zone);
                    }
                }

                let first_zone = zone_traversed_vec.pop().unwrap();
                let mut computing_field_cost =
                    FullPathCompute::zone_global_cost_to_local_cost(&first_zone, &astar.cost);

                let mut computing_field = FlowField {
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
                };

                FullPathCompute::ComputingFlowFields {
                    astar,
                    zone_to_visit: zone_traversed_vec,
                    computing: (first_zone, computing_field),
                    computed: Vec::new(),
                }
            }

            FullPathCompute::ComputingFlowFields {
                astar,
                mut zone_to_visit,
                mut computing,
                mut computed,
            } => {
                computing.1.step();
                match computing.1.state {
                    FlowFieldState::Ready => {
                        computed.push(computing);
                        if zone_to_visit.is_empty() {
                            FullPathCompute::FlowFieldComputed(Result {
                                flowfields: computed,
                            })
                        } else {
                            let next_zone = zone_to_visit.pop().unwrap();
                            let cost = FullPathCompute::zone_global_cost_to_local_cost(
                                &next_zone,
                                &astar.cost,
                            );

                            let mut integration = Field::new(
                                crate::flowfield::MAX_INTEGRATION as i32,
                                GRID_SIZE,
                                GRID_SIZE,
                            );

                            let mut next_flow = Field::<i8>::new(4, GRID_SIZE, GRID_SIZE);
                            let mut to_visit = Vec::new();

                            //Fill integration and to_visit with just completed field
                            let (last_zone, last_flowfield) = computed.last().unwrap();

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

                                integration
                                    .set(&local_next, last_flowfield.integration.get(&local_last));
                                to_visit.push(local_next);
                                next_flow.set(&local_next, last_flowfield.flow.get(&local_last));
                            }

                            let computing_flow = FlowField {
                                objective: CellPos::new(),
                                cost,
                                integration,
                                flow: next_flow,
                                to_visit,
                                state: FlowFieldState::ComputingIntegration,
                            };
                            FullPathCompute::ComputingFlowFields {
                                astar,
                                zone_to_visit,
                                computing: (next_zone, computing_flow),
                                computed,
                            }
                        }
                    }
                    _ => FullPathCompute::ComputingFlowFields {
                        astar,
                        zone_to_visit,
                        computing,
                        computed,
                    },
                }
            }
            //                FullPathCompute::FlowFieldComputed {
            //                flowfields: Vec::new(),
            //            }
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
