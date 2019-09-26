use crate::field::{CellPos, Field};

#[derive(Debug, Clone)]
pub struct AStarNode {
    pub cell_pos: CellPos,
    pub g: i32,
    pub h: i32,
    pub parent: Option<Box<AStarNode>>,
}

impl AStarNode {
    pub fn f_static(g: i32, h: i32) -> i32 {
        g + h * 2
    }

    pub fn f(&self) -> i32 {
        AStarNode::f_static(self.g, self.h)
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum NodeState {
    Open(i32),
    Closed,
    Unknown,
}

#[derive(Debug)]
pub struct Result {
    pub from: CellPos,
    pub to: CellPos,
    pub path: Vec<CellPos>,
    pub cost: Field<u8>,
}

#[derive(Debug)]
pub enum AStarCompute {
    InitialData {
        from: CellPos,
        to: CellPos,
        cost: Field<u8>,
    },
    Computing {
        from: CellPos,
        to: CellPos,
        cost: Field<u8>,
        sort_cost: u128,
        open_nodes: Vec<AStarNode>,
        nodes_state: Field<NodeState>,
    },
    Computed(Result),
}

impl AStarCompute {
    pub fn from_to(&self) -> (&CellPos, &CellPos) {
        match self {
            AStarCompute::InitialData { from, to, .. } => (from, to),
            AStarCompute::Computing { from, to, .. } => (from, to),
            AStarCompute::Computed(Result { from, to, .. }) => (from, to),
        }
    }

    fn append_parent(astarnode: &AStarNode, path: &mut Vec<CellPos>) {
        for parent_node in &astarnode.parent {
            path.push(parent_node.cell_pos.clone());
            AStarCompute::append_parent(parent_node, path);
        }
    }

    pub fn step(self) -> Self {
        match self {
            AStarCompute::InitialData { from, to, cost } => {
                let capacity = (from.distance(&to) / (10 / 2)) as usize;
                let mut open_nodes = Vec::with_capacity(capacity);

                let (w, h) = (cost.width, cost.height);

                open_nodes.push(AStarNode {
                    cell_pos: from,
                    g: 0,
                    h: from.distance(&to),
                    parent: None,
                });
                AStarCompute::Computing {
                    from,
                    to,
                    cost,
                    sort_cost: 0,
                    open_nodes,
                    nodes_state: Field::<NodeState>::new(NodeState::Unknown, w, h),
                }
            }
            AStarCompute::Computing {
                from,
                to,
                cost,
                sort_cost,
                mut open_nodes,
                mut nodes_state,
            } => {
                //                let start = std::time::Instant::now();
                let mut min_node = open_nodes.pop().unwrap();
                //                let new_sort_cost = sort_cost + start.elapsed().as_micros();

                if to == min_node.cell_pos {
                    let mut path = Vec::new();
                    path.push(to);

                    AStarCompute::append_parent(&min_node, &mut path);
                    AStarCompute::Computed(Result {
                        from,
                        to,
                        path,
                        cost,
                    })
                } else {
                    nodes_state.set(&min_node.cell_pos, NodeState::Closed);
                    let neighbors = crate::field::neighbors_with_distance_iter(
                        &min_node.cell_pos,
                        cost.width,
                        cost.height,
                    );

                    for (neighbor_pos, neighbor_dist) in neighbors {
                        match nodes_state.get(&neighbor_pos) {
                            NodeState::Unknown => {
                                let g = min_node.g
                                    + (neighbor_dist as i32
                                        * match cost.get(&neighbor_pos) {
                                            255 => 255000,
                                            x => x as i32,
                                        });
                                let h = to.distance(&neighbor_pos);
                                let f = AStarNode::f_static(g, h);

                                nodes_state.set(&neighbor_pos, NodeState::Open(f));
                                let pos = open_nodes
                                    .binary_search_by(|e| f.cmp(&e.f()))
                                    .unwrap_or_else(|e| e);

                                let node_to_evaluate = AStarNode {
                                    cell_pos: neighbor_pos.clone(),
                                    g,
                                    h,
                                    parent: Some(Box::new(min_node.clone())),
                                };
                                open_nodes.insert(pos, node_to_evaluate);
                            }
                            //                            NodeState::Open(oldF) => {
                            //                                let g = min_node.g
                            //                                    + (neighbor_dist as i32 * cost.get(&neighbor_pos) as i32);
                            //                                let h = to.distance(&neighbor_pos);
                            //                                let f = AStarNode::f_static(g, h);
                            //                                if oldF > f {
                            //                                    nodes_state.set(&neighbor_pos, NodeState::Open(f));
                            //                                    let pos_remove = open_nodes
                            //                                        .iter()
                            //                                        .position(|e| e.cell_pos == neighbor_pos)
                            //                                        .unwrap();
                            //                                    open_nodes.remove(pos_remove);
                            //
                            //                                    let node_to_evaluate = AStarNode {
                            //                                        cell_pos: neighbor_pos.clone(),
                            //                                        g,
                            //                                        h,
                            //                                        parent: Some(Box::new(min_node.clone())),
                            //                                    };
                            //
                            //                                    let pos = open_nodes
                            //                                        .binary_search_by(|e| f.cmp(&e.f()))
                            //                                        .unwrap_or_else(|e| e);
                            //                                    open_nodes.insert(pos, node_to_evaluate);
                            //                                }
                            //                            }
                            _ => {}
                        }
                    }

                    AStarCompute::Computing {
                        from,
                        to,
                        cost,
                        sort_cost: 0, //new_sort_cost,
                        open_nodes,
                        nodes_state: nodes_state,
                    }
                }
            }

            computed @ AStarCompute::Computed { .. } => computed,
        }
    }

    pub fn step_replace(s: &mut AStarCompute) {
        let old = std::mem::replace(
            s,
            AStarCompute::InitialData {
                from: CellPos::new(),
                to: CellPos::new(),
                cost: Field::new(0, 0, 0),
            },
        );
        *s = old.step();
    }
}
