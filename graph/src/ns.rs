use crate::graph::*;
use std::{fmt::Debug, mem};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Postprocess {
    None,
    Center,
    //MinPerRank
}

pub fn network_simplex<T: Debug>(
    graph: &DirectedGraph<T>,
    postprocess: Postprocess,
    opt_ranks: Option<NodeMap<i32>>,
) -> NodeMap<i32> {
    let mut ranks = opt_ranks.unwrap_or(rank(&graph));
    debug!("initial ranks: {ranks:?}");
    graph.dump(
        &format!("tmp_ns_before_start_{postprocess:?}"),
        &|_, id, _| format!("{:?} r:{}", id, ranks.get(id)).into(),
        &|_, _, edge| format!("weight:{} min_len:{}", edge.weight, edge.min_length,).into(),
        &|_, _| true,
        &|_, _| true,
    );

    _validate_rank(graph, &ranks);
    let mut data = &mut span_tree(&graph, postprocess, &mut ranks);
    _validate_rank(graph, &data.ranks);
    _validate_length(graph, &data.ranks, &data.edges);
    minmax(data);
    cut_value(data);

    let mut iter = 0;
    let mut negative_edge_search = NegativeEdgeSearch::new();
    while let Some(negative_edge) = negative_edge_search.next(&data) {
        dump_graph(&data, iter, "replace_ready", Some(negative_edge));
        debug!(
            "search replacment for {:?} {:?}",
            negative_edge,
            data.graph.edge(negative_edge)
        );
        let replacement_edge = find_replacement_edge(&mut data, negative_edge);
        debug!(
            "replace negative {:?} by {:?}",
            graph.edge(negative_edge),
            graph.edge(replacement_edge)
        );
        replace_edge(&mut data, negative_edge, replacement_edge);
        dump_graph(&data, iter, "replace_done", Some(replacement_edge));
        _validate_rank(graph, data.ranks);
        iter += 1;
    }

    normalize(data.ranks); // TODO: optimize, we can do it once
    debug!("before post process ranks: {:?}", data.ranks);

    if postprocess == Postprocess::Center {
        for edge_id in data.edges.iter_ids() {
            let edge_data = data.edges.get(edge_id);
            if !edge_data.in_tree || edge_data.cut_value != 0 {
                continue;
            }

            let replacement_edge = find_replacement_edge(&mut data, edge_id);
            let shift_ = (data
                .edges
                .get(replacement_edge)
                .slack(graph.edge(replacement_edge))
                / 2) as i32;
            debug!(
                "center balance {:?} {:?} & {:?} {:?} shift {shift_}",
                edge_id,
                graph.edge(edge_id),
                replacement_edge,
                graph.edge(replacement_edge)
            );
            if shift_ != 0 {
                shift(&mut data, edge_id, shift_);
            }
        }
    }
    normalize(data.ranks);
    debug!("after post process ranks: {:?}", data.ranks);
    dump_graph(data, 0, "ns_done", None);
    ranks
}

fn normalize(ranks: &mut NodeMap<i32>) {
    let min_rank = ranks.iter().map(|(_, &d)| d).min().unwrap();
    if min_rank != 0 {
        for (_, data) in ranks.iter_mut() {
            *data -= min_rank;
        }
    }
}

fn rank<T: Debug>(graph: &DirectedGraph<T>) -> NodeMap<i32> {
    let mut stack: Vec<_> = graph.roots().iter().copied().collect();
    let mut found_inputs = graph.node_map::<u32>();
    let mut ranks = graph.node_map();

    while let Some(id) = stack.pop() {
        debug!("rank: process {id:?}");
        let node = graph.node(id);
        let opt_rank = node
            .inputs
            .iter()
            .map(|&e| *ranks.get(graph.edge(e).from) + graph.edge(e).min_length as i32)
            .max(); //bug for same level input nodes (but they are not supported)
        for &edge_id in &node.outputs {
            let edge = graph.edge(edge_id);
            debug!(
                "rank: found {:?}, inputs = {}, fount = {}",
                edge.to,
                graph.node(edge.to).inputs.len(),
                *found_inputs.get(edge.to) + 1
            );
            if graph.node(edge.to).inputs.len() - 1 == *found_inputs.get(edge.to) as usize {
                stack.push(edge.to)
            } else {
                *found_inputs.get_mut(edge.to) += 1;
            }
        }
        if let Some(rank) = opt_rank {
            ranks.set(id, rank);
        }
    }
    ranks
}

