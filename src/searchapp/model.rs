use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Post {
    pub id: String,
    pub address: Option<String>,
    pub category: String,
    pub subcategory: Option<String>,
    pub created: Option<String>,
    pub description: Option<String>,
    pub title: Option<String>,
}