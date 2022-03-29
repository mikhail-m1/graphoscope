use graph;
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

    pub fn render(&mut self) -> JsValue {
        match &mut self.graph {
            Err(e) => (String::from("<pre>") + e + "</pre>").into(),
            Ok(dot) => {
                // let mut ranks = graph::rank::rank(&mut dot_graph.graph);
                let mut ranks = graph::rank_with_components(&dot.graph);
                //    graph::ns::network_simplex(&mut dot.graph, graph::ns::Postprocess::None);
                graph::add_virtual_nodes::add_virtual_nodes(&mut dot.graph, &mut ranks);
                //FIXME: there is no reason to keep proccessing devided, need to be reworked
                let mut output = vec![];
                let places = graph::place::places3(&dot.graph, &ranks);
                let coords = graph::xcoord::x_coordinates(&dot.graph, &ranks, &places);
                graph::draw::draw(&dot, &ranks, &coords, &mut output);
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