#[derive(Debug)]
struct NetworkSimplexData<'a, 'b, T> {
    graph: &'a DirectedGraph<T>,
    postprocess: Postprocess,
    ranks: &'b mut NodeMap<i32>,
    root: NodeId, // TODO: delete
    nodes: NodeMap<NetworkSimplexNodeData>,
    edges: EdgeMap<NetworkSimplexEdgeData>,
}

#[derive(Clone, Default, Debug)]
struct NetworkSimplexEdgeData {
    length: u32,
    cut_value: i32,
    in_tree: bool,
}

impl NetworkSimplexEdgeData {
    fn slack(&self, edge: &Edge) -> u32 {
        assert!(
            self.length >= edge.min_length,
            " for edge {:?} {:?}",
            edge,
            self
        );
        self.length - edge.min_length
    }
}

#[derive(Clone, Default, Debug)]
struct NetworkSimplexNodeData {
    min: u32,
    max: u32,
    inputs: Vec<EdgeId>, // TODO: maybe store just NodeId
    outputs: Vec<EdgeId>,
}

fn fast_delete_single_item<T: Eq>(vec: &mut Vec<T>, value: T) {
    let pos = vec
        .iter()
        .position(|v| *v == value)
        .expect("value to delete");
    vec.swap_remove(pos);
}

impl NetworkSimplexNodeData {
    fn inside(&self, min: u32, max: u32) -> bool {
        min <= self.max && self.max <= max
    }
    fn remove_output(&mut self, edge_id: EdgeId) {
        fast_delete_single_item(&mut self.outputs, edge_id);
    }
    fn remove_input(&mut self, edge_id: EdgeId) {
        fast_delete_single_item(&mut self.inputs, edge_id);
    }
}

