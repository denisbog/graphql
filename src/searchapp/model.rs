use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Post {
    pub id: String,
    pub category: String,
    pub address: Option<String>,
    pub subcategory: Option<String>,
    pub created: Option<String>,
    pub description: Option<String>,
    pub title: Option<String>,
}

impl Post {
    pub fn new(id: String, created: String) -> Self {
        Post {
            id,
            address: Default::default(),
            category: Default::default(),
            subcategory: Default::default(),
            created: Some(created),
            description: Default::default(),
            title: Default::default(),
        }
    }
}
