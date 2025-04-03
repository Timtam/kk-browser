use serde::Serialize;
use std::{
    cmp::Ordering,
    collections::HashSet,
    hash::{Hash, Hasher},
};

#[derive(Clone, Eq, PartialEq, Hash)]
pub enum ProductKey {
    Id(usize),
    Upid(String),
}

#[derive(Clone, Serialize)]
pub struct Product {
    pub id: usize,
    pub name: String,
    #[serde(skip)]
    pub content_dir: String,
    pub vendor: String,
    #[serde(skip)]
    pub upid: String,
    #[serde(skip)]
    pub presets: HashSet<usize>,
}

impl Ord for Product {
    fn cmp(&self, other: &Self) -> Ordering {
        natord::compare_ignore_case(&self.name, &other.name)
    }
}

impl PartialOrd for Product {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for Product {}

impl PartialEq for Product {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Hash for Product {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}
