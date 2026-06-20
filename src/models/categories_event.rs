use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CategoryItem {
    pub external_id: i32,
    pub name: String,
    pub parent_external_id: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct KworkCategoriesPayload {
    pub source: String,
    pub categories: Vec<CategoryItem>,
}
