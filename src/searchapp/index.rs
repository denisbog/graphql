use crate::searchapp::model::Post;

use super::state::{get_index, get_local_db};

pub async fn index_data_from_local_db() {
    let index = get_index();

    let mut index_writer = index.writer(20_000_000).unwrap();

    let schema = index.schema();

    get_local_db()
        .iter()
        .map(|item| {
            // println!("{}", std::str::from_utf8(&item.unwrap().0).unwrap());
            let post: Post =
                serde_json::from_str(std::str::from_utf8(&item.unwrap().1).unwrap()).unwrap();
            post
        })
        .for_each(|post| {
            let mut document = tantivy::Document::new();

            document.add_bytes(schema.get_field("id").unwrap(), post.id);
            if post.address.is_some() {
                document.add_text(schema.get_field("address").unwrap(), post.address.unwrap());
            }

            document.add_text(schema.get_field("category").unwrap(), post.category);

            if post.subcategory.is_some() {
                document.add_text(
                    schema.get_field("subcategory").unwrap(),
                    post.subcategory.unwrap(),
                );
            }
            if post.created.is_some() {
                document.add_text(schema.get_field("created").unwrap(), post.created.unwrap());
            }
            if post.description.is_some() {
                document.add_text(
                    schema.get_field("description").unwrap(),
                    post.description.unwrap(),
                );
            }
            if post.title.is_some() {
                document.add_text(schema.get_field("title").unwrap(), post.title.unwrap());
            }

            index_writer.add_document(document).unwrap();
        });
    index_writer.commit().unwrap();
}