fn span_tree<'a, 'b, T: Debug>(
    graph: &'a DirectedGraph<T>,
    postprocess: Postprocess,
    ranks: &'b mut NodeMap<i32>,
) -> NetworkSimplexData<'a, 'b, T> {
    let mut edge_data = graph.edge_map::<NetworkSimplexEdgeData>();
    let mut node_data = graph.node_map::<NetworkSimplexNodeData>();

    // should be part of initial ranking
    for (edge_id, edge_data) in edge_data.iter_mut() {
        let edge = graph.edge(edge_id);
        edge_data.length = (ranks.get(edge.to) - ranks.get(edge.from)) as u32;
    }

    let mut tree = graph.node_map::<u32>(); // number of the tree, 0 - not visited
    let mut stack = vec![];
    let mut current_tree = 0;

    //create set of tight trees
    for (id, _) in graph.iter_nodes_with_id() {
        if *tree.get(id) != 0 {
            continue;
        }
        stack.push(id);
        current_tree += 1;
        tree.set(id, current_tree);
        debug!("span_tree: start {} from  {:?}", current_tree, id);

        // for all outputs and inputs add nodes with slack = 0 to the tree
        while let Some(id) = stack.pop() {
            for (_, edge_id, edge, direction) in graph.iter_node_edges(id) {
                let other_side_id = edge.other_side(direction);
                if *tree.get(other_side_id) != 0 {
                    continue;
                }
                let slack = edge_data.get(edge_id).slack(edge);
                if slack != 0 {
                    continue;
                }
                tree.set(other_side_id, current_tree);
                debug!("span_tree: add {other_side_id:?} to tree {current_tree} through {edge:?}");
                stack.push(other_side_id);
                edge_data.get_mut(edge_id).in_tree = true;
                if direction == Direction::Output {
                    node_data.get_mut(id).outputs.push(edge_id);
                    node_data.get_mut(other_side_id).inputs.push(edge_id);
                } else {
                    node_data.get_mut(id).inputs.push(edge_id);
                    node_data.get_mut(other_side_id).outputs.push(edge_id);
                }
            }
        }
    }
    //merge trees
    while current_tree != 1 {
        graph.dump(
            &format!("tmp_span_tree_join_{current_tree}"),
            &|_, id, _| format!("{:?} T:{} r:{}", id, tree.get(id), ranks.get(id)).to_string(),
            &|_, _, _| "".into(),
            &|_, _| true,
            &|_, id| edge_data.get(id).in_tree,
        );

        _validate_length(graph, &ranks, &edge_data);
        // search for min edge
        let mut min_edge_id = None;
        let mut min_edge_slack = u32::MAX;
        for id in tree.iter_ids().filter(|&id| *tree.get(id) == 1) {
            for (_, edge_id, edge, direction) in graph.iter_node_edges(id) {
                if *tree.get(edge.other_side(direction)) == 1 {
                    continue;
                }
                debug!("span_tree: process {:?}", edge);
                let slack = edge_data.get_mut(edge_id).slack(edge);
                if slack < min_edge_slack {
                    min_edge_id = Some(edge_id);
                    min_edge_slack = slack;
                }
                //TODO: can we break if have slack = 1? other edges' length still unupdated
            }
        }
        assert!(min_edge_id.is_some());

        let min_edge_id = min_edge_id.unwrap();

        let edge = graph.edge(min_edge_id);
        let &to_tree = tree.get(edge.to);
        let &from_tree = tree.get(edge.from);
        let slack = min_edge_slack as i32;
        let (new_tree, slack) = if from_tree == 1 {
            (
                to_tree,
                if ranks.get(edge.from) < ranks.get(edge.to) {
                    slack
                } else {
                    -slack
                },
            )
        } else {
            (
                from_tree,
                if ranks.get(edge.to) < ranks.get(edge.from) {
                    slack
                } else {
                    -slack
                },
            )
        };

        debug!(
            "span_tree: merge tree {} and {} with {:?} slack {}",
            from_tree, to_tree, edge, slack
        );
        assert_ne!(from_tree, to_tree);

        // merge trees
        node_data.get_mut(edge.to).inputs.push(min_edge_id);
        node_data.get_mut(edge.from).outputs.push(min_edge_id);
        edge_data.get_mut(min_edge_id).in_tree = true;

        // shift first tree nodes, mark new
        // TODO: join with shift
        for (id, _) in graph.iter_nodes_with_id() {
            let v = tree.get_mut(id);
            if *v == 1 {
                *ranks.get_mut(id) += slack;
            } else if *v == new_tree {
                *v = 1;
            } else {
                continue;
            }
            for (_, edge_id, edge, _direction) in graph.iter_node_edges(id) {
                //TODO: it doesn't work, we need separate loops or delete length at all
                //if *tree.get(edge.other_side(direction)) == 1 {
                //    continue;
                //}
                edge_data.get_mut(edge_id).length =
                    (ranks.get(edge.to) - ranks.get(edge.from)) as u32;
            }
        }
        current_tree -= 1;
    }

    debug!("span_tree: done");

    NetworkSimplexData::<'a, 'b, T> {
        graph,
        ranks,
        postprocess,
        nodes: node_data,
        edges: edge_data,
        root: graph.iter_nodes_with_id().next().unwrap().0,
    }
}

fn minmax<'a, 'b, T: Debug>(data: &mut NetworkSimplexData<'a, 'b, T>) {
    fn imp<T: Debug>(
        graph: &DirectedGraph<T>,
        data: &mut NodeMap<NetworkSimplexNodeData>,
        node_id: NodeId,
        from: EdgeId,
        n: &mut u32,
    ) {
        data.get_mut(node_id).min = *n;

        // Hack to mutate data in recursive function, maybe extract min/max to another nodemap
        let mut temp = mem::take(&mut data.get_mut(node_id).outputs);
        for &edge_id in temp.iter().filter(|&&id| id != from) {
            imp(graph, data, graph.edge(edge_id).to, edge_id, n);
        }
        mem::swap(&mut temp, &mut data.get_mut(node_id).outputs);

        let mut temp = mem::take(&mut data.get_mut(node_id).inputs);
        for &edge_id in temp.iter().filter(|&&id| id != from) {
            imp(graph, data, graph.edge(edge_id).from, edge_id, n);
        }
        mem::swap(&mut temp, &mut data.get_mut(node_id).inputs);
        data.get_mut(node_id).max = *n;
        *n += 1;
    }
    let mut n = 1;
    imp(
        data.graph,
        &mut data.nodes,
        data.graph.iter_nodes_with_id().next().unwrap().0,
        UNEXISTED_EDGE_ID,
        &mut n,
    );
}

