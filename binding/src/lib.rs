use graph::{self, generator, subgraph};
use log::info;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub struct Graph {
    _input: String,
    //hack, reference to input
    graph: Result<graph::read_dot::DotGraph<'static>, String>,
}

#[wasm_bindgen]
impl Graph {
    pub fn new(dot: &JsValue) -> Self {
        let input: String = dot.as_string().unwrap();

        // same struc in rust cannot hold data and referene on it
        // in the futer we can copy all strings to internal graph structure
        // and there is no need in the original graph
        let input_static: &'static str = unsafe { std::mem::transmute(input.as_str()) };
        let graph = graph::read_dot::parse(input_static).map(|mut g| {
            graph::to_dag::to_dag(&mut g.graph);
            g
        });
        match &graph {
            Ok(g) => info!(
                "New Graph {} nodes and {} edges",
                g.graph.nodes_count(),
                g.graph.edges_count()
            ),
            Err(e) => log::error!("Parse failed: {e}"),
        }
        Self {
            _input: input,
            graph,
        }
    }

    pub fn node_count(&self) -> JsValue {
        match &self.graph {
            Ok(g) => g.graph.nodes_count().into(),
            Err(e) => e.into(),
        }
    }

    pub fn is_error(&self) -> JsValue {
        self.graph.is_err().into()
    }

    pub fn render(&self, around_node_id: &str, max_nodes: u32, max_edges: u32) -> JsValue {
        match &self.graph {
            Err(e) => (String::from("<pre>") + &e + "</pre>").into(),
            Ok(dot) => {
                if dot.graph.nodes_count() == 0 || max_nodes == 0 {
                    return r#"<svg viewBox="0 0 1 1" xmlns="http://www.w3.org/2000/svg"></svg>"#
                        .into();
                }
                let start_node_id = (around_node_id.len() != 0).then(|| {
                    let name = &around_node_id[4..];
                    dot.graph
                        .iter_nodes_with_id()
                        .filter_map(|(id, _)| {
                            (dot.graph.original_id(id) == Some(&name)).then_some(id)
                        })
                        .next()
                        .unwrap()
                });
                let (dot, extra_edges) =
                    if dot.graph.nodes_count() > max_nodes || dot.graph.edges_count() > max_edges {
                        let (subgraph, extra_edges) =
                            subgraph(dot, start_node_id, max_nodes, max_edges);
                        (subgraph, Some(extra_edges))
                    } else {
                        ((*dot).clone(), None)
                    };
                std::str::from_utf8(&graph::full_draw(dot, extra_edges.as_ref()))
                    .unwrap()
                    .into()
            }
        }
    }

    pub fn find_nodes(&self, value: &str) -> JsValue {
        (if let Ok(dot) = &self.graph {
            dot.graph
                .iter_nodes_ids()
                .flat_map(|id| dot.graph.original_id(id).into_iter())
                .filter(|oid| oid.to_lowercase().contains(&value.to_lowercase()))
                .map(|&id| JsValue::from(id))
                .collect::<js_sys::Array>()
        } else {
            js_sys::Array::new()
        })
        .into()
    }
}

#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();
    wasm_logger::init(wasm_logger::Config::new(log::Level::Debug));
    //alert("OK");
    //console::log_1(&JsValue::from_str("Hello world!"));
    Ok(())
}

#[wasm_bindgen]
pub fn parse(dot: &JsValue) -> JsValue {
    Graph::new(dot).into()
}

#[wasm_bindgen]
pub fn render_random(nodes_count: u32, edges_count: u32) -> JsValue {
    generator::random(nodes_count, edges_count).into()
}
