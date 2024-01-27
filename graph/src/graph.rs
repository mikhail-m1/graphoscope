use std::{
    env,
    fmt::{Debug, Display},
    fs::File,
    io::Write,
};

//TODO: can be optimized: use hashmap for creating
impl<T: Eq + Debug + Clone> DirectedGraph<T> {
    pub fn new(input_nodes: &[T], input_edges: &[(T, T)]) -> Self {
        let mut original_node_ids: Vec<_> = Vec::from(input_nodes);
        let mut nodes = vec![Node::default(); input_nodes.len()];

        let mut edges = Vec::with_capacity(input_edges.len());

        for (from_id, to_id) in input_edges {
            let from = original_node_ids
                .iter()
                .enumerate()
                .find(|(_, original_id)| (*original_id).eq(from_id))
                .map(|(n, _)| n)
                .unwrap_or_else(|| {
                    original_node_ids.push(from_id.clone());
                    nodes.push(Node::default());
                    nodes.len() - 1
                })
                .into();

            let to = original_node_ids
                .iter()
                .enumerate()
                .find(|(_, original_id)| (*original_id).eq(to_id))
                .map(|(n, _)| n)
                .unwrap_or_else(|| {
                    original_node_ids.push(to_id.clone());
                    nodes.push(Node::default());
                    nodes.len() - 1
                })
                .into();

            edges.push(Edge::new(from, to));
            let edge_id = edges.len() - 1;

            nodes[from.0 as usize].outputs.push(edge_id.into());
            nodes[to.0 as usize].inputs.push(edge_id.into());
        }

        Self {
            roots: nodes
                .iter()
                .enumerate()
                .filter(|(_, node)| node.inputs.is_empty())
                .map(|(n, _)| n.into())
                .collect(),
            nodes,
            edges,
            original_node_ids,
        }
    }
}

impl<T> DirectedGraph<T> {
    pub fn roots(&self) -> &[NodeId] {
        &self.roots[..]
    }

    pub fn add_root(&mut self, node_id: NodeId) {
        self.roots.push(node_id);
    }

    pub fn nodes_count(&self) -> u32 {
        self.nodes.len() as u32
    }

    pub fn node(&self, id: NodeId) -> &Node {
        &self.nodes[id.0 as usize]
    }

    pub fn original_id(&self, id: NodeId) -> Option<&T> {
        // sometimes there is no originals for temporary graphs
        if self.original_node_ids.is_empty() || self.node(id).is_virtual {
            None
        } else {
            Some(&self.original_node_ids[id.0 as usize])
        }
    }

    pub fn node_mut(&mut self, id: NodeId) -> &mut Node {
        &mut self.nodes[id.0 as usize]
    }

    pub fn add_node(&mut self, node: Node) -> NodeId {
        self.nodes.push(node);
        NodeId::from(self.nodes_count() - 1)
    }

    pub fn iter_nodes(&self) -> impl Iterator<Item = &Node> {
        self.nodes.iter()
    }

    pub fn iter_nodes_with_id(&self) -> impl Iterator<Item = (NodeId, &Node)> {
        self.nodes
            .iter()
            .enumerate()
            .map(|(n, node)| (NodeId::from(n), node))
    }

