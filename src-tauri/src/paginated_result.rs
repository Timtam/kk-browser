use serde::Serialize;

#[derive(Serialize)]
pub struct PaginatedResult<T> {
    pub results: Vec<T>,
    pub total: usize,
    pub start: usize,
    pub end: usize,
}
