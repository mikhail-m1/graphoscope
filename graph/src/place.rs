use rand::{prelude::ThreadRng, Rng};
use std::fmt::Debug;

use crate::graph::*;

pub fn places<T: Debug>(graph: &DirectedGraph<T>, ranks: &NodeMap<i32>) -> NodeMap<u32> {
    let mut places = graph.node_map();
    let mut next = vec![];

    for (id, _) in graph.iter_nodes_with_id() {
        let rank = *ranks.get(id) as usize;
        if next.len() <= rank {
            next.resize(rank + 1, 0);
        }
        places.set(id, next[rank]);
        next[rank] += 1;
    }
    places
}

pub fn places3<T: Debug>(graph: &DirectedGraph<T>, ranks: &NodeMap<i32>) -> NodeMap<u32> {
    let mut layers = vec![];

    for (id, _) in graph.iter_nodes_with_id() {
        let rank = *ranks.get(id) as usize;
        if layers.len() <= rank {
            layers.resize(rank + 1, vec![]);
        }
        layers[rank].push(id);
    }

    let mut places = places(graph, ranks);
    let mut crosses = total_number_of_crosses(graph, &places, &layers);

    for i in 0..24 {
        let mut candidate = places.clone();
        wmedian(graph, &layers, &mut candidate, i % 2 == 0);
        transpose(graph, &layers, &mut candidate);
        let candidate_crosses = total_number_of_crosses(graph, &candidate, &layers);
        if candidate_crosses <= crosses {
            std::mem::swap(&mut places, &mut candidate);
            crosses = candidate_crosses;
        }
        //dbg!(&places);
    }

    let min = places.iter().map(|(_, &v)| v).min().unwrap();
    for (_, v) in places.iter_mut() {
        *v -= min;
    }

    places
}

fn wmedian<T: Debug>(
    graph: &DirectedGraph<T>,
    layers: &[Vec<NodeId>],
    candidate: &mut NodeMap<u32>,
    top_down: bool,
) {
    if top_down {
        for i in 1..layers.len() {
            let mut medians = median(graph, candidate, &layers[i], true);
            medians.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
            let mut next_free = 0;
            for &(node_id, m) in medians.iter() {
                let next = next_free.max(m.max(0.) as u32);
                next_free = next + 1;
                candidate.set(node_id, next);
            }
        }
    } else {
        for i in (0..layers.len() - 1).rev() {
            let mut medians = median(graph, candidate, &layers[i], false);
            medians.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
            // dbg!(&medians);
            let mut next_free = 0;
            for &(node_id, m) in medians.iter() {
                let next = next_free.max(m.max(0.) as u32);
                next_free = next + 1;
                candidate.set(node_id, next);
            }
        }
    }
    //dbg!(top_down, &candidate);
}

fn median<T: Debug>(
    graph: &DirectedGraph<T>,
    places: &NodeMap<u32>,
    layer: &[NodeId],
    from_top: bool,
) -> Vec<(NodeId, f64)> {
    let mut medians = vec![];
    for &id in layer {
        let mut linked_places: Vec<u32> = if from_top {
            graph.iter_parents(id).map(|p| *places.get(p)).collect()
        } else {
            graph.iter_children(id).map(|c| *places.get(c)).collect()
        };
        linked_places.sort();

        let med = if linked_places.len() == 0 {
            -1.
        } else if linked_places.len() % 2 == 1 {
            linked_places[linked_places.len() / 2] as f64
        } else {
            //TODO try to use formula from the DOC
            (linked_places[linked_places.len() / 2 - 1] as f64
                + linked_places[linked_places.len() / 2] as f64)
                / 2.
        };
        medians.push((id, med));
    }
    medians
}

fn transpose<T: Debug>(
    graph: &DirectedGraph<T>,
    layers: &[Vec<NodeId>],
    candidate: &mut NodeMap<u32>,
) {
    let mut crosses = total_number_of_crosses(graph, candidate, layers);
    loop {
        let mut improved = false;
        for layer in layers {
            for i in 0..layer.len().saturating_sub(2) {
                let node1_place = *candidate.get(layer[i]);
                let node2_place = *candidate.get(layer[i + 1]);
                candidate.set(layer[i], node2_place);
                candidate.set(layer[i + 1], node1_place);
                let new_crosses = total_number_of_crosses(graph, candidate, layers);
                if new_crosses < crosses {
                    crosses = new_crosses;
                    improved = true;
                } else {
                    candidate.set(layer[i], node1_place);
                    candidate.set(layer[i + 1], node2_place);
                }
            }
        }
        if !improved {
            break;
        }
    }
}

