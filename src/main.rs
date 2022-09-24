use aws_sdk_dynamodb::{model::AttributeValue, Client, Error};
use std::{collections::HashMap, convert::Infallible, sync::Arc};
use tokio::sync::RwLock;
#[macro_use]
extern crate juniper;

#[macro_use]
extern crate log;

use juniper::{EmptySubscription, RootNode};

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
        info!("create user by id and name");
        let mut map = context.users.write().await;
        let post: Post = Post {
            id: id,
            created: created,
        };
        map.insert(post.id.clone(), post.clone());
        post
    }

    async fn add_user(context: &Context, post_input: PostInput) -> Post {
        info!("create user");
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
            .key(
                "id",
                aws_sdk_dynamodb::model::AttributeValue::S(id.clone()),
            )
            // .key("id", AttributeValue::S(id.clone()))
            .send()
            .await
            .unwrap();
        id
    }
}

use hyper::Body;

///
///
/// read data
/// query {users{id, name}}
///
/// create data
/// mutation {addUserIdName(id: "user4", name:"denis") {
///  id
/// }}
///
///
/// mutation {
///   addUser(userInput: {id: "name", name: "surname"}) {
///     id
///   }
/// }
///
///

impl From<&HashMap<String, AttributeValue>> for Post {
    fn from(attrs: &HashMap<String, AttributeValue>) -> Self {
        Post {
            id: attrs.get("id").unwrap().as_s().unwrap().to_string(),
            created: attrs.get("created").unwrap().as_s().unwrap().to_string(),
        }
    }
}
#[tokio::main]
async fn main() -> Result<(), Error> {
    pretty_env_logger::init();

    let config = aws_config::from_env().region("eu-west-1").load().await;
    let client = Client::new(&config);

    let table = "posts";

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
            let config = aws_config::from_env().region("eu-west-1").load().await;

            let context = Arc::new(Context {
                users: RwLock::new(HashMap::new()),
                client: Arc::new(Client::new(&config)),
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

    println!("Tables:");

    client
        .list_tables()
        .send()
        .await?
        .table_names()
        .unwrap()
        .iter()
        .for_each(|table| {
            println!("{:?}", table);
        });

    client
        .query()
        .table_name(table)
        .key_condition_expression("id =:id")
        .expression_attribute_values(
            ":id",
            aws_sdk_dynamodb::model::AttributeValue::S("123".to_string()),
        )
        .send()
        .await
        .unwrap()
        .items()
        .unwrap()
        .iter()
        .for_each(|item| println!("{:?}", item));

    let stream_client = aws_sdk_dynamodbstreams::Client::new(&config);

    let streams = stream_client
        .list_streams()
        .table_name(table)
        .send()
        .await
        .unwrap();

    for stream in streams.streams().unwrap().iter() {
        println!("{:?}", stream.stream_arn().unwrap());

        let stream_description = stream_client
            .describe_stream()
            .stream_arn(stream.stream_arn().unwrap())
            .send()
            .await
            .unwrap();

        for shard in stream_description
            .stream_description()
            .unwrap()
            .shards()
            .unwrap()
        {
            let shard_iterator = stream_client
                .get_shard_iterator()
                .stream_arn(stream.stream_arn().unwrap())
                .shard_iterator_type(aws_sdk_dynamodbstreams::model::ShardIteratorType::TrimHorizon)
                .shard_id(shard.shard_id().unwrap())
                .send()
                .await
                .unwrap();

            if let Some(shard_itor) = shard_iterator.shard_iterator() {
                let mut next_shard: String = shard_itor.to_string();
                let mut i = 0;
                loop {
                    let shard_records = stream_client
                        .get_records()
                        .shard_iterator(next_shard)
                        .send()
                        .await
                        .unwrap();

                    shard_records.records().unwrap().iter().for_each(|record| {
                        println!("{:?}", record);
                        i = 0;
                    });
                    i = i + 1;

                    if i > 10 {
                        break;
                    }
                    next_shard = shard_records.next_shard_iterator().unwrap().to_string();
                }
            }
        }
    }
    Ok(())
}
