use searchapp::{index::index_data_from_local_db, load::load_to_local_db, graphql::serve_local_index_data};

mod searchapp;

pub async fn load() {
    load_to_local_db().await;
}

pub async fn index() {
    index_data_from_local_db().await;
}

// pub async fn search() {
//     search_local_index();
// }

pub async fn serve() {
    serve_local_index_data().await.unwrap();
}