pub fn places2<T: Debug>(graph: &DirectedGraph<T>, ranks: &NodeMap<u32>) -> NodeMap<u32> {
    let mut layers = vec![];

    for (id, _) in graph.iter_nodes_with_id() {
        let rank = *ranks.get(id) as usize;
        if layers.len() <= rank {
            layers.resize(rank + 1, vec![]);
        }
        layers[rank].push(id);
    }

    if layers.len() <= 1 {
        let mut map = graph.node_map();
        for (n, (id, _)) in graph.iter_nodes_with_id().enumerate() {
            map.set(id, n as u32)
        }
        return map;
    }

    let biggest_layer = layers
        .iter()
        .enumerate()
        .map(|(n, nodes)| (n, nodes.len()))
        .max_by_key(|(_, s)| *s)
        .unwrap()
        .0;

    let other_layer = if biggest_layer == 0 && layers.len() > 1 {
        2
    } else if biggest_layer == layers.len() - 1 {
        biggest_layer - 1
    } else if layers[biggest_layer - 1].len() >= layers[biggest_layer + 1].len() {
        biggest_layer - 1
    } else {
        biggest_layer + 1
    };

    let mut places = graph.node_map();

    place_layers(
        graph,
        &layers[biggest_layer.min(other_layer)],
        &layers[biggest_layer.max(other_layer)],
        &mut places,
    );

    for i in (0..biggest_layer.min(other_layer)).rev() {
        dbg!("place", i);
        place_layer(graph, &layers[i], &layers[i + 1], &mut places, false);
    }

    for i in biggest_layer.max(other_layer) + 1..layers.len() {
        place_layer(graph, &layers[i - 1], &layers[i], &mut places, true);
    }

    places
}

struct PlaceContext<'a, T: Debug> {
    graph: &'a DirectedGraph<T>,
    top_layer: &'a [NodeId],
    bottom_layer: &'a [NodeId],
    places: &'a mut NodeMap<u32>,
    used_top_places: Vec<bool>,
    used_bottom_places: Vec<bool>,
    candidates: NodeMap<bool>,
    processed: NodeMap<bool>,
    processed_count: u32,
    candidates_vec: Vec<(NodeId, bool)>,
    next: NodeId,
    is_upper: bool,
    place: usize,
    rng: ThreadRng,
}

impl<'a, T: Debug> PlaceContext<'a, T> {
    fn new(
        graph: &'a DirectedGraph<T>,
        top_layer: &'a [NodeId],
        bottom_layer: &'a [NodeId],
        places: &'a mut NodeMap<u32>,
    ) -> Self {
        let places_count = top_layer.len().max(bottom_layer.len());
        Self {
            graph,
            top_layer,
            bottom_layer,
            places,
            used_top_places: vec![false; places_count],
            used_bottom_places: vec![false; places_count],
            candidates: graph.node_map::<bool>(),
            processed: graph.node_map::<bool>(),
            processed_count: 0,
            candidates_vec: vec![],
            next: top_layer[0],
            is_upper: false,
            place: 0,
            rng: rand::thread_rng(),
        }
    }

    fn has_unprocessed(&self) -> bool {
        (self.processed_count as usize) < self.top_layer.len() + self.bottom_layer.len()
    }

    fn top_id_and_count(&self) -> Option<(NodeId, usize)> {
        self.top_layer
            .iter()
            .filter(|&&id| !self.processed.get(id))
            .map(|&id| (id, self.graph.node(id).outputs.len()))
            .max_by_key(|(_, count)| *count)
    }

    fn bottom_id_and_count(&self) -> Option<(NodeId, usize)> {
        self.bottom_layer
            .iter()
            .filter(|&&id| !self.processed.get(id))
            .map(|&id| (id, self.graph.node(id).inputs.len()))
            .max_by_key(|(_, count)| *count)
    }

    fn add_next_to_processed(&mut self) {
        self.processed.set(self.next, true);
        self.processed_count += 1;
        self.places.set(self.next, self.place as u32);
        if self.is_upper {
            self.used_top_places[self.place] = true;
        } else {
            self.used_bottom_places[self.place] = true;
        }
    }

