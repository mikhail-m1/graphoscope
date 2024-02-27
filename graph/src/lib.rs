#[macro_use]
extern crate log;
use read_dot::DotGraph;

use self::graph::*;
use std::fmt::Debug;

pub mod add_virtual_nodes;
pub mod draw;
pub mod generator;
pub mod graph;
pub mod ns;
pub mod place;
pub mod read_dot;
pub mod to_dag;
pub mod xcoord;
extern crate pest;
extern crate pest_derive;

pub fn full_draw<'a>(mut dot: DotGraph<'a>, extra_edges: Option<&NodeMap<(u32, u32)>>) -> Vec<u8> {
    to_dag::to_dag(&mut dot.graph);
    let mut ranks = rank_with_components(&dot.graph);
    add_virtual_nodes::add_virtual_nodes(&mut dot.graph, &mut ranks);
    let places = place::places3(&dot.graph, &ranks);
    let coords = xcoord::x_coordinates(&dot.graph, &ranks, &places);
    let mut output = vec![];
    draw::draw(&dot, &ranks, &coords, extra_edges, &mut output);
    output
}

pub fn subgraph<'a>(
    dot: &'a DotGraph,
    opt_start: Option<NodeId>,
    max_nodes: u32,
    max_edges: u32,
) -> (DotGraph<'a>, NodeMap<(u32, u32)>) {
    let start = opt_start
        .or(dot.graph.roots().get(0).copied())
        .expect("need start");
    debug!("subgraph: start from {start:?}");
    //TODO: remember used root
    let mut output = DirectedGraph::default();
    let mut map = dot.graph.node_map::<Option<NodeId>>();
    let mut queue = vec![start];
    let mut next_queue = vec![];
    while !queue.is_empty() && output.nodes_count() < max_nodes && output.edges_count() < max_edges
    {
        let node_id = queue.pop().unwrap();
        if map.get(node_id).is_none() {
            let new_node_id = output.add_node(Node::default());
            debug!("subgraph: add {node_id:?} as {new_node_id:?}");
            map.set(node_id, Some(new_node_id));
            for (_, _, edge, direction) in dot.graph.iter_node_edges(node_id) {
                let (other_id, is_output) = if direction == Direction::Output {
                    (edge.to, true)
                } else {
                    (edge.from, false)
                };
                if let Some(new_other_id) = map.get(other_id).to_owned() {
                    debug!("subgraph: copy edge {edge:?}");
                    // TODO: copy edge params
                    let edge_id = output.add_edge(if is_output {
                        Edge::new(new_node_id, new_other_id)
                    } else {
                        Edge::new(new_other_id, new_node_id)
                    });
                    if is_output {
                        output.node_mut(new_node_id).outputs.push(edge_id);
                        output.node_mut(new_other_id).inputs.push(edge_id);
                    } else {
                        output.node_mut(new_other_id).outputs.push(edge_id);
                        output.node_mut(new_node_id).inputs.push(edge_id);
                    }
                } else {
                    debug!("subgraph: push {other_id:?} to next queue");
                    next_queue.push(other_id);
                }
            }
        }
        if queue.is_empty() {
            debug!("subgraph: next step size {}", next_queue.len());
            std::mem::swap(&mut queue, &mut next_queue);
        }
    }
    info!(
        "subgraph: output {} nodes and {} edges, input {} and {}, limits {} and {}",
        output.nodes_count(),
        output.edges_count(),
        dot.graph.nodes_count(),
        dot.graph.edges_count(),
        max_nodes,
        max_edges
    );
    let mut extra_edges = output.node_map();
    for (id, &new_id_opt) in map.iter() {
        if let Some(new_id) = new_id_opt {
            let node = dot.graph.node(id);
            let new_node = output.node(new_id);
            extra_edges.set(
                new_id,
                (
                    (node.inputs.len() - new_node.inputs.len()) as u32,
                    (node.outputs.len() - new_node.outputs.len()) as u32,
                ),
            );
        }
    }

    // map labels
    (dot.map_to_new(output, map), extra_edges)
}

