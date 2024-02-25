use rand::Rng;

pub fn random(nodes_count: u32, edges_count: u32) -> String {
    let mut graph = "digraph x {".to_string();
    for i in 0..nodes_count {
        graph += &format!("N{};", i);
    }
    if nodes_count > 0 {
        let mut rng = rand::thread_rng();
        let mut unconnected = vec![true; nodes_count as usize];
        let mut unconnected_count = nodes_count;
        for _ in 0..edges_count {
            let (from, to) = if unconnected_count == 0 {
                (rng.gen_range(0..nodes_count), rng.gen_range(0..nodes_count))
            } else {
                let start = rng.gen_range(0..nodes_count) as usize;
                let from = (start..(nodes_count as usize))
                    .chain(0..start)
                    .find(|&v| unconnected[v])
                    .unwrap();
                unconnected[from] = false;
                unconnected_count -= 1;
                let to = loop {
                    let v = rng.gen_range(0..nodes_count) as usize;
                    if v != from || nodes_count <= 1 {
                        break v;
                    }
                };

                if unconnected[to] {
                    unconnected[to] = false;
                    unconnected_count -= 1;
                }
                if rng.gen_bool(0.5) {
                    (from as u32, to as u32)
                } else {
                    (to as u32, from as u32)
                }
            };
            graph += &format!("N{} -> N{};", from, to);
        }
    }
    graph += "}";
    graph
}