    pub fn iter_children(&self, id: NodeId) -> impl Iterator<Item = NodeId> + '_ {
        self.node(id)
            .outputs
            .iter()
            .map(move |edge_id| self.edge(*edge_id).to)
    }

    pub fn iter_parents(&self, id: NodeId) -> impl Iterator<Item = NodeId> + '_ {
        self.node(id)
            .inputs
            .iter()
            .map(move |edge_id| self.edge(*edge_id).from)
    }

    pub fn iter_node_edges(
        &self,
        id: NodeId,
    ) -> impl Iterator<Item = (&Node, EdgeId, &Edge, Direction)> {
        let node = self.node(id);
        node.outputs
            .iter()
            .map(move |&e_id| (e_id, Direction::Output))
            .chain(node.inputs.iter().map(|&e_id| (e_id, Direction::Input)))
            .map(move |(e_id, d)| (node, e_id, self.edge(e_id), d))
    }

    pub fn edges_count(&self) -> u32 {
        self.edges.len() as u32
    }

    pub fn edge(&self, id: EdgeId) -> &Edge {
        &self.edges[id.0 as usize]
    }

    pub fn edge_mut(&mut self, id: EdgeId) -> &mut Edge {
        &mut self.edges[id.0 as usize]
    }

    pub fn iter_edges_with_id(&self) -> impl Iterator<Item = (EdgeId, &Edge)> {
        self.edges
            .iter()
            .enumerate()
            .map(|(n, node)| (EdgeId::from(n), node))
    }

    pub fn iter_edges(&self) -> impl Iterator<Item = &Edge> {
        self.edges.iter()
    }

    pub fn for_each_edge_mut<F>(&mut self, f: &mut F)
    where
        F: FnMut(&mut DirectedGraph<T>, EdgeId),
    {
        for n in 0..self.edges.len() {
            f(self, EdgeId::from(n as u32));
        }
    }

    pub fn add_edge(&mut self, edge: Edge) -> EdgeId {
        self.edges.push(edge);
        EdgeId::from(self.edges.len() as u32 - 1)
    }

    pub fn node_map<V: Default + Clone>(&self) -> NodeMap<V> {
        NodeMap::new(self.nodes_count())
    }

    pub fn edge_map<V: Default + Clone>(&self) -> EdgeMap<V> {
        let mut values = vec![];
        values.resize(self.edges_count() as usize, V::default());
        EdgeMap { values }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn dump<NodeFilter, EdgeFilter, NodeFormater, EdgeFormater>(
        &self,
        _filename: &str,
        _node_formater: &NodeFormater,
        _edge_formater: &EdgeFormater,
        _node_filter: &NodeFilter,
        _edge_filter: &EdgeFilter,
    ) where
        NodeFormater: Fn(&DirectedGraph<T>, NodeId, &Node) -> String,
        EdgeFormater: Fn(&DirectedGraph<T>, EdgeId, &Edge) -> String,
        NodeFilter: Fn(&DirectedGraph<T>, NodeId) -> bool,
        EdgeFilter: Fn(&DirectedGraph<T>, EdgeId) -> bool,
    {
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn dump<NodeFilter, EdgeFilter, NodeFormater, EdgeFormater>(
        &self,
        filename_prefix: &str,
        node_formater: &NodeFormater,
        edge_formater: &EdgeFormater,
        node_filter: &NodeFilter,
        edge_filter: &EdgeFilter,
    ) where
        NodeFormater: Fn(&DirectedGraph<T>, NodeId, &Node) -> String,
        EdgeFormater: Fn(&DirectedGraph<T>, EdgeId, &Edge) -> String,
        NodeFilter: Fn(&DirectedGraph<T>, NodeId) -> bool,
        EdgeFilter: Fn(&DirectedGraph<T>, EdgeId) -> bool,
    {
        let Ok(ending) = env::var("GS_DUMP_STEPS") else {
            return;
        };

        let mut file = File::create(&format!("{filename_prefix}_{ending}.dot")).unwrap();
        let mut buf = "digraph temp {".to_string();

        for (id, node) in self
            .iter_nodes_with_id()
            .filter(|(id, _)| node_filter(self, *id))
        {
            buf += &if node.is_virtual {
                format!("{}[label=\"{} v\"];\n", id.0, node_formater(self, id, node))
            } else {
                format!("{}[label=\"{}\"];\n", id.0, node_formater(self, id, node))
            };
        }

        for (id, edge) in self.iter_edges_with_id()
        //.filter(|(id, _)| edge_filter(self, *id))
        {
            if edge_filter(self, id) {
                buf += &format!("{} -> {}", edge.from.0, edge.to.0)
            } else {
                buf += &format!("{} -> {}[style=\"dotted\"]", edge.from.0, edge.to.0)
            }
            buf += &format!("[label=\"{}\"]\n;", edge_formater(self, id, edge));
        }

        buf += "}";
        file.write_all(buf.as_bytes()).unwrap();
    }
}

impl<T: Display> DirectedGraph<T> {
    pub fn dot_result<W: Write>(&self, mut write: W, ranks: &NodeMap<i32>, places: &NodeMap<u32>) {
        write
            .write_all("digraph temp {\n".as_bytes())
            .expect("success");

        for (id, _) in self.iter_nodes_with_id().filter(|&(_, n)| !n.is_virtual) {
            write
                .write_all(
                    format!(
                        "{}[pos=\"{},{}\"];\n",
                        self.original_id(id).unwrap(),
                        places.get(id),
                        ranks.get(id),
                    )
                    .as_bytes(),
                )
                .expect("success");
        }

        for edge in self.iter_edges() {
            let (from, to) = if edge.is_inverted() {
                (edge.to, edge.from)
            } else {
                (edge.from, edge.to)
            };
            let from_name = self
                .original_id(from)
                .map(|v| format!("{}", v))
                .unwrap_or(format!("v_{}", from.0));

            let to_name = self
                .original_id(from)
                .map(|v| format!("{}", v))
                .unwrap_or(format!("v_{}", to.0));

            write
                .write_all(
                    format!(
                        "{} -> {}[pos=\"e{},{} {},{}\"];\n",
                        from_name,
                        to_name,
                        places.get(from),
                        ranks.get(from),
                        places.get(to),
                        ranks.get(to)
                    )
                    .as_bytes(),
                )
                .expect("success");
        }

        write.write_all("}".as_bytes()).expect("success")
    }
}

impl<T: Default + Clone> NodeMap<T> {
    pub fn new(size: u32) -> Self {
        Self {
            values: vec![T::default(); size as usize],
        }
    }

    pub fn get(&self, id: NodeId) -> &T {
        assert!((id.0 as usize) < self.values.len());
        &self.values[id.0 as usize]
    }

    pub fn get_mut(&mut self, id: NodeId) -> &mut T {
        assert!((id.0 as usize) < self.values.len());
        &mut self.values[id.0 as usize]
    }

    pub fn set(&mut self, id: NodeId, value: T) {
        if id.0 as usize >= self.values.len() {
            self.values.resize(id.0 as usize + 1, T::default())
        }
        self.values[id.0 as usize] = value;
    }

    pub fn find_first<P>(&self, predicate: P) -> Option<NodeId>
    where
        P: Fn(&T) -> bool,
    {
        self.values
            .iter()
            .enumerate()
            .filter(|(_, v)| predicate(v))
            .next()
            .map(|(n, _)| NodeId(n as u32))
    }

    pub fn iter(&self) -> impl Iterator<Item = (NodeId, &T)> {
        self.values
            .iter()
            .enumerate()
            .map(|(n, v)| (NodeId::from(n as u32), v))
    }

    pub fn iter_ids(&self) -> impl Iterator<Item = NodeId> {
        (0..self.values.len()).map(|n| NodeId::from(n as u32))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (NodeId, &mut T)> {
        self.values
            .iter_mut()
            .enumerate()
            .map(|(n, v)| (NodeId::from(n as u32), v))
    }
}

impl<T: Default + Clone> EdgeMap<T> {
    pub fn get(&self, id: EdgeId) -> &T {
        assert!((id.0 as usize) < self.values.len());
        &self.values[id.0 as usize]
    }

    pub fn get_mut(&mut self, id: EdgeId) -> &mut T {
        assert!((id.0 as usize) < self.values.len());
        &mut self.values[id.0 as usize]
    }

    pub fn set(&mut self, id: EdgeId, value: T) {
        if id.0 as usize >= self.values.len() {
            self.values.resize(id.0 as usize + 1, T::default())
        }
        self.values[id.0 as usize] = value;
    }

    pub fn find_first<P>(&self, predicate: P) -> Option<EdgeId>
    where
        P: Fn(&T) -> bool,
    {
        self.values
            .iter()
            .enumerate()
            .filter(|(_, v)| predicate(v))
            .next()
            .map(|(n, _)| EdgeId(n as u32))
    }

    pub fn iter(&self) -> impl Iterator<Item = (EdgeId, &T)> {
        self.values
            .iter()
            .enumerate()
            .map(|(n, v)| (EdgeId::from(n as u32), v))
    }

    pub fn iter_ids(&self) -> impl Iterator<Item = EdgeId> {
        (0..self.values.len()).map(|n| EdgeId::from(n as u32))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (EdgeId, &mut T)> {
        self.values
            .iter_mut()
            .enumerate()
            .map(|(n, v)| (EdgeId::from(n as u32), v))
    }
}

impl Node {
    pub fn to_output(&mut self, edge_id: EdgeId) {
        self.inputs.retain(|&id| id != edge_id);
        self.outputs.push(edge_id);
    }

    pub fn to_input(&mut self, edge_id: EdgeId) {
        self.outputs.retain(|&id| id != edge_id);
        self.inputs.push(edge_id);
    }

    pub fn virt() -> Self {
        Node {
            inputs: vec![],
            outputs: vec![],
            is_virtual: true,
        }
    }

    pub fn edges(&self) -> impl Iterator<Item = EdgeId> + '_ {
        self.inputs.iter().chain(self.outputs.iter()).map(|v| *v)
    }
}

impl Edge {
    pub fn new(from: NodeId, to: NodeId) -> Self {
        Self {
            from,
            to,
            kind: EdgeKind::Normal,
            min_length: 1,
            weight: 1,
        }
    }

    pub fn new_inverted(from: NodeId, to: NodeId) -> Self {
        Self {
            from,
            to,
            kind: EdgeKind::Inverted,
            min_length: 1, //TODO copy
            weight: 1,
        }
    }

    pub fn invert(&mut self) {
        std::mem::swap(&mut self.from, &mut self.to);
        self.kind = EdgeKind::Inverted;
    }

    pub fn is_inverted(&self) -> bool {
        self.kind == EdgeKind::Inverted
    }

    pub(crate) fn other_side(&self, direction: Direction) -> NodeId {
        if direction == Direction::Output {
            self.to
        } else {
            self.from
        }
    }
}

#[derive(Default)]
pub struct DirectedGraph<T> {
    roots: Vec<NodeId>,
    nodes: Vec<Node>,
    edges: Vec<Edge>,
    original_node_ids: Vec<T>,
}

#[derive(Default, Clone)]
pub struct Node {
    pub inputs: Vec<EdgeId>,
    pub outputs: Vec<EdgeId>,
    pub is_virtual: bool,
}

#[derive(Clone)]
pub struct Edge {
    pub from: NodeId,
    pub to: NodeId,
    pub kind: EdgeKind,
    pub min_length: u32,
    pub weight: i32,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum EdgeKind {
    Normal,
    Inverted,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(u32);

impl From<u32> for NodeId {
    fn from(i: u32) -> Self {
        NodeId(i)
    }
}

impl From<usize> for NodeId {
    fn from(i: usize) -> Self {
        NodeId(i as u32)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct EdgeId(u32);

pub const UNEXISTED_EDGE_ID: EdgeId = EdgeId(u32::max_value());

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Direction {
    Input,
    Output,
}

impl From<u32> for EdgeId {
    fn from(i: u32) -> Self {
        EdgeId(i)
    }
}

impl From<usize> for EdgeId {
    fn from(i: usize) -> Self {
        EdgeId(i as u32)
    }
}

#[derive(Clone, PartialEq)]
pub struct NodeMap<T> {
    values: Vec<T>,
}

pub struct EdgeMap<T> {
    values: Vec<T>,
}

impl<T: Debug> Debug for DirectedGraph<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        struct NodeFmt<'a, T: Debug>(&'a Node, &'a DirectedGraph<T>, NodeId);
        impl<'a, T: Debug> Debug for NodeFmt<'a, T> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let mut m = f.debug_struct(&format!("{:?}", self.2));
                if !self.0.inputs.is_empty() {
                    m.field("i", &NodeLinksFmt(self.1, self.2, Direction::Input));
                }
                if !self.0.outputs.is_empty() {
                    m.field("o", &NodeLinksFmt(self.1, self.2, Direction::Output));
                }
                m.finish()
            }
        }

        struct NodesFmt<'a, T: Debug>(&'a DirectedGraph<T>);
        impl<'a, T: Debug> Debug for NodesFmt<'a, T> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_list()
                    .entries(
                        self.0
                            .iter_nodes_with_id()
                            .map(|(id, node)| NodeFmt(node, self.0, id)),
                    )
                    .finish()
            }
        }

        struct NodeLinksFmt<'a, T>(&'a DirectedGraph<T>, NodeId, Direction);
        impl<'a, T> Debug for NodeLinksFmt<'a, T> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let mut l = f.debug_list();
                let node = self.0.node(self.1);
                match self.2 {
                    Direction::Input => {
                        for &id in &node.inputs {
                            l.entry(&self.0.edge(id).from);
                        }
                    }
                    Direction::Output => {
                        for &id in &node.outputs {
                            l.entry(&self.0.edge(id).to);
                        }
                    }
                }
                l.finish()
            }
        }

        f.debug_struct("DirectedGraph")
            .field("roots", &self.roots)
            .field("\nnodes", &NodesFmt(self))
            .field("\nedges", &self.edges)
            .finish()
    }
}

