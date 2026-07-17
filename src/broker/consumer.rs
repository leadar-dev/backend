use std::sync::Arc;

use futures_lite::StreamExt;
use lapin::{
    options::{
        BasicAckOptions, BasicConsumeOptions, BasicNackOptions, QueueBindOptions,
        QueueDeclareOptions,
    },
    types::FieldTable,
    Channel, Connection, ConnectionProperties,
};
use sqlx::PgPool;
use tracing::{debug, error, info, instrument, warn};

use crate::broker::publisher::Publisher;
use crate::errors::AppError;
use crate::models::want::{KworkWantPayload, MessageEnvelope};
use crate::services::wants as wants_service;

const QUEUE_NAME: &str = "backend.wants";
const EXCHANGE_NAME: &str = "leadar.events";
const BINDING_KEY: &str = "parser.*.want";
const MAX_RETRIES: u32 = 3;

pub async fn start_consumer(
    broker_url: &str,
    pool: PgPool,
    publisher: Arc<Publisher>,
) -> anyhow::Result<()> {
    info!("connecting to broker for consumer");
    let conn = connect_with_retry(broker_url).await?;
    let channel = conn
        .create_channel()
        .await
        .map_err(|e| anyhow::anyhow!("failed to create channel: {e}"))?;

    setup_queue(&channel).await?;

    let mut consumer = channel
        .basic_consume(
            QUEUE_NAME,
            "backend-consumer",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await
        .map_err(|e| anyhow::anyhow!("failed to start consume: {e}"))?;

    info!(queue = QUEUE_NAME, "consumer started");

    while let Some(delivery) = consumer.next().await {
        match delivery {
            Ok(delivery) => {
                let routing_key = delivery.routing_key.as_str().to_string();
                debug!(routing_key = %routing_key, size = delivery.data.len(), "message received");

                match handle_message(&delivery.data, &pool, &publisher).await {
                    Ok(()) => {
                        if let Err(e) = delivery.ack(BasicAckOptions::default()).await {
                            error!(err = %e, "failed to ack message");
                        }
                    }
                    Err(e) => {
                        error!(err = %e, routing_key = %routing_key, "message handling failed, sending to DLQ");
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
                error!(err = %e, "consumer delivery error");
            }
        }
    }

    warn!("consumer stream ended, connection may have dropped");
    Ok(())
}

#[instrument(skip(data, pool, publisher))]
async fn handle_message(
    data: &[u8],
    pool: &PgPool,
    publisher: &Arc<Publisher>,
) -> Result<(), AppError> {
    let envelope: MessageEnvelope = serde_json::from_slice(data)
        .map_err(|e| AppError::InvalidPayload(format!("envelope deserialize error: {e}")))?;

    debug!(event = %envelope.event, version = envelope.version, "processing envelope");

    match envelope.event.as_str() {
        "parser.kwork.want" | "parser.fl.want" | "parser.upwork.want" => {
            let payload: KworkWantPayload = serde_json::from_value(envelope.payload)
                .map_err(|e| AppError::InvalidPayload(format!("payload deserialize error: {e}")))?;

            let upsert_result = wants_service::upsert(pool, &payload).await?;

            if upsert_result.is_insert {
                info!(want_id = upsert_result.id, source = %payload.source, "new want, publishing backend.want.new");
                publisher
                    .publish_want_new(&payload, upsert_result.id)
                    .await
                    .map_err(|e| AppError::Broker(e.to_string()))?;
            } else {
                debug!(want_id = upsert_result.id, "want updated (not new), skipping publish");
            }
        }
        unknown => {
            warn!(event = %unknown, "unknown event type, skipping");
        }
    }

    Ok(())
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
        .map_err(|e| anyhow::anyhow!("queue declare failed: {e}"))?;

    channel
        .queue_bind(
            QUEUE_NAME,
            EXCHANGE_NAME,
            BINDING_KEY,
            QueueBindOptions::default(),
            FieldTable::default(),
        )
        .await
        .map_err(|e| anyhow::anyhow!("queue bind failed: {e}"))?;

    info!(queue = QUEUE_NAME, binding = BINDING_KEY, "queue setup complete");
    Ok(())
}

async fn connect_with_retry(broker_url: &str) -> anyhow::Result<Connection> {
    let mut last_err = None;
    for attempt in 1..=MAX_RETRIES {
        match Connection::connect(broker_url, ConnectionProperties::default()).await {
            Ok(conn) => {
                info!(attempt, "broker connection established");
                return Ok(conn);
            }
            Err(e) => {
                let delay =
                    tokio::time::Duration::from_millis(500 * 2u64.pow(attempt - 1));
                warn!(attempt, err = %e, delay_ms = delay.as_millis(), "broker connect failed, retrying");
                tokio::time::sleep(delay).await;
                last_err = Some(e);
            }
        }
    }
    Err(anyhow::anyhow!(
        "broker connect failed after {MAX_RETRIES} attempts: {last_err:?}"
    ))
}
