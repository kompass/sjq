#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JsonPathStage {
    Node(String),
    Index(u64),
}

impl JsonPathStage {
    fn is_node(&self) -> bool {
        match self {
            &JsonPathStage::Node(_) => true,
            _ => false,
        }
    }

    fn is_index(&self) -> bool {
        match self {
            &JsonPathStage::Index(_) => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonPath(Vec<JsonPathStage>);

impl JsonPath {
    pub fn root() -> JsonPath {
        JsonPath(Vec::new())
    }

    pub fn push_node(&mut self, node_name: &str) {
        self.0.push(JsonPathStage::Node(node_name.to_string()));
    }

    pub fn pop_node(&mut self) {
        assert!(self.0.last().map_or(false, |x| x.is_node()));

        self.0.pop();
    }

    pub fn push_index(&mut self, index: u64) {
        self.0.push(JsonPathStage::Index(index));
    }

    pub fn inc_index(&mut self) {
        if let JsonPathStage::Index(ref mut i) = self.0.last_mut().unwrap() {
            *i += 1;
        } else {
            panic!("A node in a JsonPath can't be incremented.");
        }
    }

    pub fn pop_index(&mut self) {
        assert!(self.0.last().map_or(false, |x| x.is_index()));

        self.0.pop();
    }

    pub fn iter(&self) -> std::slice::Iter<'_, JsonPathStage> {
        self.0.iter()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}
