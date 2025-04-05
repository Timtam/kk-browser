use crate::product::ProductKey;
use serde::Serialize;
use std::{
    cmp::Ordering,
    collections::HashSet,
    hash::{Hash, Hasher},
    path::PathBuf,
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
    pub file_name: PathBuf,
    pub categories: HashSet<usize>,
    pub modes: HashSet<usize>,
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
