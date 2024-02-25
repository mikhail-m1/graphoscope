use graph::{generator, read_dot};
extern crate simplelog;
use clap::{Parser, Subcommand};
use std::{fs::File, io::Read, path::PathBuf, str};

fn main() {
    let args = Cli::parse();

    let _ = simplelog::TermLogger::init(
        args.log_level,
        simplelog::Config::default(),
        simplelog::TerminalMode::Stderr,
        simplelog::ColorChoice::Auto,
    );

    let data = match args.command {
        Commands::Dot { path } => {
            let mut file = File::open(path).expect("Cannnot open the dot file");
            let mut data = String::new();
            file.read_to_string(&mut data).unwrap();
            data
        }
        Commands::Generate {
            nodes_count,
            edges_count,
        } => generator::random(nodes_count, edges_count),
    };

    let dot = read_dot::parse(&data).expect("parse error");
    if dot.graph.nodes_count() == 0 {
        return;
    }
    let dot = if dot.graph.nodes_count() > 3 {
        graph::subgraph(&dot, None, args.max_nodes, args.max_edges)
    } else {
        dot
    };
    let output = graph::full_draw(dot);
    let res = str::from_utf8(&output).expect("invalid utf");
    print!("{}", res);
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// max nodes
    #[arg(short = 'n', long, default_value_t = 200)]
    max_nodes: u32,
    /// Optional name to operate on

    /// max esges
    #[arg(short = 'e', long, default_value_t = 200)]
    max_edges: u32,

    /// log level [trace, debug, info, warn, error]
    #[arg(short = 'l', long, default_value_t = simplelog::LevelFilter::Debug)]
    log_level: simplelog::LevelFilter,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// read dot file
    Dot {
        /// path to dot file
        path: PathBuf,
    },
    /// generate graph
    Generate {
        /// nodes count
        nodes_count: u32,
        /// edges count
        edges_count: u32,
    },
}
