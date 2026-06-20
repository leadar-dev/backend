use futures_lite::StreamExt;
use lapin::{
    options::{
        BasicAckOptions, BasicConsumeOptions, BasicNackOptions, QueueDeclareOptions,
    },
    types::FieldTable,
    Channel, Connection, ConnectionProperties,
};
use sqlx::PgPool;
use tracing::{debug, error, info, instrument, warn};

use crate::db::categories as categories_db;
use crate::errors::AppError;
use crate::models::categories_event::KworkCategoriesPayload;
use crate::models::want::MessageEnvelope;

const QUEUE_NAME: &str = "backend.categories";
const MAX_RETRIES: u32 = 3;

pub async fn start_categories_consumer(broker_url: &str, pool: PgPool) -> anyhow::Result<()> {
    info!("connecting to broker for categories consumer");
    let conn = connect_with_retry(broker_url).await?;
    let channel = conn
        .create_channel()
        .await
        .map_err(|e| anyhow::anyhow!("failed to create channel: {}", e))?;

    setup_queue(&channel).await?;

    let mut consumer = channel
        .basic_consume(
            QUEUE_NAME,
            "backend-categories-consumer",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await
        .map_err(|e| anyhow::anyhow!("failed to start consume: {}", e))?;

    info!(queue = QUEUE_NAME, "categories consumer started");

    while let Some(delivery) = consumer.next().await {
        match delivery {
            Ok(delivery) => {
                debug!(size = delivery.data.len(), "categories message received");

                match handle_message(&delivery.data, &pool).await {
                    Ok(count) => {
                        info!(count, "categories upserted");
                        if let Err(e) = delivery.ack(BasicAckOptions::default()).await {
                            error!(err = %e, "failed to ack message");
                        }
                    }
                    Err(e) => {
                        error!(err = %e, "categories message handling failed, nacking");
                        if let Err(nack_err) = delivery
                            .nack(BasicNackOptions {
                                requeue: false,
                                ..Default::default()
                            })
                            .await
                        {
                            error!(err = %nack_err, "failed to nack message");
                        }
                    }
                }
            }
            Err(e) => {
                error!(err = %e, "categories consumer delivery error");
            }
        }
    }

    warn!("categories consumer stream ended");
    Ok(())
}

#[instrument(skip(data, pool))]
async fn handle_message(data: &[u8], pool: &PgPool) -> Result<usize, AppError> {
    let envelope: MessageEnvelope = serde_json::from_slice(data)
        .map_err(|e| AppError::InvalidPayload(format!("envelope deserialize error: {e}")))?;

    debug!(event = %envelope.event, "processing categories envelope");

    let payload: KworkCategoriesPayload = serde_json::from_value(envelope.payload)
        .map_err(|e| AppError::InvalidPayload(format!("categories payload deserialize error: {e}")))?;

    let count = categories_db::upsert_categories(pool, &payload).await?;
    Ok(count)
}

async fn setup_queue(channel: &Channel) -> anyhow::Result<()> {
    channel
        .queue_declare(
            QUEUE_NAME,
            QueueDeclareOptions {
                durable: true,
                passive: true,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await
        .map_err(|e| anyhow::anyhow!("queue declare failed: {}", e))?;

    info!(queue = QUEUE_NAME, "categories queue ready");
    Ok(())
}

async fn connect_with_retry(broker_url: &str) -> anyhow::Result<Connection> {
    let mut last_err = None;
    for attempt in 1..=MAX_RETRIES {
        match Connection::connect(broker_url, ConnectionProperties::default()).await {
            Ok(conn) => {
                info!(attempt, "broker connection established for categories consumer");
                return Ok(conn);
            }
            Err(e) => {
                let delay = tokio::time::Duration::from_millis(500 * 2u64.pow(attempt - 1));
                warn!(attempt, err = %e, delay_ms = delay.as_millis(), "broker connect failed, retrying");
                tokio::time::sleep(delay).await;
                last_err = Some(e);
            }
        }
    }
    Err(anyhow::anyhow!(
        "broker connect failed after {} attempts: {:?}",
        MAX_RETRIES,
        last_err
    ))
}