pub fn rank_with_components<T: Debug>(graph: &DirectedGraph<T>) -> NodeMap<i32> {
    let (components, map) = split_components(graph);
    if components.len() == 1 {
        ns::network_simplex(graph, ns::Postprocess::None, None)
    } else {
        let component_ranks: Vec<_> = components
            .iter()
            .inspect(|c| debug!("{:?}", c))
            .map(|c| ns::network_simplex(c, ns::Postprocess::None, None))
            .collect();
        merge_components(graph, &map, &component_ranks)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NewNodePlace {
    component: u32,
    new_id: NodeId,
}

impl Default for NewNodePlace {
    fn default() -> Self {
        Self {
            component: 0,
            new_id: NodeId::from(0u32),
        }
    }
}

pub fn split_components<T: Debug>(
    graph: &DirectedGraph<T>,
) -> (Vec<DirectedGraph<()>>, NodeMap<NewNodePlace>) {
    debug!(
        "split_components: for DAG with {} nodes",
        graph.nodes_count()
    );

    struct S<'a, T: Debug> {
        graph: &'a DirectedGraph<T>,
        map: NodeMap<NewNodePlace>,
        current: u32,
        components: Vec<DirectedGraph<()>>,
        visited: NodeMap<bool>,
    }
    impl<'a, T: Debug> S<'a, T> {
        fn copy_node(&mut self, id: NodeId) {
            self.visited.set(id, true);
            let node = self.graph.node(id);
            let new_node = Node {
                inputs: Vec::with_capacity(node.inputs.len()),
                outputs: Vec::with_capacity(node.outputs.len()),
                ..*node
            };
            let new_id = self.components[self.current as usize - 1].add_node(new_node);
            self.map.set(
                id,
                NewNodePlace {
                    component: self.current - 1,
                    new_id,
                },
            )
        }

        fn copy_edge(&mut self, edge: &Edge) {
            let NewNodePlace { new_id, component } = self.map.get(edge.from);
            let component = &mut self.components[*component as usize];
            let new_to = self.map.get(edge.to).new_id;
            debug!(
                "split_components: copy edge {edge:?} as {new_id:?} -> {new_to:?} to {component:?}"
            );
            let edge_id = component.add_edge(Edge {
                from: *new_id,
                to: new_to,
                ..*edge
            });
            component.node_mut(*new_id).outputs.push(edge_id);
            component.node_mut(new_to).inputs.push(edge_id);
        }
    }

    let mut queue = vec![];
    let mut s = S {
        graph,
        map: graph.node_map(),
        components: vec![],
        current: 0,
        visited: graph.node_map(),
    };

    for (id, _) in graph.iter_nodes_with_id() {
        if *s.visited.get(id) {
            continue;
        }
        s.current += 1;
        debug!("split_components: start component {}", s.current - 1);
        s.components.push(DirectedGraph::default());
        queue.push(id);
        s.copy_node(id);

        // for all outputs and inputs add reachabe nodes to current component
        while let Some(id) = queue.pop() {
            debug!("split_components: pop {:?}", id);
            let node = graph.node(id);
            for &edge_id in &node.outputs {
                let edge = graph.edge(edge_id);
                if *s.visited.get(edge.to) {
                    continue;
                }
                debug!("split_components: add {:?}", edge);
                queue.push(edge.to);
                s.copy_node(edge.to);
            }

            for &edge_id in &node.inputs {
                let edge = graph.edge(edge_id);
                if *s.visited.get(edge.from) {
                    continue;
                }
                debug!("split_components: add {:?}", edge);
                queue.push(edge.from);
                s.copy_node(edge.from);
            }
        }
    }

    for edge in graph.iter_edges() {
        s.copy_edge(edge);
    }
    // No need to copy self edge becase those doesn't affect Y position.

    debug!("split_components: graph {:?}", graph);
    debug!("split_components: node map {:?}", s.map);
    debug!("split_components: components {:?}", s.components);

    assert_eq!(
        graph.edges_count(),
        s.components.iter().map(|c| c.edges_count()).sum(),
    );
    assert_eq!(
        graph.nodes_count(),
        s.components.iter().map(|c| c.nodes_count()).sum(),
    );

    for &root_id in graph.roots() {
        let new_place = s.map.get(root_id);
        s.components[new_place.component as usize].add_root(new_place.new_id);
    }

    (s.components, s.map)
}

pub fn merge_components<T: Debug>(
    graph: &DirectedGraph<T>,
    map: &NodeMap<NewNodePlace>,
    component_ranks: &[NodeMap<i32>],
) -> NodeMap<i32> {
    let mut ranks = graph.node_map();
    for (node_id, rank) in ranks.iter_mut() {
        let new_place = map.get(node_id);
        *rank = *component_ranks[new_place.component as usize].get(new_place.new_id);
    }
    ranks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_components_test() {
        init_log();
        let graph = DirectedGraph::new(&[0], &[(1, 2), (3, 2), (4, 4), (5, 4), (5, 6), (6, 4)]);
        let (components, map) = split_components(&graph);
        assert_eq!(components.len(), 3);
        assert_eq!(components[0].nodes_count(), 1);
        assert_eq!(components[2].nodes_count(), 3);
        assert_eq!(components[2].nodes_count(), 3);
        assert_eq!(
            map.iter().map(|(_, v)| v).collect::<Vec<_>>(),
            &[
                &NewNodePlace {
                    component: 0,
                    new_id: NodeId::from(0u32)
                },
                &NewNodePlace {
                    component: 1,
                    new_id: NodeId::from(0u32)
                },
                &NewNodePlace {
                    component: 1,
                    new_id: NodeId::from(1u32)
                },
                &NewNodePlace {
                    component: 1,
                    new_id: NodeId::from(2u32)
                },
                &NewNodePlace {
                    component: 2,
                    new_id: NodeId::from(0u32)
                },
                &NewNodePlace {
                    component: 2,
                    new_id: NodeId::from(1u32)
                },
                &NewNodePlace {
                    component: 2,
                    new_id: NodeId::from(2u32)
                },
            ]
        );
    }

    #[test]
    fn rank_with_components_test() {
        let graph = DirectedGraph::new(&[], &[(0, 1), (2, 3)]);
        let ranks = rank_with_components(&graph);
        assert_eq!(
            ranks.iter().map(|(_, &v)| v).collect::<Vec<_>>(),
            &[0, 1, 0, 1]
        );
    }

    #[test]
    fn rank_loop_and_dot() {
        let _ = init_log();
        let graph = DirectedGraph::new(&[0], &[(1, 2), (2, 3), (1, 3)]);
        let ranks = rank_with_components(&graph);
        assert_eq!(
            ranks.iter().map(|(_, &v)| v).collect::<Vec<_>>(),
            &[0, 0, 1, 2]
        );
    }

    #[test]
    fn test_subgraph() {
        let _ = init_log();
        let dot = read_dot::parse("digraph x {a->b; a->c; a->d; a->e; b->f; b->c;}").unwrap();
        let (new, _) = subgraph(&dot, None, 3, 10);
        assert_eq!(new.graph.nodes_count(), 3);
        assert_eq!(new.graph.nodes_count(), 3);
        //TODO: add more cases and checks.
    }

    pub fn init_log() {
        _ = simplelog::TermLogger::init(
            simplelog::LevelFilter::Debug,
            simplelog::Config::default(),
            simplelog::TerminalMode::Mixed,
            simplelog::ColorChoice::Auto,
        );
        //.unwrap()
    }
}
