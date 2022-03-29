use crate::graph::*;
use crate::read_dot::DotGraph;

use std::io::Write;
use svg::node::element::{Ellipse, Marker, Path, Text, SVG};
use svg::node::Text as NodeText;
use svg::Document;

pub fn draw<'a, W: Write>(
    dot: &DotGraph<'a>,
    ranks: &NodeMap<i32>,
    places: &NodeMap<u32>,
    write: W,
) {
    let graph = &dot.graph;
    let x_step = 70. / 50.;
    let y_step = 70.;
    let rx = 20f64;
    let ry = 10f64;
    let mut max_x = 0;
    let mut max_y = 0;

    let mut document = Document::new();
    document = document.add(
        Marker::new()
            .set("id", "arrow")
            .set("viewBox", "0, 0, 7, 4")
            .set("refX", 5)
            .set("refY", 2)
            .set("markerUnits", "strokeWidth")
            .set("markerWidth", "7")
            .set("markerHeight", "4")
            .set("orient", "auto")
            .add(Path::new().set("d", "M 0 0 L 7 2 L 0 4 z")),
    );
    document = document.add(
        Marker::new()
            .set("id", "arrow-inverted")
            .set("viewBox", "0, 0, 7, 4")
            .set("refX", 2)
            .set("refY", 2)
            .set("markerUnits", "strokeWidth")
            .set("markerWidth", "7")
            .set("markerHeight", "4")
            .set("orient", "auto")
            .add(Path::new().set("d", "M 7 0 L 0 2 L 7 4 z")),
    );

    for (id, node) in graph.iter_nodes_with_id() {
        for edge in node.outputs.iter().map(|&id| graph.edge(id)) {
            let to_rank = *ranks.get(edge.to);
            let from_rank = *ranks.get(edge.from);
            let (from_rank, to_rank, to_id) = if from_rank < to_rank {
                (from_rank, to_rank, edge.to)
            } else {
                (to_rank, from_rank, edge.from)
            };

            let (marker_start, marker_end) = if edge.is_inverted() && !node.is_virtual {
                ("url(#arrow-inverted)", "")
            } else if !edge.is_inverted() && !graph.node(to_id).is_virtual {
                ("", "url(#arrow)")
            } else {
                ("", "")
            };

            let y_start = if node.is_virtual {
                from_rank as f64 * y_step
            } else {
                from_rank as f64 * y_step + ry + if marker_start.is_empty() { -0.2 } else { 1. }
            };

            let y_end = if graph.node(to_id).is_virtual {
                to_rank as f64 * y_step
            } else {
                to_rank as f64 * y_step - ry - if marker_end.is_empty() { -0.2 } else { 1. }
            };

            let t = Path::new()
                .set("stroke", "black")
                .set("marker-start", marker_start)
                .set("marker-end", marker_end)
                .set("fill", "none")
                .set(
                    "d",
                    format!(
                        "M{0},{1} C{0},{2},{3},{4} {3},{5}",
                        rx + *places.get(id) as f64 * x_step,
                        ry + y_start,
                        ry + y_start + ry + ry,
                        rx + *places.get(edge.to) as f64 * x_step,
                        y_end - ry,
                        ry + y_end,
                    ),
                );
            document = document.add(t);
        }

        if !node.is_virtual {
            let mut group = SVG::new()
                .set("x", *places.get(id) as f64 * x_step)
                .set("y", *ranks.get(id) as f64 * y_step)
                .set("width", rx * 2.)
                .set("height", ry * 2.);

            let svg_id = format!("svg_{}", graph.original_id(id).unwrap());

            group = group.add(
                Ellipse::new()
                    .set("cx", "50%")
                    .set("cy", "50%")
                    .set("rx", "48%")
                    .set("ry", "47%")
                    .set("fill", "silver")
                    .set("stroke", "black")
                    .set("stroke-width", 1)
                    .set("onClick", format!("outputClickHandler('{}')", &svg_id))
                    .set("id", svg_id.as_str()),
            );

            if let Some(name) = dot
                .labels
                .get(id)
                .or_else(|| graph.original_id(id).map(|s| *s))
            {
                group = group.add(
                    Text::new()
                        .add(NodeText::new(name))
                        .set("x", "50%")
                        .set("y", "50%")
                        .set("onClick", format!("outputClickHandler('{}')", &svg_id))
                        .set("dominant-baseline", "middle")
                        .set("text-anchor", "middle")
                        .set("font-size", 4),
                );
            }
            document = document.add(group);
        }
        max_x = max_x.max(*places.get(id));
        max_y = max_y.max(*ranks.get(id));
    }

    document = document.set(
        "viewBox",
        (
            0.,
            0,
            max_x as f64 * x_step as f64 + rx * 2.,
            max_y as f64 * y_step + rx,
        ),
    );

    let _todo = svg::write(write, &document);
}

#[cfg(test)]
mod tests {
    use crate::read_dot::parse;

    use super::*;

    #[test]
    fn simple() {
        let input = "digraph test { 0->2; 1->2; 0->3; 3->4; }";
        let mut dot = parse(input).unwrap();
        let mut ranks = dot.graph.node_map();
        ranks.set(NodeId::from(2u32), 1);
        ranks.set(NodeId::from(3u32), 1);
        dot.graph.node_mut(NodeId::from(3u32)).is_virtual = true;
        ranks.set(NodeId::from(4u32), 2);
        let mut p = dot.graph.node_map();
        p.set(NodeId::from(1u32), 1);
        p.set(NodeId::from(3u32), 1);
        let mut s = vec![];
        draw(&dot, &ranks, &p, &mut s);
        // assert_eq!(std::str::from_utf8(&s[..]).unwrap(), ""); TODO
    }
}