    fn add_candidates(&mut self) {
        if self.is_upper {
            for id in self.graph.iter_children(self.next) {
                if !self.candidates.get(id) && !self.processed.get(id) {
                    self.candidates.set(id, true);
                    self.candidates_vec.push((id, false));
                }
            }
        } else {
            for id in self.graph.iter_parents(self.next) {
                if !self.candidates.get(id) && !self.processed.get(id) {
                    self.candidates.set(id, true);
                    self.candidates_vec.push((id, true));
                }
            }
        };
    }

    fn candidates(&self) -> Vec<(NodeId, (u32, u32))> {
        self.candidates_vec
            .iter()
            .map(|&(id, is_upper)| {
                if is_upper {
                    (
                        id,
                        self.graph
                            .iter_children(id)
                            .filter(|&to_id| *self.processed.get(to_id))
                            .fold((0, 0), |(s, c1), to_id| {
                                (s + self.places.get(to_id), c1 + 1)
                            }),
                    )
                } else {
                    (
                        id,
                        self.graph
                            .iter_parents(id)
                            .filter(|&to_id| *self.processed.get(to_id))
                            .fold((0, 0), |(s, c1), to_id| {
                                (s + self.places.get(to_id), c1 + 1)
                            }),
                    )
                }
            })
            .collect()
    }

    fn select_next(&mut self, candidates: &mut Vec<(NodeId, (u32, u32))>) -> PossiblePlaces {
        candidates.sort_by_key(|(_, (_, c))| *c);
        let max = (candidates.last().unwrap().1).1 as f64;
        let from = max * 2.0 / 3.0;

        let from_pos = candidates
            .iter()
            .position(|&(_, (_, c))| c as f64 >= from)
            .unwrap();

        let (next, (s, c)) = candidates[self.rng.gen_range(from_pos..candidates.len())];
        self.next = next;
        PossiblePlaces::new(s, c)
    }

    fn move_candidate_to_next(&mut self) {
        let next_position = self
            .candidates_vec
            .iter()
            .position(|(id, _)| *id == self.next)
            .unwrap();

        self.is_upper = self.candidates_vec[next_position].1;
        self.candidates_vec.remove(next_position);
    }

    fn calculate_place(&mut self, places_iter: PossiblePlaces) {
        //TODO prevent infinity loop in case of error
        self.place = places_iter
            .filter(|&place| {
                if self.is_upper {
                    place < self.used_top_places.len() && !self.used_top_places[place]
                } else {
                    place < self.used_bottom_places.len() && !self.used_bottom_places[place]
                }
            })
            .next()
            .unwrap();
    }

    fn add_processed_layer(&mut self, is_upper: bool) {
        let layer = if is_upper {
            self.top_layer
        } else {
            self.bottom_layer
        };
        self.is_upper = is_upper;
        for &id in layer {
            self.processed.set(id, true);
            self.processed_count += 1;
        }
    }

    fn add_unprocessed_layer(&mut self, is_bottom: bool) {
        let layer = if is_bottom {
            self.bottom_layer
        } else {
            self.top_layer
        };
        for &id in layer {
            self.candidates_vec.push((id, !is_bottom))
        }
    }
}

fn place_layers<T: Debug>(
    graph: &DirectedGraph<T>,
    top_layer: &[NodeId],
    bottom_layer: &[NodeId],
    places: &mut NodeMap<u32>,
) {
    let mut min_number_of_crosses = u32::MAX;
    for _ in 0..3 {
        let mut temp_places = graph.node_map();
        let mut c = PlaceContext::new(graph, top_layer, bottom_layer, &mut temp_places);

        while c.has_unprocessed() {
            let top_top_node_id = c.top_id_and_count();
            let top_bottom_node_id = c.bottom_id_and_count();

            if top_bottom_node_id.is_none()
                || (top_top_node_id.is_some()
                    && top_top_node_id.unwrap().1 >= top_bottom_node_id.unwrap().1)
            {
                c.next = top_top_node_id.unwrap().0;
                c.is_upper = true;
            } else {
                c.next = top_bottom_node_id.unwrap().0;
                c.is_upper = false;
            };
            c.calculate_place(PossiblePlaces::new(0, 1));

            loop {
                c.add_next_to_processed();
                c.add_candidates();
                if c.candidates_vec.is_empty() {
                    break;
                }
                let mut candidates_links_count = c.candidates();
                let possible_places = c.select_next(&mut candidates_links_count);
                //dbg!((&candidates_links_count, c.next));
                c.move_candidate_to_next();
                c.calculate_place(possible_places);
            }
        }
        let new_number_of_crosses =
            number_of_crosses(c.graph, c.top_layer, c.bottom_layer, c.places);
        if new_number_of_crosses < min_number_of_crosses {
            min_number_of_crosses = new_number_of_crosses;

            for &id in c.top_layer {
                places.set(id, *c.places.get(id));
            }
            for &id in c.bottom_layer {
                places.set(id, *c.places.get(id));
            }
        }
        if min_number_of_crosses == 0 {
            break;
        }
    }
}