fn cut_value<'a, 'b, T: Debug>(data: &mut NetworkSimplexData<'a, 'b, T>) {
    fn edge_value(edge: &Edge, node_id: NodeId, data: &NetworkSimplexEdgeData) -> i32 {
        let value = if data.in_tree {
            //FIXME need to check only own subtree
            data.cut_value - edge.weight
        } else {
            -edge.weight
        };
        if edge.to == node_id {
            value
        } else {
            -value
        }
    }

    #[derive(PartialEq, Eq)]
    enum Direction {
        Down,
        Up,
    }

    fn imp<T: Debug>(
        graph: &DirectedGraph<T>,
        data: &NodeMap<NetworkSimplexNodeData>,
        current_node_id: NodeId,
        from: EdgeId,
        edge_data: &mut EdgeMap<NetworkSimplexEdgeData>,
    ) {
        if from != UNEXISTED_EDGE_ID {
            //need for second runs, can be optimized
            edge_data.get_mut(from).cut_value = 0;
        }
        let node_data = data.get(current_node_id);

        for &edge_id in node_data.outputs.iter().filter(|&&id| id != from) {
            imp(graph, data, graph.edge(edge_id).to, edge_id, edge_data);
        }
        for &edge_id in node_data.inputs.iter().filter(|&&id| id != from) {
            imp(graph, data, graph.edge(edge_id).from, edge_id, edge_data);
        }
        if from != UNEXISTED_EDGE_ID {
            let edge = graph.edge(from);
            let (processed_side, direction) = if current_node_id != edge.to {
                (edge.from, Direction::Up)
            } else {
                (edge.to, Direction::Down)
            };

            let node = graph.node(processed_side);

            let value = node
                .inputs
                .iter()
                .map(|&edge_id| {
                    edge_value(graph.edge(edge_id), processed_side, edge_data.get(edge_id))
                })
                .sum::<i32>()
                + node
                    .outputs
                    .iter()
                    .map(|&edge_id| {
                        edge_value(graph.edge(edge_id), processed_side, edge_data.get(edge_id))
                    })
                    .sum::<i32>();

            edge_data.get_mut(from).cut_value = if direction == Direction::Up {
                value
            } else {
                -value
            };
        }
    }
    imp(
        data.graph,
        &data.nodes,
        data.root,
        UNEXISTED_EDGE_ID,
        &mut data.edges,
    );
}

struct NegativeEdgeSearch {
    last: Option<EdgeId>,
}

impl NegativeEdgeSearch {
    fn new() -> Self {
        Self { last: None }
    }
    fn next<'a, 'b, T: Debug>(&mut self, data: &NetworkSimplexData<'a, 'b, T>) -> Option<EdgeId> {
        self.last = data
            .graph
            .iter_edges_with_last(self.last)
            // .take(30) // TODO: create constant
            .map(|id| (id, data.edges.get(id)))
            .filter(|(_, d)| d.in_tree && d.cut_value < 0)
            .min_by(|(_, d1), (_, d2)| d1.cut_value.cmp(&d2.cut_value))
            .map(|(id, _)| id);
        self.last
    }
}

