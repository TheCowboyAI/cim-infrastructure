// Copyright (c) 2025 - Cowboy AI, Inc.
//! Simple NATS connectivity test

use async_nats::jetstream;

#[tokio::test]
async fn test_nats_connection() -> Result<(), Box<dyn std::error::Error>> {
    println!("Attempting to connect to NATS at 10.0.20.1:4222...");

    let client = async_nats::connect("nats://10.0.20.1:4222").await?;
    println!("✅ Connected to NATS successfully");

    let _jetstream = jetstream::new(client);
    println!("✅ JetStream context created");

    Ok(())
}

#[tokio::test]
async fn test_create_stream() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing stream creation...");

    let client = async_nats::connect("nats://10.0.20.1:4222").await?;
    let jetstream = jetstream::new(client);

    let stream_name = format!("TEST_CONNECTIVITY_{}", uuid::Uuid::now_v7());

    let stream = jetstream
        .get_or_create_stream(async_nats::jetstream::stream::Config {
            name: stream_name.clone(),
            subjects: vec![format!("test.{}.>", stream_name)],
            ..Default::default()
        })
        .await?;

    println!("✅ Stream created: {}", stream.cached_info().config.name);

    // Clean up
    jetstream.delete_stream(&stream_name).await?;
    println!("✅ Stream deleted (cleanup)");

    Ok(())
}
