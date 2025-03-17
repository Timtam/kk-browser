use serde::{ser::SerializeSeq, Serialize, Serializer};
use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
};

#[derive(Clone, Serialize)]
pub struct CategoryNode {
    name: String,
    #[serde(serialize_with = "serialize_children")]
    children: HashMap<String, CategoryNode>,
    id: Option<usize>,
}

impl Hash for CategoryNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.name.hash(state);
    }
}

impl CategoryNode {
    pub fn new<S>(id: Option<usize>, name: S) -> Self
    where
        S: Into<String>,
    {
        CategoryNode {
            id,
            name: name.into(),
            children: HashMap::new(),
        }
    }

    pub fn append(&mut self, id: Option<usize>, mut path: Vec<String>) {
        if path.is_empty() || path.first().unwrap().is_empty() {
            return;
        }

        let nid: Option<usize> = if path.len() == 1 || path.get(1).unwrap().is_empty() {
            id
        } else {
            None
        };

        let child = self
            .children
            .entry(path.first().unwrap().clone())
            .and_modify(|e| {
                e.id = e.id.or(nid);
            })
            .or_insert(CategoryNode::new(nid, path.first().unwrap().clone()));

        if path.len() > 1 {
            child.append(id, path.split_off(1));
        }
    }
}

fn serialize_children<S>(t: &HashMap<String, CategoryNode>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut seq = s.serialize_seq(Some(t.len())).unwrap();
    t.values().for_each(|v| {
        seq.serialize_element(v).unwrap();
    });
    seq.end()
}
