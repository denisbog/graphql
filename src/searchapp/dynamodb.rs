use std::collections::HashMap;

use aws_sdk_dynamodb::model::AttributeValue;

use aws_sdk_dynamodb::Client;

use crate::searchapp::model::Post;
use crate::searchapp::state::get_table_name;

use super::state::get_dynamodbstream_client;

impl From<&HashMap<String, AttributeValue>> for Post {
    fn from(attrs: &HashMap<String, AttributeValue>) -> Self {
        let mut post = Post {
            id: attrs.get("id").unwrap().as_s().unwrap().clone(),
            address: Default::default(),
            category: attrs.get("category").unwrap().as_s().unwrap().clone(),
            subcategory: Default::default(),
            created: Default::default(),
            title: Default::default(),
            description: Default::default(),
        };
        if attrs.get("description").is_some() {
            post.description = Some(attrs.get("description").unwrap().as_s().unwrap().clone());
        }
        if attrs.get("title").is_some() {
            post.title = Some(attrs.get("title").unwrap().as_s().unwrap().clone());
        }
        if attrs.get("subcategory").is_some() {
            post.subcategory = Some(attrs.get("subcategory").unwrap().as_s().unwrap().clone());
        }
        if attrs.get("created").is_some() {
            post.created = Some(attrs.get("created").unwrap().as_s().unwrap().clone());
        }
        if attrs.get("address").is_some() {
            post.address = Some(attrs.get("address").unwrap().as_s().unwrap().clone());
        }
        post
    }
}

pub async fn get_data_from_dynamodb(client: &Client) -> Vec<Post> {
    client
        .scan()
        .table_name(get_table_name())
        .send()
        .await
        .unwrap()
        .items()
        .unwrap()
        .iter()
        .map(Post::from)
        .collect()
}

pub async fn parse_streams() {
    let stream_client = get_dynamodbstream_client().await;

    let streams = stream_client
        .list_streams()
        .table_name(get_table_name())
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
                    i += 1;

                    if i > 10 {
                        break;
                    }
                    next_shard = shard_records.next_shard_iterator().unwrap().to_string();
                }
            }
        }
    }
}
