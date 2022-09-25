use searchapp::{
    graphql::server_local_index_data, index::index_data_from_local_db, load::load_to_local_db, search::search_local_index,
};

mod searchapp;

pub async fn load() {
    load_to_local_db().await;
}

pub async fn index() {
    index_data_from_local_db().await;
}

pub async fn search() {
    search_local_index();
}

pub async fn server() {
    server_local_index_data().await.unwrap();
}
