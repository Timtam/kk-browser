use serde::Serialize;

#[derive(Serialize)]
pub struct Preset {
    pub name: String,
    pub vendor: String,
    pub comment: String,
    pub product: String,
    pub id: usize,
}
