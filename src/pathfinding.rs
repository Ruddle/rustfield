use crate::field::{CellPos, Field};
use std::collections::{HashMap, HashSet};

pub struct PathComputer {
    pub astars: Vec<AStarCompute>,
}

impl PathComputer {
    pub fn new() -> Self {
        PathComputer { astars: Vec::new() }
    }
}

#[derive(Debug, Clone)]
pub struct AStarNode {
    pub cell_pos: CellPos,
    pub g: i32,
    pub h: i32,
    pub parent: Option<Box<AStarNode>>,
}

impl AStarNode {
    pub fn f_static(g: i32, h: i32) -> i32 {
        g + h * 1
    }

    pub fn f(&self) -> i32 {
        AStarNode::f_static(self.g, self.h)
    }
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
        closed_nodes: HashSet<CellPos>,
    },
    Computed {
        from: CellPos,
        to: CellPos,
        path: Vec<CellPos>,
    },
}

impl AStarCompute {
    pub fn from_to(&self) -> (&CellPos, &CellPos) {
        match self {
            AStarCompute::InitialData { from, to, .. } => (from, to),
            AStarCompute::Computing { from, to, .. } => (from, to),
            AStarCompute::Computed { from, to, .. } => (from, to),
        }
    }

    fn append_parent(astarnode: &AStarNode, path: &mut Vec<CellPos>) {
        for parent_node in &astarnode.parent {
            path.push(parent_node.cell_pos.clone());
            AStarCompute::append_parent(parent_node, path);
        }
    }

    fn neighbor_replace_push_in_open(
        neighbor_pos: &CellPos,
        neighbor_dist: i8,
        min_node: &AStarNode,
        open_nodes: &mut Vec<AStarNode>,
        cost: &Field<u8>,
        to: &CellPos,
    ) {
        let g = min_node.g + (neighbor_dist as i32 * cost.get(neighbor_pos) as i32);
        let h = to.distance(neighbor_pos);

        let index = open_nodes
            .iter()
            .enumerate()
            .find(|x| x.1.cell_pos == *neighbor_pos)
            .map(|x| x.0);

        match index {
            Some(index) => {
                if AStarNode::f_static(g, h) < open_nodes[index].f() {
                    let node_to_evaluate = AStarNode {
                        cell_pos: neighbor_pos.clone(),
                        g,
                        h,
                        parent: Some(Box::new(min_node.clone())),
                    };
                    std::mem::replace(&mut open_nodes[index], node_to_evaluate);
                }
            }
            None => {
                let node_to_evaluate = AStarNode {
                    cell_pos: neighbor_pos.clone(),
                    g,
                    h,
                    parent: Some(Box::new(min_node.clone())),
                };
                open_nodes.push(node_to_evaluate);
            }
        }
    }

    pub fn step(self) -> Self {
        match self {
            AStarCompute::InitialData { from, to, cost } => AStarCompute::Computing {
                from,
                to,
                cost,
                sort_cost: 0,
                open_nodes: vec![AStarNode {
                    cell_pos: from,
                    g: 0,
                    h: from.distance(&to),
                    parent: None,
                }],
                closed_nodes: HashSet::new(),
            },
            AStarCompute::Computing {
                from,
                to,
                cost,
                sort_cost,
                mut open_nodes,
                mut closed_nodes,
            } => {
                //                let start = std::time::Instant::now();

                let mut min_node = open_nodes[0].clone();
                let mut min_f = open_nodes[0].f();
                let mut min_index = 0;
                for (index, node) in open_nodes.iter().enumerate() {
                    let f = node.f();
                    if f < min_f {
                        min_f = f;
                        min_index = index;
                        min_node = (*node).clone();
                    }
                }
                //                let new_sort_cost = sort_cost + start.elapsed().as_micros();

                if to == min_node.cell_pos {
                    let mut path = Vec::new();

                    AStarCompute::append_parent(&min_node, &mut path);
                    AStarCompute::Computed { from, to, path }
                } else {
                    open_nodes.remove(min_index);
                    closed_nodes.insert(min_node.cell_pos);
                    let neighbors = crate::field::neighbors_with_distance_iter(
                        &min_node.cell_pos,
                        cost.width,
                        cost.height,
                    );

                    for (neighbor_pos, neighbor_dist) in neighbors {
                        if !closed_nodes.contains(&neighbor_pos) {
                            AStarCompute::neighbor_replace_push_in_open(
                                &neighbor_pos,
                                neighbor_dist,
                                &min_node,
                                &mut open_nodes,
                                &cost,
                                &to,
                            );
                        }
                    }

                    AStarCompute::Computing {
                        from,
                        to,
                        cost,
                        sort_cost: 0, //new_sort_cost,
                        open_nodes,
                        closed_nodes,
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

impl PathComputer {
    pub fn astar(from: CellPos, to: CellPos, cost: Field<u8>) -> AStarCompute {
        AStarCompute::InitialData { from, to, cost }
    }
}
