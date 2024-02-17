#[macro_use]
extern crate log;
use graph::{generator, read_dot};
extern crate simplelog;
use std::{env::args, fs::File, io::Read, str};

fn main() {
    let _ = simplelog::TermLogger::init(
        simplelog::LevelFilter::Debug,
        simplelog::Config::default(),
        simplelog::TerminalMode::Stderr,
        simplelog::ColorChoice::Auto,
    );
    if args().len() <= 1 || args().len() > 3 {
        error!("pass a dot file or number of nodes or edges to generate");
        return;
    }
    let data = if args().len() == 2 {
        let mut file = File::open(args().last().unwrap()).unwrap();
        let mut data = String::new();
        file.read_to_string(&mut data).unwrap();
        data
    } else {
        let mut args = args();
        args.next();
        generator::random(
            args.next().unwrap().parse().unwrap(),
            args.next().unwrap().parse().unwrap(),
        )
    };

    let dot = read_dot::parse(&data).expect("parse error");
    if dot.graph.nodes_count() == 0 {
        return;
    }
    let output = graph::full_draw(dot);
    let res = str::from_utf8(&output).expect("invalid utf");
    print!("{}", res);
}
