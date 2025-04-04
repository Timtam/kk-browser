use serde::Serialize;
use std::{
    cmp::Ordering,
    collections::HashSet,
    hash::{Hash, Hasher},
};

#[derive(Clone, Serialize)]
pub struct Category {
    pub id: usize,
    pub name: String,
    pub subcategory: String,
    pub subsubcategory: String,
    #[serde(skip)]
    pub presets: HashSet<usize>,
}

impl Category {
    pub fn get_name(&self) -> String {
        let mut name = self.name.to_string();

        if !self.subcategory.is_empty() {
            name.push_str(&format!(" / {}", self.subcategory));
        }

        if !self.subsubcategory.is_empty() {
            name.push_str(&format!(" / {}", self.subsubcategory));
        }

        name
    }
}

impl Ord for Category {
    fn cmp(&self, other: &Self) -> Ordering {
        natord::compare_ignore_case(&self.get_name(), &other.get_name())
    }
}

impl PartialOrd for Category {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for Category {}

impl PartialEq for Category {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Hash for Category {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

#[derive(Serialize)]
pub struct Mode {
    pub id: usize,
    pub name: String,
    #[serde(skip)]
    pub presets: HashSet<usize>,
}
