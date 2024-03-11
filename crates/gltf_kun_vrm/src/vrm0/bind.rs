use gltf_kun::graph::{gltf::Primitive, ByteNode, Graph, NodeIndex, OtherEdgeHelpers};
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum BindEdges {
    #[serde(rename = "VRM/Bind/Primitive")]
    Primitive,
}

impl ToString for BindEdges {
    fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct BindWeight {
    pub weight: Option<f32>,
}

impl From<&Vec<u8>> for BindWeight {
    fn from(bytes: &Vec<u8>) -> Self {
        if bytes.is_empty() {
            return Self::default();
        }
        serde_json::from_slice(bytes).expect("Failed to deserialize weight")
    }
}

impl From<&BindWeight> for Vec<u8> {
    fn from(value: &BindWeight) -> Self {
        serde_json::to_vec(value).expect("Failed to serialize weight")
    }
}

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Bind(pub NodeIndex);

impl From<NodeIndex> for Bind {
    fn from(index: NodeIndex) -> Self {
        Self(index)
    }
}

impl From<Bind> for NodeIndex {
    fn from(bind: Bind) -> Self {
        bind.0
    }
}

impl ByteNode<BindWeight> for Bind {}
impl OtherEdgeHelpers for Bind {}

impl Bind {
    pub fn primitive(&self, graph: &Graph) -> Option<Primitive> {
        self.find_property(graph, &BindEdges::Primitive.to_string())
    }
    pub fn set_primitive(&self, graph: &mut Graph, primitive: Option<Primitive>) {
        self.set_property(graph, BindEdges::Primitive.to_string(), primitive);
    }
}