fn place_layer<T: Debug>(
    graph: &DirectedGraph<T>,
    top_layer: &[NodeId],
    bottom_layer: &[NodeId],
    places: &mut NodeMap<u32>,
    is_upper_processed: bool,
) {
    let mut c = PlaceContext::new(graph, top_layer, bottom_layer, places);
    c.add_processed_layer(is_upper_processed);
    c.add_unprocessed_layer(is_upper_processed);
    let mut candidates_links_count = c.candidates();
    c.is_upper = !is_upper_processed;

    while !candidates_links_count.is_empty() {
        let possible_places = c.select_next(&mut candidates_links_count);
        //dbg!((&candidates_links_count, c.next));
        c.calculate_place(possible_places);
        c.add_next_to_processed();
        // TODO optimize
        let pos = candidates_links_count
            .iter()
            .position(|i| i.0 == c.next)
            .unwrap();
        candidates_links_count.remove(pos);
    }
}

fn number_of_crosses<T: Debug>(
    graph: &DirectedGraph<T>,
    top_layer: &[NodeId],
    bottom_layer: &[NodeId],
    places: &NodeMap<u32>,
) -> u32 {
    let mut is_in_bottom = graph.node_map::<bool>();
    let mut crosses = 0;

    for &id in bottom_layer {
        is_in_bottom.set(id, true);
    }

    let mut top_ordered_id_and_place: Vec<_> =
        top_layer.iter().map(|&id| (id, places.get(id))).collect();
    top_ordered_id_and_place.sort_by_key(|(_, place)| *place);

    let max_bottom_place = bottom_layer
        .iter()
        .map(|&id| *places.get(id))
        .max()
        .unwrap();
    let mut bottom_links_count = vec![0; max_bottom_place as usize + 1];

    for (from_id, _) in top_ordered_id_and_place {
        let mut to_places: Vec<_> = graph
            .iter_children(from_id)
            .filter(|&to_id| *is_in_bottom.get(to_id))
            .map(|id| *places.get(id) as usize)
            .collect();
        to_places.sort();

        for place in to_places {
            bottom_links_count[place] += 1;
            crosses += &bottom_links_count[place + 1..].iter().sum();
        }
    }
    crosses
}

fn total_number_of_crosses<T: Debug>(
    graph: &DirectedGraph<T>,
    places: &NodeMap<u32>,
    layers: &[Vec<NodeId>],
) -> u32 {
    (1..layers.len())
        .map(|bottom| number_of_crosses(graph, &layers[bottom - 1], &layers[bottom], places))
        .sum()
}

#[derive(Debug)]
struct PossiblePlaces {
    start: u32,
    current: u32,
}

impl PossiblePlaces {
    fn new(sum: u32, count: u32) -> Self {
        Self {
            start: sum / count.max(1),
            current: u32::max_value(),
        }
    }
}

