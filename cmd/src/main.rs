#[macro_use]
extern crate log;
use graph::{read_dot, to_dag};
extern crate simplelog;
use std::{env::args, fs::File, io::Read, str};

fn main() {
    let _ = simplelog::TermLogger::init(
        simplelog::LevelFilter::Debug,
        simplelog::Config::default(),
        simplelog::TerminalMode::Stderr,
        simplelog::ColorChoice::Auto,
    );
    if args().len() <= 1 {
        error!("pass a dot file");
        return;
    }
    let mut file = File::open(args().last().unwrap()).unwrap();
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();

    let mut dot = read_dot::parse(&data).expect("parse error");
    to_dag::to_dag(&mut dot.graph);
    let mut ranks = graph::rank_with_components(&dot.graph);
    graph::add_virtual_nodes::add_virtual_nodes(&mut dot.graph, &mut ranks);
    let places = graph::place::places3(&dot.graph, &ranks);
    let coords = graph::xcoord::x_coordinates(&dot.graph, &ranks, &places);
    let mut output = vec![];
    graph::draw::draw(&dot, &ranks, &coords, &mut output);

    let res = str::from_utf8(&output).expect("invalid utf");
    print!("{}", res);
}
