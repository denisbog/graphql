use aws_sdk_dynamodb::{Client, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let config = aws_config::from_env().region("eu-west-1").load().await;
    let client = Client::new(&config);
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

    let table = "posts";

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
