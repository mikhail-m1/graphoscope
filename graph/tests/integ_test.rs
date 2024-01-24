use graph::{read_dot, to_dag};
use std::{fs::File, io::Read, str};
extern crate difference;

#[test]
fn itest_aim() -> Result<(), std::io::Error> {
    check("../dot_files/aim.dot", "tests/results/aim.dot")
}

#[test]
fn itest_test() -> Result<(), std::io::Error> {
    check("../dot_files/test.dot", "tests/results/test.dot")
}

#[test]
fn itest_aim_simp2() -> Result<(), std::io::Error> {
    check("../dot_files/aim_simp2.dot", "tests/results/aim_simp2.dot")
}

fn check(input_name: &str, output_name: &str) -> Result<(), std::io::Error> {
    let mut data = String::new();
    File::open(input_name)
        .expect("input file")
        .read_to_string(&mut data)?;

    let mut dot = read_dot::parse(&data).expect("parse error");
    to_dag::to_dag(&mut dot.graph);
    let mut ranks = graph::rank_with_components(&dot.graph);
    graph::add_virtual_nodes::add_virtual_nodes(&mut dot.graph, &mut ranks);
    let places = graph::place::places3(&dot.graph, &ranks);
    let coords = graph::xcoord::x_coordinates(&dot.graph, &ranks, &places);
    println!("OK!!!");

    if std::env::var("GS_UPDATE_TEST_RESULTS").is_ok() {
        dot.graph.dot_result(
            File::create(output_name).expect("output file"),
            &ranks,
            &coords,
        )
    }
    let mut output = vec![];
    dot.graph.dot_result(&mut output, &ranks, &coords);

    let mut expected = Vec::new();
    File::open(output_name)
        .expect("output file")
        .read_to_end(&mut expected)?;

    let expected = str::from_utf8(&expected).unwrap();
    let output = str::from_utf8(&output).unwrap();

    //TODO find how to output diff by line
    let diff = difference::Changeset::new(expected, output, "\n");
    for diff in &diff.diffs {
        match diff {
            difference::Difference::Add(s) => println!("+{}", s),
            difference::Difference::Rem(s) => println!("-{}", s),
            _ => {}
        }
    }
    assert!(expected == output);
    Ok(())
}

//#[test]
fn compare_graphs() {
    let mut data = String::new();
    File::open("../dot_files/aim_min_ambiguty.dot")
        .expect("input file")
        .read_to_string(&mut data)
        .unwrap();

    let mut dot = read_dot::parse(&data).expect("parse error");
    to_dag::to_dag(&mut dot.graph);
    let mut ranks = graph::rank_with_components(&dot.graph);
    graph::add_virtual_nodes::add_virtual_nodes(&mut dot.graph, &mut ranks);
    let _places = graph::place::places3(&dot.graph, &ranks);
    let old = &dot.graph; // Change this to compare different graphs
    let new = &dot.graph;

    assert_eq!(old.nodes_count(), new.nodes_count());
    assert_eq!(old.edges_count(), new.edges_count());

    for (id, node) in old.iter_nodes_with_id() {
        assert_eq!(
            node.inputs
                .iter()
                .map(|&e| (e, old.edge(e).to))
                .collect::<Vec<_>>(),
            new.node(id)
                .inputs
                .iter()
                .map(|&e| (e, old.edge(e).to))
                .collect::<Vec<_>>()
        );

        assert_eq!(node.inputs, new.node(id).inputs, "for node {id:?} inputs");
        assert_eq!(
            node.outputs,
            new.node(id).outputs,
            "for node {id:?} outputs"
        );
    }
}
