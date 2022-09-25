

use graph;

#[tokio::main]
async fn main () {
    graph::serve().await;
}