impl Iterator for PossiblePlaces {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        self.current = if self.current == u32::max_value() {
            self.start
        } else if self.current <= self.start {
            self.start - self.current + self.start + 1
        } else if self.current - self.start <= self.start {
            self.start + self.start - self.current
        } else {
            self.current + 1
        };
        Some(self.current as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple() {
        let dag = DirectedGraph::new(&['a', 'b', 'c'], &[]);
        let mut ranks = dag.node_map();
        ranks.set(NodeId::from(2u32), 1);
        let p = places(&dag, &ranks);
        assert_eq!(*p.get(NodeId::from(0u32)), 0);
        assert_eq!(*p.get(NodeId::from(1u32)), 1);
        assert_eq!(*p.get(NodeId::from(2u32)), 0);
    }

    #[test]
    fn no_crosses() {
        let dag = DirectedGraph::new(
            &[0, 1, 2, 3, 4, 5],
            &[(0, 3), (0, 4), (1, 4), (1, 5), (2, 5)],
        );
        let mut places = dag.node_map();
        places.set(NodeId::from(1u32), 1);
        places.set(NodeId::from(2u32), 2);
        places.set(NodeId::from(4u32), 1);
        places.set(NodeId::from(5u32), 2);
        assert_eq!(
            number_of_crosses(
                &dag,
                &[NodeId::from(0u32), NodeId::from(1u32), NodeId::from(2u32)],
                &[NodeId::from(3u32), NodeId::from(4u32), NodeId::from(5u32)],
                &places
            ),
            0
        )
    }

    #[test]
    fn simple_cross() {
        let dag = DirectedGraph::new(&[0, 1, 2, 3], &[(0, 3), (1, 2)]);
        let mut places = dag.node_map();
        places.set(NodeId::from(1u32), 1);
        places.set(NodeId::from(3u32), 1);
        assert_eq!(
            number_of_crosses(
                &dag,
                &[NodeId::from(0u32), NodeId::from(1u32)],
                &[NodeId::from(2u32), NodeId::from(3u32)],
                &places
            ),
            1
        )
    }

    #[test]
    fn more_crosses() {
        let dag = DirectedGraph::new(
            &[0, 1, 2, 3, 4, 5],
            &[(0, 3), (0, 4), (1, 4), (1, 5), (2, 5), (0, 5)],
        );
        let mut places = dag.node_map();
        places.set(NodeId::from(1u32), 1);
        places.set(NodeId::from(2u32), 2);
        places.set(NodeId::from(3u32), 2);
        places.set(NodeId::from(5u32), 1);
        let top_layer = &[NodeId::from(0u32), NodeId::from(1u32), NodeId::from(2u32)];
        let bottom_layer = &[NodeId::from(3u32), NodeId::from(4u32), NodeId::from(5u32)];
        assert_eq!(number_of_crosses(&dag, top_layer, bottom_layer, &places), 4);

        places.set(NodeId::from(1u32), 2);
        places.set(NodeId::from(2u32), 1);
        assert_eq!(number_of_crosses(&dag, top_layer, bottom_layer, &places), 5);
    }

    #[test]
    fn two_layers() {
        let dag = DirectedGraph::new(
            &[0, 1, 2, 3, 4, 5],
            &[(0, 3), (0, 4), (1, 4), (1, 5), (2, 5), (0, 5)],
        );
        let top_layer = &[NodeId::from(0u32), NodeId::from(1u32), NodeId::from(2u32)];
        let bottom_layer = &[NodeId::from(3u32), NodeId::from(4u32), NodeId::from(5u32)];

        let mut places = dag.node_map();
        place_layers(&dag, top_layer, bottom_layer, &mut places);
        assert_eq!(places.iter().map(|(_, v)| *v).sum::<u32>(), 6); //FIXME
        dbg!(&places);
        dbg!(number_of_crosses(&dag, top_layer, bottom_layer, &places));
    }

    #[test]
    fn two_layers_two_clusters() {
        let dag = DirectedGraph::new(&[0, 1, 2, 3, 4, 5], &[(0, 3), (0, 4), (1, 4), (2, 5)]);
        let top_layer = &[NodeId::from(0u32), NodeId::from(1u32), NodeId::from(2u32)];
        let bottom_layer = &[NodeId::from(3u32), NodeId::from(4u32), NodeId::from(5u32)];

        let mut places = dag.node_map();
        place_layers(&dag, top_layer, bottom_layer, &mut places);
        dbg!(&places);
        assert_eq!(places.iter().map(|(_, v)| *v).sum::<u32>(), 6); //FIXME
        assert!(number_of_crosses(&dag, top_layer, bottom_layer, &places) <= 1);
    }

    #[test]
    fn place_layer_1() {
        let dag = DirectedGraph::new(&[0, 1, 2, 3, 4, 5], &[(0, 5), (1, 4), (1, 3), (2, 4)]);
        let top_layer = &[NodeId::from(0u32), NodeId::from(1u32), NodeId::from(2u32)];
        let bottom_layer = &[NodeId::from(3u32), NodeId::from(4u32), NodeId::from(5u32)];
        let mut places = dag.node_map();
        places.set(NodeId::from(1u32), 1);
        places.set(NodeId::from(2u32), 2);
        place_layer(&dag, top_layer, bottom_layer, &mut places, true);
        dbg!(&places);
    }

    #[test]
    fn iterator() {
        let mut i = PossiblePlaces::new(2, 1);
        assert_eq!(i.next(), Some(2));
        assert_eq!(i.next(), Some(3));
        assert_eq!(i.next(), Some(1));
        assert_eq!(i.next(), Some(4));
        assert_eq!(i.next(), Some(0));
        assert_eq!(i.next(), Some(5));
        assert_eq!(i.next(), Some(6));
        assert_eq!(i.next(), Some(7));
    }
}
