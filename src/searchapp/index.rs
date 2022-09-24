use tantivy::{
    schema::{Schema, FAST, INDEXED, TEXT},
    Index,
};

use crate::searchapp::model::Post;

#[tokio::main]
async fn main() {
    let db = sled::open("sled").expect("open");

    let mut schema_builder = Schema::builder();
    let id = schema_builder.add_bytes_field("id", INDEXED | FAST);
    let address = schema_builder.add_text_field("address", TEXT);
    let category = schema_builder.add_text_field("category", TEXT);
    let subcategory = schema_builder.add_text_field("subcategory", TEXT);
    let created = schema_builder.add_text_field("created", TEXT);
    let description = schema_builder.add_text_field("description", TEXT);
    let title = schema_builder.add_text_field("title", TEXT);

    let schema = schema_builder.build();

    // Indexing documents
    let dir = tantivy::directory::MmapDirectory::open("index").unwrap();

    let index = Index::open_or_create(dir, schema.clone()).unwrap();

    // Here we use a buffer of 100MB that will be split
    // between indexing threads.
    let mut index_writer = index.writer(100_000_000).unwrap();

    db.iter()
        .map(|item| {
            // println!("{}", std::str::from_utf8(&item.unwrap().0).unwrap());
            let post: Post =
                serde_json::from_str(std::str::from_utf8(&item.unwrap().1).unwrap()).unwrap();
            post
        })
        .for_each(|post| {
            let mut document = tantivy::Document::new();

            document.add_bytes(id, post.id);
            if post.address.is_some() {
                document.add_text(address, post.address.unwrap());
            }
            
            document.add_text(category, post.category);

            if post.subcategory.is_some() {
                document.add_text(subcategory, post.subcategory.unwrap());
            }
            if post.created.is_some() {
                document.add_text(created, post.created.unwrap());
            }
            if post.description.is_some() {
                document.add_text(description, post.description.unwrap());
            }
            if post.title.is_some() {
                document.add_text(title, post.title.unwrap());
            }

            index_writer.add_document(document).unwrap();
        });

    index_writer.commit().unwrap();
}
