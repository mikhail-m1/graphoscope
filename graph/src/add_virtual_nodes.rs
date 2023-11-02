use crate::graph::*;
use std::fmt::Debug;

pub fn add_virtual_nodes<T: Debug>(graph: &mut DirectedGraph<T>, ranks: &mut NodeMap<i32>) {
    graph.for_each_edge_mut(&mut |graph, mut edge_id| {
        let edge = graph.edge(edge_id);
        let is_inverted = edge.kind == EdgeKind::Inverted;
        let from_id = edge.from;
        let to_id = edge.to;
        let mut from_rank = *ranks.get(from_id);
        let to_node = graph.node(to_id);
        let to_rank = *ranks.get(to_id);
        let to_edge_position = to_node.inputs.iter().position(|&e| e == edge_id).unwrap();
        debug_assert!(
            from_rank < to_rank,
            "edge {:?} invalid ranks {} {}",
            edge,
            from_rank,
            to_rank
        );
        while from_rank + 1 < to_rank {
            let node_id = graph.add_node(Node {
                inputs: vec![edge_id],
                outputs: Vec::with_capacity(1),
                is_virtual: true,
            });
            debug!("add_virtual_nodes: new node {node_id:?} for link {from_id:?} {to_id:?}");
            ranks.set(node_id, from_rank + 1);

            graph.edge_mut(edge_id).to = node_id;
            let new_edge_id = graph.add_edge(if is_inverted {
                Edge::new_inverted(node_id, to_id)
            } else {
                Edge::new(node_id, to_id)
            });
            graph.node_mut(node_id).outputs.push(new_edge_id);
            from_rank += 1;

            if from_rank + 1 == to_rank {
                graph.node_mut(to_id).inputs[to_edge_position] = new_edge_id;
            }
            edge_id = new_edge_id;
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_chages() {
        let mut dag = DirectedGraph::new(&['a', 'b'], &[('a', 'b')]);
        let mut ranks = dag.node_map();
        ranks.set(NodeId::from(0u32), 0);
        ranks.set(NodeId::from(1u32), 1);
        add_virtual_nodes(&mut dag, &mut ranks);
        dag.assert(
            &[Node::with_outputs(&[0]), Node::with_inputs(&[0])],
            &[Edge::new(NodeId::from(0u32), NodeId::from(1u32))],
        );
    }

    #[test]
    fn simple() {
        let mut dag = DirectedGraph::new(&['a', 'b'], &[('a', 'b')]);
        let mut ranks = dag.node_map();
        ranks.set(NodeId::from(0u32), 0);
        ranks.set(NodeId::from(1u32), 3);
        add_virtual_nodes(&mut dag, &mut ranks);
        assert_ranks(&ranks, &[0, 3, 1, 2]);
        dag.assert(
            &[
                Node::with_outputs(&[0]),
                Node::with_inputs(&[2]),
                Node::with_both(&[0], &[1]),
                Node::with_both(&[1], &[2]),
            ],
            &[
                Edge::new(NodeId::from(0u32), NodeId::from(2u32)),
                Edge::new(NodeId::from(2u32), NodeId::from(3u32)),
                Edge::new(NodeId::from(3u32), NodeId::from(1u32)),
            ],
        );
    }

    #[test]
    fn inverted() {
        let mut dag = DirectedGraph::new(&['a', 'b'], &[('a', 'b')]);
        let mut ranks = dag.node_map();
        ranks.set(NodeId::from(0u32), 0);
        ranks.set(NodeId::from(1u32), 2);
        dag.edge_mut(EdgeId::from(0u32)).kind = EdgeKind::Inverted;
        add_virtual_nodes(&mut dag, &mut ranks);
        assert_ranks(&ranks, &[0, 2, 1]);
        dag.assert(
            &[
                Node::with_outputs(&[0]),
                Node::with_inputs(&[1]),
                Node::with_both(&[0], &[1]),
            ],
            &[
                Edge::new_inverted(NodeId::from(0u32), NodeId::from(2u32)),
                Edge::new_inverted(NodeId::from(2u32), NodeId::from(1u32)),
            ],
        );
    }

    fn assert_ranks(result: &NodeMap<i32>, expected: &[i32]) {
        assert_eq!(result.iter().map(|(_, &v)| v).collect::<Vec<_>>(), expected);
    }
}
