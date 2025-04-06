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

#[derive(Clone, Serialize)]
pub struct Mode {
    pub id: usize,
    pub name: String,
    #[serde(skip)]
    pub presets: HashSet<usize>,
}

impl Ord for Mode {
    fn cmp(&self, other: &Self) -> Ordering {
        natord::compare_ignore_case(&self.name, &other.name)
    }
}

impl PartialOrd for Mode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for Mode {}

impl PartialEq for Mode {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Hash for Mode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

#[derive(Clone, Serialize)]
pub struct Bank {
    pub id: usize,
    pub entry1: String,
    pub entry2: String,
    pub entry3: String,
    #[serde(skip)]
    pub presets: HashSet<usize>,
}

impl Bank {
    pub fn get_name(&self) -> String {
        let mut name = self.entry1.to_string();

        if !self.entry2.is_empty() {
            name.push_str(&format!(" / {}", self.entry2));
        }

        if !self.entry3.is_empty() {
            name.push_str(&format!(" / {}", self.entry3));
        }

        name
    }
}

impl Ord for Bank {
    fn cmp(&self, other: &Self) -> Ordering {
        natord::compare_ignore_case(&self.get_name(), &other.get_name())
    }
}

impl PartialOrd for Bank {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for Bank {}

impl PartialEq for Bank {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Hash for Bank {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}
