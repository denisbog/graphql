#[tokio::main]
async fn main () {
    graph::load().await;
    graph::index().await;
}