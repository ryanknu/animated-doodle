use aws_sdk_dynamodb::model::{
    AttributeDefinition, GlobalSecondaryIndex, KeySchemaElement, KeyType, Projection,
    ProjectionType, ProvisionedThroughput, ScalarAttributeType,
};

/// Things related to starting the container.
/// Mostly just stands up the database.
///
/// *Usually* this would be codified in terraform, but for the purposes of this
/// exercise, I'm just calling these API's on container start, and discarding
/// errors.
///
/// Terraform apply (at least for DynamoDB tables) should be ran as part of a
/// CI/CD pipeline, even if your other terraform is typically ran manually.
pub async fn init(client: &aws_sdk_dynamodb::Client) {
    create_messages_table(client).await;
    create_user_table(client).await;
}

pub async fn create_messages_table(client: &aws_sdk_dynamodb::Client) {
    let result = client
        .create_table()
        .table_name("messages")
        .attribute_definitions(
            AttributeDefinition::builder()
                .attribute_name("room_id")
                .attribute_type(ScalarAttributeType::N)
                .build(),
        )
        .attribute_definitions(
            AttributeDefinition::builder()
                .attribute_name("sort")
                .attribute_type(ScalarAttributeType::S)
                .build(),
        )
        .attribute_definitions(
            AttributeDefinition::builder()
                .attribute_name("name")
                .attribute_type(ScalarAttributeType::S)
                .build(),
        )
        .key_schema(
            KeySchemaElement::builder()
                .attribute_name("room_id")
                .key_type(KeyType::Hash)
                .build(),
        )
        .key_schema(
            KeySchemaElement::builder()
                .attribute_name("sort")
                .key_type(KeyType::Range)
                .build(),
        )
        .global_secondary_indexes(
            GlobalSecondaryIndex::builder()
                .index_name("name-index")
                .key_schema(
                    KeySchemaElement::builder()
                        .attribute_name("name")
                        .key_type(KeyType::Hash)
                        .build(),
                )
                .projection(
                    Projection::builder()
                        .projection_type(ProjectionType::KeysOnly)
                        .build(),
                )
                .provisioned_throughput(
                    ProvisionedThroughput::builder()
                        .read_capacity_units(5)
                        .write_capacity_units(5)
                        .build(),
                )
                .build(),
        )
        .provisioned_throughput(
            ProvisionedThroughput::builder()
                .read_capacity_units(5)
                .write_capacity_units(5)
                .build(),
        )
        .send()
        .await;

    println!("{:?}", result);
}

pub async fn create_user_table(client: &aws_sdk_dynamodb::Client) {
    let result = client
        .create_table()
        .table_name("users")
        .attribute_definitions(
            AttributeDefinition::builder()
                .attribute_name("user_id")
                .attribute_type(ScalarAttributeType::N)
                .build(),
        )
        .attribute_definitions(
            AttributeDefinition::builder()
                .attribute_name("name")
                .attribute_type(ScalarAttributeType::S)
                .build(),
        )
        .key_schema(
            KeySchemaElement::builder()
                .attribute_name("user_id")
                .key_type(KeyType::Hash)
                .build(),
        )
        .global_secondary_indexes(
            GlobalSecondaryIndex::builder()
                .index_name("name-index")
                .key_schema(
                    KeySchemaElement::builder()
                        .attribute_name("name")
                        .key_type(KeyType::Hash)
                        .build(),
                )
                .projection(
                    Projection::builder()
                        .projection_type(ProjectionType::KeysOnly)
                        .build(),
                )
                .provisioned_throughput(
                    ProvisionedThroughput::builder()
                        .read_capacity_units(5)
                        .write_capacity_units(5)
                        .build(),
                )
                .build(),
        )
        .provisioned_throughput(
            ProvisionedThroughput::builder()
                .read_capacity_units(5)
                .write_capacity_units(5)
                .build(),
        )
        .send()
        .await;

    println!("{:?}", result);
}
