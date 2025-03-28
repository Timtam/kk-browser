use human_sort::compare;
use serde::Serialize;
use std::{
    cmp::Ordering,
    hash::{Hash, Hasher},
};

#[derive(Clone, Serialize)]
pub struct Product {
    pub id: usize,
    pub name: String,
    pub vendor: String,
}

impl Ord for Product {
    fn cmp(&self, other: &Self) -> Ordering {
        compare(&self.name, &other.name)
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
