use std::collections::HashMap;

use crate::graph::*;
use pest::{iterators::Pair, Parser};

#[derive(pest_derive::Parser)]
#[grammar = "dot.pest"]
struct DotParser;

pub struct DotGraph<'a> {
    pub graph: DirectedGraph<&'a str>,
    pub labels: NodeMap<Option<&'a str>>,
}

pub fn parse<'a>(data: &'a str) -> Result<DotGraph<'a>, String> {
    let graph = DotParser::parse(Rule::graph, &data)
        .map_err(|e| e.to_string())?
        .next()
        .unwrap();

    Ok(convert_graph(graph))
}

fn convert_graph<'a>(graph: Pair<'a, Rule>) -> DotGraph<'a> {
    // println!("Rule:    {:?}", graph.as_rule());
    //println!("Text:    {}\n", graph.as_str());
    let mut ids = vec![];
    let mut labels = HashMap::new();
    let mut links = vec![];

    for statement in graph.into_inner().skip(1).next().unwrap().into_inner() {
        match statement.as_rule() {
            Rule::link => links.push(link(statement)),
            Rule::node => {
                let (id, label) = node(statement);
                ids.push(id);
                labels.insert(id, label);
            }
            _ => unreachable!(),
        }
    }
    let g = DirectedGraph::new(&ids, &links);
    let mut map = g.node_map();
    for (id, _) in g.iter_nodes_with_id() {
        if let Some(&label) = g.original_id(id).and_then(|&id| labels.get(id)) {
            map.set(id, Some(label))
        }
    }
    DotGraph {
        graph: g,
        labels: map,
    }
}

fn link<'a>(link: Pair<'a, Rule>) -> (&'a str, &'a str) {
    let mut items = link.into_inner();
    let from = items.next().unwrap().as_str();
    let to = items.next().unwrap().as_str();
    (from, to)
}

fn node<'a>(node: Pair<'a, Rule>) -> (&'a str, &'a str) {
    let mut items = node.into_inner();
    let name = items.next().unwrap().as_str();
    let label = items.next().and_then(label).unwrap_or(name);
    (name, label)
}

fn label<'a>(attributes: Pair<'a, Rule>) -> Option<&'a str> {
    attributes.into_inner().skip(1).next().map(|s| s.as_str())
}
