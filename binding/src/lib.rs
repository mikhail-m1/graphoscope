use std::mem::swap;

use graph::{self, generator};
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

    pub fn render(&mut self) -> JsValue {
        let mut graph = Err("Done".to_string());
        swap(&mut self.graph, &mut graph);
        match graph {
            Err(e) => (String::from("<pre>") + &e + "</pre>").into(),
            Ok(dot) => {
                if dot.graph.nodes_count() == 0 {
                    return r#"<svg viewBox="0 0 1 1" xmlns="http://www.w3.org/2000/svg"></svg>"#
                        .into();
                }
                let output = graph::full_draw(dot);
                std::str::from_utf8(&output).unwrap().into()
            }
        }
    }
}

#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();
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
