use graph::{self, generator, subgraph};
use log::info;
use ouroboros::self_referencing;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub struct Graph {
    holder: Holder,
}

#[self_referencing]
struct Holder {
    input: String,
    #[borrows(input)]
    #[covariant]
    graph: Result<graph::read_dot::DotGraph<'this>, String>,
}

#[wasm_bindgen]
pub struct SearchResultItem {
    id: String,
    label: String,
}

#[wasm_bindgen]
impl SearchResultItem {
    pub fn id(&self) -> JsValue {
        self.id.clone().into()
    }

    pub fn label(&self) -> JsValue {
        self.label.clone().into()
    }
}

#[wasm_bindgen]
impl Graph {
    pub fn new(dot: &JsValue) -> Self {
        let input: String = dot.as_string().unwrap();
        Self {
            holder: HolderBuilder {
                input,
                graph_builder: |input| {
                    graph::read_dot::parse(input)
                        .map(|mut g| {
                            info!(
                                "New Graph {} nodes and {} edges",
                                g.graph.nodes_count(),
                                g.graph.edges_count()
                            );
                            graph::to_dag::to_dag(&mut g.graph);
                            g
                        })
                        .inspect_err(|e| log::error!("Parse failed: {e}"))
                },
            }
            .build(),
        }
    }

    pub fn node_count(&self) -> JsValue {
        match &self.holder.borrow_graph() {
            Ok(g) => g.graph.nodes_count().into(),
            Err(e) => e.into(),
        }
    }

    pub fn is_error(&self) -> JsValue {
        self.holder.borrow_graph().is_err().into()
    }

    pub fn render(&self, around_node_id: &str, max_nodes: u32, max_edges: u32) -> JsValue {
        match &self.holder.borrow_graph() {
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
        let value = &value.to_lowercase();
        (if let Ok(dot) = &self.holder.borrow_graph() {
            dot.graph
                .iter_nodes_ids()
                .filter_map(|id| {
                    let &oid = dot.graph.original_id(id).unwrap();
                    let label = dot.labels.get(id).unwrap_or("");
                    (oid.to_lowercase().contains(value) || label.to_lowercase().contains(value))
                        .then(|| SearchResultItem {
                            id: oid.to_string(),
                            label: label.to_string(),
                        })
                })
                .map(|id| JsValue::from(id))
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
