use aws_sdk_dynamodb::Client;
use tantivy::{
    schema::{Schema, FAST, INDEXED, TEXT},
    Index,
};

pub fn get_table_name() -> &'static str {
    "driveme"
}

pub async fn get_dynamodb_client() -> Client {
    let config = aws_config::from_env().region("eu-west-1").load().await;
    Client::new(&config)
}

pub async fn get_dynamodbstream_client() -> aws_sdk_dynamodbstreams::Client {
    let config = aws_config::from_env().region("eu-west-1").load().await;
    aws_sdk_dynamodbstreams::Client::new(&config)
}

pub fn get_local_db() -> sled::Db {
    sled::open("sled").expect("open")
}

pub fn get_schema() -> tantivy::schema::Schema {
    let mut schema_builder = Schema::builder();
    schema_builder.add_bytes_field("id", INDEXED | FAST);
    schema_builder.add_text_field("address", TEXT);
    schema_builder.add_text_field("category", TEXT);
    schema_builder.add_text_field("subcategory", TEXT);
    schema_builder.add_text_field("created", TEXT);
    schema_builder.add_text_field("description", TEXT);
    schema_builder.add_text_field("title", TEXT);
    schema_builder.build()
}

pub fn get_index() -> tantivy::Index {
    // Indexing documents
    let dir = tantivy::directory::MmapDirectory::open("index").unwrap();
    Index::open_or_create(dir, get_schema()).unwrap()
}
