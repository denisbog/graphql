use searchapp::{index::index_data_from_local_db, load::load_to_local_db, search, graphql::server_local_index_data};

mod searchapp;

pub async fn load() {
    load_to_local_db().await;
}

pub async fn index() {
    index_data_from_local_db().await;
}

pub async fn search() {
    searchapp::search::search();
}

pub async fn server() {
    server_local_index_data().await.unwrap();
}
