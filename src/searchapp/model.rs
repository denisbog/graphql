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

#[juniper::graphql_object]
impl Post {
    async fn id(&self) -> String {
        self.id.clone()
    }
    async fn category(&self) -> String {
        self.category.clone()
    }
    async fn address(&self) -> Option<String> {
        self.address.clone()
    }
    async fn subcategory(&self) -> Option<String> {
        self.subcategory.clone()
    }
    async fn created(&self) -> Option<String> {
        self.created.clone()
    }
    async fn description(&self) -> Option<String> {
        self.description.clone()
    }
    async fn title(&self) -> Option<String> {
        self.title.clone()
    }
    async fn photos(&self) -> String {
        "some photo".to_string()
    }
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
