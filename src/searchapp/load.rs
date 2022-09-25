
use aws_sdk_dynamodb::{Client};
use sled::Db;

use crate::searchapp::model::Post;

async fn store_data_to_local_db(db: &Db, items: &Vec<Post>) {
    items.iter().for_each(|post| {
        db.insert(
            post.id.clone(),
            serde_json::to_string(post).unwrap().into_bytes(),
        )
        .expect("insert");
    })
}

pub async fn load_to_local_db() {
    let config = aws_config::from_env().region("eu-central-1").load().await;
    let client = Client::new(&config);

    let items = super::dynamodb::get_data_from_dynamodb(&client).await;
    let db = sled::open("sled").expect("open");
    store_data_to_local_db(&db, &items).await;
    db.flush().expect("flush");
}
