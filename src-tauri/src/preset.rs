use crate::product::ProductKey;
use serde::Serialize;
use std::{
    cmp::Ordering,
    hash::{Hash, Hasher},
};

#[derive(Clone, Serialize)]
pub struct Preset {
    pub name: String,
    pub vendor: String,
    pub comment: String,
    #[serde(skip)]
    pub product_id: ProductKey,
    pub product_name: String,
    pub id: usize,
}

impl Ord for Preset {
    fn cmp(&self, other: &Self) -> Ordering {
        natord::compare_ignore_case(&self.name, &other.name)
    }
}

impl PartialOrd for Preset {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for Preset {}

impl PartialEq for Preset {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Hash for Preset {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}