fn find_replacement_edge<'a, 'b, T: Debug>(
    data: &NetworkSimplexData<'a, 'b, T>,
    edge_id: EdgeId,
) -> EdgeId {
    struct S<'a, 'b, T: Debug> {
        data: &'b NetworkSimplexData<'a, 'b, T>,
        direction: Direction,
        min_slack: u32,
        min_edge: EdgeId,
        tree_min: u32,
        tree_max: u32,
    }

    impl<'a, 'b, T: Debug> S<'a, 'b, T> {
        fn f(&mut self, node_id: NodeId, edge_id: EdgeId) -> &mut Self {
            // DSF substree for edge with min slack to the main tree
            for (_, candidate_edge_id, edge, direction) in self.data.graph.iter_node_edges(node_id)
            {
                if self.direction == direction
                    && !self
                        .data
                        .nodes
                        .get(edge.other_side(direction))
                        .inside(self.tree_min, self.tree_max)
                {
                    let slack = self.data.edges.get(candidate_edge_id).slack(edge);
                    debug!(
                        " replacement: check {:?} slack {}, current min {}",
                        edge, slack, self.min_slack
                    );
                    if slack < self.min_slack {
                        self.min_slack = slack;
                        self.min_edge = candidate_edge_id;
                        if slack == 0 {
                            return self;
                        }
                    }
                } else if candidate_edge_id != edge_id
                    && self.data.edges.get(candidate_edge_id).in_tree
                {
                    self.f(edge.other_side(direction), candidate_edge_id);
                }
            }
            self
        }
    }

    let edge = data.graph.edge(edge_id);
    let (direction, subtree_root) = if data.nodes.get(edge.to).max < data.nodes.get(edge.from).max {
        // edge to the subtree, neet to find from subtree
        (Direction::Output, edge.to)
    } else {
        // edge from the subtree, need to find to the subtree
        (Direction::Input, edge.from)
    };
    let node_data = data.nodes.get(subtree_root);
    S {
        data,
        direction,
        min_slack: u32::MAX,
        min_edge: edge_id,
        tree_min: node_data.min,
        tree_max: node_data.max,
    }
    .f(subtree_root, edge_id)
    .min_edge
}

fn replace_edge<'a, 'b, T: Debug>(
    data: &mut NetworkSimplexData<'a, 'b, T>,
    from_id: EdgeId,
    to_id: EdgeId,
) {
    assert_ne!(from_id, to_id);
    let from_edge_data = data.edges.get_mut(from_id);
    from_edge_data.in_tree = false;
    from_edge_data.cut_value = 0;
    data.edges.get_mut(to_id).in_tree = true;
    let from = data.graph.edge(from_id);
    let to = data.graph.edge(to_id);
    data.nodes.get_mut(from.from).remove_output(from_id);
    data.nodes.get_mut(from.to).remove_input(from_id);
    data.nodes.get_mut(to.from).outputs.push(to_id);
    data.nodes.get_mut(to.to).inputs.push(to_id);

    let slack = data.edges.get(to_id).slack(to) as i32;
    if slack > 0 {
        shift(data, from_id, slack);
    }
    // TODO: possible to optimize, don't need to calculate for all
    minmax(data);
    cut_value(data);
}

// adds shift to edge length and adjust affected nodes of the subtree
// TODO: always shift subtree but can be optimized to move part with min nodes count
fn shift<'a, 'b, T: Debug>(
    data: &mut NetworkSimplexData<'a, 'b, T>,
    start_edge_id: EdgeId,
    shift: i32,
) {
    let start_edge = data.graph.edge(start_edge_id);
    let from = data.nodes.get(start_edge.from);
    let to = data.nodes.get(start_edge.to);

    let (min, max, sub_root, other) = if from.max > to.max {
        // Top down
        (to.min, to.max, start_edge.to, start_edge.from)
    } else {
        // Bottom Up
        (from.min, from.max, start_edge.from, start_edge.to)
    };

    let node_shift = if data.ranks.get(sub_root) > data.ranks.get(other) {
        shift //move subtree down
    } else {
        -shift // up
    };
    debug!(
        "shift: edge {:?} nodes min {} max {} for {}, subroot {:?}",
        start_edge, min, max, node_shift, sub_root,
    );

    for (node_id, node_data) in data.nodes.iter() {
        if node_data.inside(min, max) {
            *data.ranks.get_mut(node_id) += node_shift;
            for edge_id in data.graph.node(node_id).edges() {
                let edge = data.graph.edge(edge_id);
                data.edges.get_mut(edge_id).length =
                    (data.ranks.get(edge.to) - data.ranks.get(edge.from)) as u32;
            }
        }
    }
}

#[cfg(debug_assertions)]
fn _validate_rank<T: Debug>(graph: &DirectedGraph<T>, ranks: &NodeMap<i32>) {
    for (_, edge) in graph.iter_edges_with_id() {
        if ranks.get(edge.to) - ranks.get(edge.from) < 1 {
            panic!(
                "rank validate failed, {:?}:{:?}:{} to {:?}:{:?}:{}",
                edge.from,
                graph.original_id(edge.from),
                ranks.get(edge.from),
                edge.to,
                graph.original_id(edge.to),
                ranks.get(edge.to)
            );
        }
    }
}
#[cfg(not(debug_assertions))]
fn _validate_rank<T: Debug>(_: &DirectedGraph<T>, _: &NodeMap<i32>) {}

