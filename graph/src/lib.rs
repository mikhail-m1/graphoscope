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

pub fn full_draw<'a>(mut dot: DotGraph<'a>) -> Vec<u8> {
    to_dag::to_dag(&mut dot.graph);
    let mut ranks = rank_with_components(&dot.graph);
    add_virtual_nodes::add_virtual_nodes(&mut dot.graph, &mut ranks);
    let places = place::places3(&dot.graph, &ranks);
    let coords = xcoord::x_coordinates(&dot.graph, &ranks, &places);
    let mut output = vec![];
    draw::draw(&dot, &ranks, &coords, &mut output);
    output
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

    debug!("split_components: {:?}", graph);
    debug!("split_components: {:?}", s.map);
    debug!("split_components: {:?}", s.components);

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