impl Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut m = f.debug_struct(if self.is_virtual { "VNode" } else { "Node" });
        if !self.inputs.is_empty() {
            m.field("i", &self.inputs);
        }
        if !self.outputs.is_empty() {
            m.field("o", &self.outputs);
        }
        m.finish()
    }
}

impl Debug for Edge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut m = f.debug_struct(&format!(
            "{}{:?}->{:?}",
            if self.is_inverted() { "inv" } else { "" },
            self.from,
            self.to
        ));
        if self.min_length != 1 {
            m.field("mlen", &self.min_length);
        }
        if self.weight != 1 {
            m.field("w", &self.weight);
        }
        m.finish()
    }
}

impl<T: Debug> Debug for NodeMap<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut m = f.debug_map();
        for (id, v) in self.values.iter().enumerate() {
            m.entry(&NodeId::from(id as u32), v);
        }
        m.finish()
    }
}

impl<T: Debug> Debug for EdgeMap<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut m = f.debug_map();
        for (id, v) in self.values.iter().enumerate() {
            m.entry(&id, v);
        }
        m.finish()
    }
}

impl Debug for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("N{}", self.0))
    }
}

impl Debug for EdgeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("E{}", self.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn simple() {
        DirectedGraph::new(&["a", "b"], &[("a", "b")]).assert(
            &[Node::with_outputs(&[0]), Node::with_inputs(&[0])],
            &[Edge::new(NodeId(0), NodeId(1))],
        );
    }

    #[test]
    fn loop_graph() {
        DirectedGraph::new(&["a", "b"], &[("a", "b"), ("b", "a")]).assert(
            &[Node::with_both(&[1], &[0]), Node::with_both(&[0], &[1])],
            &[
                Edge::new(NodeId(0), NodeId(1)),
                Edge::new(NodeId(1), NodeId(0)),
            ],
        );
    }

    impl Node {
        pub fn with_inputs(inputs: &[u32]) -> Self {
            Self {
                inputs: inputs.iter().map(|&id| EdgeId(id)).collect(),
                outputs: vec![],
                is_virtual: false,
            }
        }
        pub fn with_outputs(outputs: &[u32]) -> Self {
            Self {
                inputs: vec![],
                outputs: outputs.iter().map(|&id| EdgeId(id)).collect(),
                is_virtual: false,
            }
        }
        pub fn with_both(inputs: &[u32], outputs: &[u32]) -> Self {
            Self {
                inputs: inputs.iter().map(|&id| EdgeId(id)).collect(),
                outputs: outputs.iter().map(|&id| EdgeId(id)).collect(),
                is_virtual: false,
            }
        }
    }

    impl<T: Clone + Eq + std::fmt::Debug> DirectedGraph<T> {
        pub fn assert(&self, nodes: &[Node], edges: &[Edge]) {
            assert_eq!(self.nodes.len(), nodes.len(), "Nodes count");
            for ((id, node_ext), node_int) in self.iter_nodes_with_id().zip(&self.nodes) {
                assert_eq!(
                    node_int.inputs,
                    node_ext.inputs,
                    "inputs for {:?}",
                    self.original_id(id)
                );
                assert_eq!(
                    node_int.outputs,
                    node_ext.outputs,
                    "outputs for {:?}",
                    self.original_id(id)
                );
            }
            assert_eq!(self.edges.len(), edges.len());
            for (n, (edge_ext, edge_int)) in edges.iter().zip(self.edges.iter()).enumerate() {
                assert_eq!(edge_int.from, edge_ext.from, "Node {}", n);
                assert_eq!(edge_int.to, edge_ext.to, "Node {}", n);
                assert_eq!(edge_int.kind, edge_ext.kind, "Node {}", n);
            }
        }
    }
}
