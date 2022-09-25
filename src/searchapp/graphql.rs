use aws_sdk_dynamodb::{model::AttributeValue, Client, Error};
use std::{collections::HashMap, convert::Infallible, sync::Arc};
use tokio::sync::RwLock;

use juniper::{graphql_object, EmptySubscription, GraphQLInputObject, GraphQLObject, RootNode};

use hyper::Body;
use crate::searchapp::state::{get_dynamodb_client};
struct Query;

struct Mutation;

#[derive(GraphQLInputObject, Clone)]
struct PostInput {
    id: String,
    created: String,
}

impl From<PostInput> for Post {
    fn from(user_input: PostInput) -> Self {
        Self {
            id: user_input.id,
            created: user_input.created,
        }
    }
}

#[derive(Clone, GraphQLObject, Debug)]
struct Post {
    id: String,
    created: String,
}

use hyper::{
    server::Server,
    service::{make_service_fn, service_fn},
    Method, Response, StatusCode,
};

struct Context {
    users: RwLock<HashMap<String, Post>>,
    client: Arc<Client>,
}

impl juniper::Context for Context {}

#[graphql_object(context = Context)]
impl Query {
    async fn users(context: &Context) -> Vec<Post> {
        let map = context.users.read().await;
        map.values().cloned().collect()
    }

    async fn posts(context: &Context) -> Vec<Post> {
        context
            .client
            .scan()
            .table_name("posts")
            .send()
            .await
            .unwrap()
            .items()
            .unwrap()
            .iter()
            .map(Post::from)
            .collect()
    }
}

#[graphql_object(context = Context)]
impl Mutation {
    #[graphql(name = "addUserIdName")]
    async fn add_user(context: &Context, id: String, created: String) -> Post {
        log::info!("create user by id and name");
        let mut map = context.users.write().await;
        let post: Post = Post {
            id: id,
            created: created,
        };
        map.insert(post.id.clone(), post.clone());
        post
    }

    async fn add_user(context: &Context, post_input: PostInput) -> Post {
        log::info!("create user");
        let mut map = context.users.write().await;
        let post = Post::from(post_input.clone());
        map.insert(post.id.clone(), post.clone());
        post
    }

    async fn add_post(context: &Context, post_input: PostInput) -> Post {
        context
            .client
            .put_item()
            .table_name("posts")
            .item("id", AttributeValue::S(post_input.id.clone()))
            .item("created", AttributeValue::S(post_input.created.clone()))
            .send()
            .await
            .unwrap();
        Post::from(post_input)
    }

    async fn delete_post(context: &Context, id: String) -> String {
        context
            .client
            .delete_item()
            .table_name("posts")
            .key("id", aws_sdk_dynamodb::model::AttributeValue::S(id.clone()))
            // .key("id", AttributeValue::S(id.clone()))
            .send()
            .await
            .unwrap();
        id
    }
}



impl From<&HashMap<String, AttributeValue>> for Post {
    fn from(attrs: &HashMap<String, AttributeValue>) -> Self {
        Post {
            id: attrs.get("id").unwrap().as_s().unwrap().to_string(),
            created: attrs.get("created").unwrap().as_s().unwrap().to_string(),
        }
    }
}
pub async fn server_local_index_data() -> Result<(), Error> {
    pretty_env_logger::init();
    let addr = ([127, 0, 0, 1], 3000).into();
    let root_node = Arc::new(RootNode::new(
        Query,
        Mutation,
        EmptySubscription::<Context>::new(),
    ));

    println!("{}", root_node.as_schema_language());

    let new_service = make_service_fn(move |_| {
        let root_node = root_node.clone();
        async {
            let context = Arc::new(Context {
                users: RwLock::new(HashMap::new()),
                client: Arc::new(get_dynamodb_client().await),
            });
            Ok::<_, hyper::Error>(service_fn(move |req| {
                let root_node = root_node.clone();
                let ctx = context.clone();
                async {
                    Ok::<_, Infallible>(match (req.method(), req.uri().path()) {
                        (&Method::GET, "/") => juniper_hyper::graphiql("/graphql", None).await,
                        (&Method::GET, "/graphql") | (&Method::POST, "/graphql") => {
                            juniper_hyper::graphql(root_node, ctx, req).await
                        }
                        _ => {
                            let mut response = Response::new(Body::empty());
                            *response.status_mut() = StatusCode::NOT_FOUND;
                            response
                        }
                    })
                }
            }))
        }
    });

    let server = Server::bind(&addr).serve(new_service);
    println!("Listening on http://{addr}");

    if let Err(e) = server.await {
        eprintln!("server error: {e}")
    }

    Ok(())
}
