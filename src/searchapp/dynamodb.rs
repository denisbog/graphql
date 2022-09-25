use std::collections::HashMap;

use aws_sdk_dynamodb::model::AttributeValue;

use aws_sdk_dynamodb::{Client};

use crate::searchapp::model::Post;

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

pub async fn get_data_from_dynamodb(client: &Client) -> Vec<Post> {
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