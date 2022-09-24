use std::collections::HashMap;

use aws_sdk_dynamodb::{model::AttributeValue, Client};
use serde::{Deserialize, Serialize};
use sled::Db;

#[derive(Debug, Serialize, Deserialize)]
pub struct Post {
    id: String,
    address: Option<String>,
    category: String,
    subcategory: Option<String>,
    created: Option<String>,
    description: Option<String>,
    title: Option<String>,
}

impl From<&HashMap<String, AttributeValue>> for Post {
    fn from(attrs: &HashMap<String, AttributeValue>) -> Self {
        let mut post = Post {
            id: attrs.get("id").unwrap().as_s().unwrap().clone(),
            address: Default::default(),
            category: attrs.get("category").unwrap().as_s().unwrap().clone(),
            subcategory: Default::default(),
            created: Default::default(),
            title: Default::default(),
            description: Default::default(),
        };
        if attrs.get("description").is_some() {
            post.description = Some(attrs.get("description").unwrap().as_s().unwrap().clone());
        }
        if attrs.get("title").is_some() {
            post.title = Some(attrs.get("title").unwrap().as_s().unwrap().clone());
        }
        if attrs.get("subcategory").is_some() {
            post.subcategory = Some(attrs.get("subcategory").unwrap().as_s().unwrap().clone());
        }
        if attrs.get("created").is_some() {
            post.created = Some(attrs.get("created").unwrap().as_s().unwrap().clone());
        }
        if attrs.get("address").is_some() {
            post.address = Some(attrs.get("address").unwrap().as_s().unwrap().clone());
        }
        post
    }
}

async fn load_db(client: &Client) -> Vec<Post> {
    let table = "driveme";
    client
        .scan()
        .table_name(table)
        .send()
        .await
        .unwrap()
        .items()
        .unwrap()
        .iter()
        .map(|item| return Post::from(item))
        .collect()
}

async fn cache(db: &Db, items: &Vec<Post>) {
    items.iter().for_each(|post| {
        db.insert(
            post.id.clone(),
            serde_json::to_string(post).unwrap().into_bytes(),
        )
        .expect("insert");
    })
}

#[tokio::main]
async fn main() {
    let config = aws_config::from_env().region("eu-central-1").load().await;
    let client = Client::new(&config);
    let items = load_db(&client).await;
    println!("{}", items.len());
    println!("{:?}", items[0]);
    let db = sled::open("sled").expect("open");
    cache(&db, &items).await;
    db.flush().expect("flush");
}
