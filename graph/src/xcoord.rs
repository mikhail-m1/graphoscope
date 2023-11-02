use crate::{graph::*, ns::network_simplex};
use std::fmt::Debug;

pub fn x_coordinates<T: Debug>(
    graph: &DirectedGraph<T>,
    ranks: &NodeMap<i32>,
    places: &NodeMap<u32>,
) -> NodeMap<u32> {
    let mut temp_graph = DirectedGraph::<()>::new(&[], &[]);
    for _ in 0..graph.nodes_count() {
        temp_graph.add_node(Node::virt());
    }

    // TODO: use for each edge?
    for (node_id, _) in graph.iter_nodes_with_id() {
        for child_id in graph.iter_children(node_id) {
            let temp_node_id = temp_graph.add_node(Node::virt());
            let edge_id = temp_graph.add_edge(Edge {
                from: temp_node_id,
                to: node_id,
                kind: EdgeKind::Normal,
                min_length: 1,
                weight: 1,
            });
            temp_graph.node_mut(temp_node_id).outputs.push(edge_id);
            temp_graph.node_mut(node_id).inputs.push(edge_id);

            let edge_id = temp_graph.add_edge(Edge {
                from: temp_node_id,
                to: child_id,
                kind: EdgeKind::Normal,
                min_length: 1,
                weight: 1,
            });
            temp_graph.node_mut(temp_node_id).outputs.push(edge_id);
            temp_graph.node_mut(child_id).inputs.push(edge_id);
            temp_graph.add_root(temp_node_id);
        }
    }

    //TODO: maybe pass layers?
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

    for layer in &layers {
        let mut iter = layer.iter().filter_map(|v| *v).peekable();
        while let Some(id) = iter.next() {
            if let Some(&next) = iter.peek() {
                let edge_id = temp_graph.add_edge(Edge {
                    from: id,
                    to: next,
                    kind: EdgeKind::Normal,
                    min_length: 50,
                    weight: 0,
                });
                temp_graph.node_mut(id).outputs.push(edge_id);
                temp_graph.node_mut(next).inputs.push(edge_id);
            }
        }
    }

    //TODO: need to make additional tweaks to center clusters with slack
    let coordinates = network_simplex(&temp_graph, crate::ns::Postprocess::Center, None);

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