#[cfg(debug_assertions)]
fn _validate_length<T: Debug>(
    graph: &DirectedGraph<T>,
    ranks: &NodeMap<i32>,
    edge_data: &EdgeMap<NetworkSimplexEdgeData>,
) {
    for (edge_id, edge) in graph.iter_edges_with_id() {
        if ranks.get(edge.to) - ranks.get(edge.from) != edge_data.get(edge_id).length as i32 {
            panic!(
                "length {} validate failed, {:?}:{:?}:{} to {:?}:{:?}:{}",
                edge_data.get(edge_id).length,
                edge.from,
                graph.original_id(edge.from),
                ranks.get(edge.from),
                edge.to,
                graph.original_id(edge.to),
                ranks.get(edge.to)
            );
        }
    }
}

#[cfg(not(debug_assertions))]
fn _validate_length<T: Debug>(
    _: &DirectedGraph<T>,
    _: &NodeMap<i32>,
    _: &EdgeMap<NetworkSimplexEdgeData>,
) {
}

fn dump_graph<'a, 'b, T>(
    data: &NetworkSimplexData<'a, 'b, T>,
    iter: u32,
    place: &str,
    special_edge: Option<EdgeId>,
) {
    data.graph.dump(
        &format!("tmp_ns_{:?}_{place}_{iter}", data.postprocess,),
        &|_, id, _| format!("{:?} r:{}", id, data.ranks.get(id)).into(),
        &|_, id, edge| {
            format!(
                "cut:{} slack:{} len:{} weight:{}{}",
                data.edges.get(id).cut_value,
                data.edges.get(id).slack(edge),
                data.edges.get(id).length,
                edge.weight,
                if special_edge == Some(id) { " *" } else { "" },
            )
            .into()
        },
        &|_, _| true,
        &|_, id| data.edges.get(id).in_tree,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{tests::init_log, to_dag::to_dag};

    #[test]
    fn rank_simple() {
        let dag = DirectedGraph::new(&['a', 'b'], &[('a', 'b')]);
        let ranks = rank(&dag);
        assert_ranks(&ranks, &[0, 1]);
    }

    #[test]
    fn rank_v1() {
        let dag = DirectedGraph::new(&['a', 'b', 'c', 'd'], &[('a', 'd'), ('b', 'c'), ('c', 'd')]);
        let ranks = rank(&dag);
        assert_ranks(&ranks, &[0, 0, 1, 2]);
    }

    #[test]
    fn rank_v2() {
        let dag = DirectedGraph::new(
            &['a', 'b', 'c', 'd', 'e', 'f'],
            &[('a', 'b'), ('b', 'c'), ('c', 'd'), ('e', 'f'), ('f', 'd')],
        );
        let ranks = rank(&dag);
        assert_ranks(&ranks, &[0, 1, 2, 3, 0, 1]);
    }

    #[test]
    fn rank_loop() {
        let mut dag = DirectedGraph::new(&[], &[(0, 1), (1, 2), (2, 0)]);
        to_dag(&mut dag);
        let ranks = rank(dbg!(&dag));
        assert_ranks(&ranks, &[0, 1, 2]);
    }

    fn assert_ranks(result: &NodeMap<i32>, expected: &[i32]) {
        assert_eq!(result.iter().map(|(_, &v)| v).collect::<Vec<_>>(), expected);
    }

    #[test]
    fn merge_trees() {
        init_log();
        let mut dag = DirectedGraph::new(&[0, 1, 2, 3], &[(0, 1), (1, 2), (2, 3)]);
        let mut ranks = dag.node_map();
        ranks.set(NodeId::from(0u32), 1);
        ranks.set(NodeId::from(1u32), 10);
        ranks.set(NodeId::from(2u32), 12);
        ranks.set(NodeId::from(3u32), 16);
        dag.edge_mut(EdgeId::from(1u32)).min_length = 2;
        dag.edge_mut(EdgeId::from(2u32)).min_length = 3;
        let data = span_tree(&mut dag, Postprocess::None, &mut ranks);
        normalize(data.ranks);
        let tree_edges_count: usize = dbg!(&data.nodes)
            .iter()
            .map(|(_, n)| n.inputs.len() + n.outputs.len())
            .sum();
        assert_eq!(tree_edges_count / 2, data.graph.nodes_count() as usize - 1);
        assert_ranks(data.ranks, &[0, 1, 3, 6]);
        assert_eq!(
            (1 + 2 + 3) as u32,
            data.edges.iter().map(|(_, d)| d.length).sum(),
        );
        assert_eq!(
            data.edges.iter().filter(|(_, e)| e.in_tree).count(),
            dag.nodes_count() as usize - 1
        );
    }

    #[test]
    fn span_tree_dimond() {
        /*
                0
              /   \
              1   2
              \   /
                3
        */
        let mut dag = DirectedGraph::new(&[0, 1, 2, 3], &[(0, 1), (0, 2), (1, 3), (2, 3)]);
        let mut ranks = dag.node_map();
        ranks.set(NodeId::from(0u32), 0);
        ranks.set(NodeId::from(1u32), 1);
        ranks.set(NodeId::from(2u32), 1);
        ranks.set(NodeId::from(3u32), 2);
        let data = span_tree(&mut dag, Postprocess::None, &mut ranks);
        let tree_edges_count: usize = dbg!(&data.nodes)
            .iter()
            .map(|(_, n)| n.inputs.len() + n.outputs.len())
            .sum();
        assert_eq!(tree_edges_count / 2, data.graph.nodes_count() as usize - 1);
        assert_eq!(
            data.edges.iter().filter(|(_, e)| e.in_tree).count(),
            dag.nodes_count() as usize - 1
        );
    }

    #[test]
    fn lr_two_parallel() {
        /*
           -- 2           1
          /    +         +  +
         d------\---+---b    \
                 c--+----------a

        */
        let mut dag = DirectedGraph::new(
            &['2', '1', 'a', 'b', 'c', 'd'],
            &[
                ('2', 'c'),
                ('2', 'd'),
                ('1', 'a'),
                ('1', 'b'),
                ('b', 'd'),
                ('a', 'c'),
            ],
        );
        let mut ranks = rank(&mut dag);
        dbg!(&ranks.iter().collect::<Vec<_>>());
        let mut data = span_tree(&mut dag, Postprocess::None, &mut ranks);
        let tree_edges_count: usize = dbg!(&data.nodes)
            .iter()
            .map(|(_, n)| n.inputs.len() + n.outputs.len())
            .sum();
        minmax(&mut data);
        dbg!(&data.nodes);
        cut_value(&mut data);
        dbg!(&data.edges);
        assert_eq!(tree_edges_count / 2, dag.nodes_count() as usize - 1);
    }

    /*
           a (1,5)
           /     \
        b(1,3)     e(4,4)moeng
         /  \
    c(1,1)   d(2,2)
    */
    #[test]
    fn minmax_test() {
        let mut dag = DirectedGraph::new(
            &['a', 'b', 'c', 'd', 'e'],
            &[('a', 'b'), ('b', 'c'), ('b', 'd'), ('a', 'e')],
        );
        let mut ranks = rank(&mut dag);
        let mut data = span_tree(&mut dag, Postprocess::None, &mut ranks);
        minmax(&mut data);
        assert_eq!(data.nodes.get(NodeId::from(0u32)).min, 1);
        assert_eq!(data.nodes.get(NodeId::from(1u32)).min, 1);
        assert_eq!(data.nodes.get(NodeId::from(2u32)).min, 1);
        assert_eq!(data.nodes.get(NodeId::from(2u32)).max, 1);
        assert_eq!(data.nodes.get(NodeId::from(3u32)).min, 2);
        assert_eq!(data.nodes.get(NodeId::from(3u32)).max, 2);
        assert_eq!(data.nodes.get(NodeId::from(1u32)).max, 3);
        assert_eq!(data.nodes.get(NodeId::from(4u32)).min, 4);
        assert_eq!(data.nodes.get(NodeId::from(4u32)).max, 4);
        assert_eq!(data.nodes.get(NodeId::from(0u32)).max, 5);
    }

    #[test]
    fn ns_steps() {
        let mut dag = DirectedGraph::new(
            &[0, 1, 2, 3, 4, 5, 6],
            &[
                (0, 3),
                (0, 6),
                (1, 3),
                (1, 5),
                (2, 3),
                (2, 4),
                (4, 5),
                (5, 6),
            ],
        );
        dag.edge_mut(EdgeId::from(6u32)).weight = 0;
        dag.edge_mut(EdgeId::from(7u32)).weight = 0;
        let mut ranks = rank(&mut dag);
        let mut data = span_tree(&mut dag, Postprocess::None, &mut ranks);
        minmax(&mut data);
        cut_value(&mut data);
        dbg!(&data);

        assert_eq!(data.edges.get(EdgeId::from(0u32)).cut_value, 2);
        assert_eq!(data.edges.get(EdgeId::from(1u32)).cut_value, 0);
        assert_eq!(data.edges.get(EdgeId::from(2u32)).cut_value, 2);
        assert_eq!(data.edges.get(EdgeId::from(3u32)).cut_value, 0);
        assert_eq!(data.edges.get(EdgeId::from(4u32)).cut_value, -1);
        assert_eq!(data.edges.get(EdgeId::from(5u32)).cut_value, 3);
        assert_eq!(data.edges.get(EdgeId::from(6u32)).cut_value, 2);
        assert_eq!(data.edges.get(EdgeId::from(7u32)).cut_value, 1);

        let negatove_edge = dbg!(NegativeEdgeSearch::new().next(&data)).unwrap();
        let replacement_edge = find_replacement_edge(&mut data, negatove_edge);
        dbg!(&negatove_edge, replacement_edge);
        replace_edge(&mut data, negatove_edge, replacement_edge);
        dbg!(&data, &data.ranks);

        assert_eq!(data.edges.get(EdgeId::from(0u32)).cut_value, 2);
        assert_eq!(data.edges.get(EdgeId::from(1u32)).cut_value, 0);
        assert_eq!(data.edges.get(EdgeId::from(2u32)).cut_value, 1);
        assert_eq!(data.edges.get(EdgeId::from(3u32)).cut_value, 1);
        assert_eq!(data.edges.get(EdgeId::from(4u32)).cut_value, 0);
        assert_eq!(data.edges.get(EdgeId::from(5u32)).cut_value, 2);
        assert_eq!(data.edges.get(EdgeId::from(6u32)).cut_value, 1);
        assert_eq!(data.edges.get(EdgeId::from(7u32)).cut_value, 1);

        let negatove_edge = dbg!(NegativeEdgeSearch::new().next(&data));

        assert!(negatove_edge.is_none())
    }

    // 1 to 3 converted for lr
    #[test]
    fn network_simplex_test() {
        let mut dag = DirectedGraph::new(
            &[0, 1, 2, 3, 4, 5, 6],
            &[
                (0, 3),
                (0, 6),
                (1, 3),
                (1, 5),
                (2, 3),
                (2, 4),
                (4, 5),
                (5, 6),
            ],
        );
        dag.edge_mut(EdgeId::from(6u32)).weight = 0;
        dag.edge_mut(EdgeId::from(7u32)).weight = 0;
        let ranks = network_simplex(&dag, Postprocess::None, None);
        assert_ranks(&ranks, &[1, 1, 0, 2, 1, 2, 3])
    }

    // a->b; a->c; c->d for LR
    #[test]
    fn network_simplex_tail() {
        init_log();
        let mut dag = DirectedGraph::new(
            &[0, 1, 2, 3, 5, 6],
            &[(0, 3), (0, 4), (1, 3), (1, 5), (4, 5), (2, 5), (2, 6)],
        );
        dag.edge_mut(EdgeId::from(4u32)).min_length = 100;
        dag.edge_mut(EdgeId::from(4u32)).weight = 0;
        let ranks = network_simplex(&dag, Postprocess::Center, None);
        assert_ranks(&ranks, &[0, 50, 100, 51, 101, 101, 1])
    }

    #[test]
    fn network_simplex_loop() {
        let mut dag = DirectedGraph::new(&[], &[(0, 1), (1, 2), (2, 0)]);
        to_dag(&mut dag);
        let ranks = network_simplex(&dag, Postprocess::None, None);
        assert_ranks(&ranks, &[0, 1, 2])
    }
}
