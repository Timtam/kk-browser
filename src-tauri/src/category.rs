use serde::Serialize;
use std::collections::HashSet;

#[derive(Clone, Serialize)]
pub struct Category {
    pub id: usize,
    pub name: String,
    pub subcategory: String,
    pub subsubcategory: String,
    #[serde(skip)]
    pub presets: HashSet<usize>,
}

#[derive(Serialize)]
pub struct Mode {
    pub id: usize,
    pub name: String,
    #[serde(skip)]
    pub presets: HashSet<usize>,
}
