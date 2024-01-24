use crate::{graph::*, ns::network_simplex};
use std::fmt::Debug;

pub fn x_coordinates<T: Debug>(
    graph: &DirectedGraph<T>,
    ranks: &NodeMap<i32>,
    places: &NodeMap<u32>,
) -> NodeMap<u32> {
    let node_width = 50;
    let mut temp_graph = DirectedGraph::<()>::new(&[], &[]);
    debug!("x_coord: Input graph has {} nodes", graph.nodes_count());

    // Each edge will be replaced by node and two edges.
    let mut left_right_ranks = NodeMap::new(graph.nodes_count() + graph.edges_count());

    //TODO: pass layers
    let mut layers = vec![];
    for (id, _) in graph.iter_nodes_with_id() {
        let rank = *ranks.get(id) as usize;
        if layers.len() <= rank {
            layers.resize(rank + 1, vec![]);
        }
        let layer = &mut layers[rank];
        let place = *places.get(id) as usize;
        if layer.len() <= place {
            layer.resize(place + 1, None);
        }
        layer[place] = Some(id)
    }

    for _ in 0..graph.nodes_count() {
        temp_graph.add_node(Node::virt()); //TODO: rewrite to real node
    }

    // Links nodes on the same level.
    for layer in &layers {
        let mut iter = layer.iter().filter_map(|v| *v).enumerate().peekable();
        while let Some((index, id)) = iter.next() {
            left_right_ranks.set(id, index as i32 * node_width as i32);
            if let Some(&(_, next)) = iter.peek() {
                let edge_id = temp_graph.add_edge(Edge {
                    from: id,
                    to: next,
                    kind: EdgeKind::Normal,
                    min_length: node_width, // TODO: use real node width
                    weight: 0,
                });
                temp_graph.node_mut(id).outputs.push(edge_id);
                temp_graph.node_mut(next).inputs.push(edge_id);
            }
        }
    }

    // Create auxiliary nodes and edges to replace input edges.
    for edge in graph.iter_edges() {
        let temp_node_id = temp_graph.add_node(Node::virt());
        debug!(
            "x_coord: add node {temp_node_id:?} for edge {:?} {:?} {:?}",
            edge,
            graph.original_id(edge.from),
            graph.original_id(edge.to)
        );
        let edge_id = temp_graph.add_edge(Edge {
            from: temp_node_id,
            to: edge.from,
            kind: EdgeKind::Normal,
            min_length: 1,
            weight: edge.weight,
        });
        temp_graph.node_mut(temp_node_id).outputs.push(edge_id);
        temp_graph.node_mut(edge.from).inputs.push(edge_id);

        let edge_id = temp_graph.add_edge(Edge {
            from: temp_node_id,
            to: edge.to,
            kind: EdgeKind::Normal,
            min_length: 1,
            weight: edge.weight,
        });
        temp_graph.node_mut(temp_node_id).outputs.push(edge_id);
        temp_graph.node_mut(edge.to).inputs.push(edge_id);
        debug!(
            "check for root, left {}, right {}, min {}",
            ranks.get(edge.from),
            ranks.get(edge.to),
            *ranks.get(edge.from).min(ranks.get(edge.to))
        );

        let min_rank = *left_right_ranks
            .get(edge.from)
            .min(left_right_ranks.get(edge.to));
        left_right_ranks.set(temp_node_id, min_rank - 1);
        if min_rank == 0 {
            // TODO: check how it affects ns, previously all auxiliary nodes were added.
            debug!("add root: {:?}", temp_node_id);
            temp_graph.add_root(temp_node_id);
        }
    }

    let coordinates = network_simplex(
        &temp_graph,
        crate::ns::Postprocess::Center,
        Some(left_right_ranks),
    );

    //TODO: maybe just return coordinates or downsize it?
    let mut result = graph.node_map();
    let min = result
        .iter()
        .map(|(id, _)| coordinates.get(id))
        .min()
        .unwrap();
    for (id, value) in result.iter_mut() {
        *value = (*coordinates.get(id) - min) as u32
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple() {
        let graph = DirectedGraph::new(&[0, 1, 2], &[(0, 1), (0, 2)]);
        let mut ranks = graph.node_map();
        ranks.set(NodeId::from(1u32), 1);
        ranks.set(NodeId::from(2u32), 1);
        let mut places = graph.node_map();
        places.set(NodeId::from(2u32), 1);
        let xs = x_coordinates(&graph, &ranks, &places);
        assert_eq!(*xs.get(NodeId::from(0u32)), 25);
        assert_eq!(*xs.get(NodeId::from(1u32)), 0);
        assert_eq!(*xs.get(NodeId::from(2u32)), 50);
    }

    #[test]
    fn simple2() {
        let graph = DirectedGraph::new(&[0, 1, 2, 3], &[(0, 1), (0, 2), (0, 3)]);
        let mut ranks = graph.node_map();
        ranks.set(NodeId::from(1u32), 1);
        ranks.set(NodeId::from(2u32), 1);
        ranks.set(NodeId::from(3u32), 1);
        let mut places = graph.node_map();
        places.set(NodeId::from(2u32), 1);
        places.set(NodeId::from(3u32), 2);
        let xs = x_coordinates(&graph, &ranks, &places);
        assert_eq!(*xs.get(NodeId::from(0u32)), 50);
        assert_eq!(*xs.get(NodeId::from(1u32)), 0);
        assert_eq!(*xs.get(NodeId::from(2u32)), 50);
        assert_eq!(*xs.get(NodeId::from(3u32)), 100);
    }
}
