use std::fmt::Debug;
use std::mem::swap;

use crate::graph::*;

enum Action {
    EnterToRoot(NodeId),
    Enter {
        to_node_id: NodeId,
        from_node_it: NodeId,
        edge_id: EdgeId,
    },
    Leave(NodeId),
}

pub fn to_dag<T: Debug>(graph: &mut DirectedGraph<T>) {
    let mut visited_count = 0;
    let mut visited = graph.node_map::<bool>();
    let mut path = graph.node_map::<bool>();
    let mut stack: Vec<_> = graph
        .roots()
        .iter()
        .map(|&n| Action::EnterToRoot(n))
        .collect();

    while visited_count != graph.nodes_count() {
        if stack.is_empty() {
            // convert first unvisited node to the root by reveting all input edges
            let first_unvisited = visited.find_first(|v| !v).unwrap();
            stack.push(Action::EnterToRoot(first_unvisited));
            let first_node = graph.node_mut(first_unvisited);
            let mut old_inputs = vec![];
            swap(&mut old_inputs, &mut first_node.inputs);
            first_node.outputs.extend(&old_inputs);
            for &edge_id in old_inputs.iter() {
                let edge = graph.edge_mut(edge_id);
                let from_id = edge.from;
                edge.invert();
                graph.node_mut(from_id).to_input(edge_id);
            }
            graph.add_root(first_unvisited);
        }

        while let Some(action) = stack.pop() {
            match action {
                Action::Leave(id) => path.set(id, false),
                Action::EnterToRoot(to_node_id) => {
                    //TODO: think how to merge code with Enter, create function or lambda
                    visited.set(to_node_id, true);
                    path.set(to_node_id, true);
                    visited_count += 1;
                    stack.push(Action::Leave(to_node_id));
                    stack.extend(
                        graph
                            .node(to_node_id)
                            .outputs
                            .iter()
                            .map(|&e| Action::Enter {
                                to_node_id: graph.edge(e).to,
                                from_node_it: graph.edge(e).from,
                                edge_id: e,
                            }),
                    )
                }
                Action::Enter {
                    to_node_id,
                    edge_id,
                    from_node_it,
                } => {
                    // invert edge if saw on the path
                    if *visited.get(to_node_id) {
                        if *path.get(to_node_id) {
                            graph.edge_mut(edge_id).invert();
                            graph.node_mut(to_node_id).to_output(edge_id);
                            graph.node_mut(from_node_it).to_input(edge_id);
                        }
                        continue;
                    }
                    visited.set(to_node_id, true);
                    path.set(to_node_id, true);
                    visited_count += 1;
                    stack.push(Action::Leave(to_node_id));
                    stack.extend(
                        graph
                            .node(to_node_id)
                            .outputs
                            .iter()
                            .map(|&e| Action::Enter {
                                to_node_id: graph.edge(e).to,
                                from_node_it: graph.edge(e).from,
                                edge_id: e,
                            }),
                    )
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dag_no_changes() {
        let mut dag = DirectedGraph::new(&['a', 'b'], &[('a', 'b')]);
        to_dag(&mut dag);
        dag.assert(
            &[Node::with_outputs(&[0]), Node::with_inputs(&[0])],
            &[Edge::new(NodeId::from(0u32), NodeId::from(1u32))],
        );
    }

    #[test]
    fn loop_to_dag() {
        let mut dag = DirectedGraph::new(&['a', 'b'], &[('a', 'b'), ('b', 'a')]);
        to_dag(&mut dag);
        dag.assert(
            &[Node::with_outputs(&[0, 1]), Node::with_inputs(&[0, 1])],
            &[
                Edge::new(NodeId::from(0u32), NodeId::from(1u32)),
                Edge::new_inverted(NodeId::from(0u32), NodeId::from(1u32)),
            ],
        );
    }

    #[test]
    fn loop_with_input_to_dag() {
        let mut dag = DirectedGraph::new(&['a', 'b', 'c'], &[('a', 'b'), ('b', 'c'), ('c', 'b')]);
        to_dag(&mut dag);
        dag.assert(
            &[
                Node::with_outputs(&[0]),
                Node::with_both(&[0], &[1, 2]),
                Node::with_inputs(&[1, 2]),
            ],
            &[
                Edge::new(NodeId::from(0u32), NodeId::from(1u32)),
                Edge::new(NodeId::from(1u32), NodeId::from(2u32)),
                Edge::new_inverted(NodeId::from(1u32), NodeId::from(2u32)),
            ],
        );
    }

    #[test]
    fn loop_and_input_to_shared_node_to_dag() {
        let mut dag = DirectedGraph::new(
            &['a', 'b', 'c', 'd'],
            &[('a', 'b'), ('c', 'd'), ('d', 'c'), ('c', 'b')],
        );
        to_dag(&mut dag);
        dag.assert(
            &[
                Node::with_outputs(&[0]),
                Node::with_inputs(&[0, 3]),
                Node::with_outputs(&[1, 3, 2]),
                Node::with_inputs(&[1, 2]),
            ],
            &[
                Edge::new(NodeId::from(0u32), NodeId::from(1u32)),
                Edge::new(NodeId::from(2u32), NodeId::from(3u32)),
                Edge::new_inverted(NodeId::from(2u32), NodeId::from(3u32)),
                Edge::new(NodeId::from(2u32), NodeId::from(1u32)),
            ],
        );
    }
}
