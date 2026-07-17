use futures_lite::StreamExt;
use lapin::{
    options::{BasicAckOptions, BasicConsumeOptions, QueueDeclareOptions},
    types::FieldTable,
    Channel, Connection, ConnectionProperties,
};
use tracing::{error, info};

const DLQ_BACKEND_WANTS: &str = "backend.wants.dead";
const DLQ_BOT_NOTIFICATIONS: &str = "bot.notifications.dead";

pub async fn start_dlq_consumer(broker_url: &str) -> anyhow::Result<()> {
    info!("connecting to broker for DLQ consumer");
    let conn = Connection::connect(broker_url, ConnectionProperties::default())
        .await
        .map_err(|e| anyhow::anyhow!("DLQ broker connect failed: {e}"))?;

    let channel = conn
        .create_channel()
        .await
        .map_err(|e| anyhow::anyhow!("DLQ channel create failed: {e}"))?;

    for queue in [DLQ_BACKEND_WANTS, DLQ_BOT_NOTIFICATIONS] {
        channel
            .queue_declare(
                queue,
                QueueDeclareOptions {
                    durable: true,
                    ..Default::default()
                },
                FieldTable::default(),
            )
            .await
            .map_err(|e| {
                anyhow::anyhow!("DLQ queue declare failed for {queue}: {e}")
            })?;
    }

    let channel2 = conn
        .create_channel()
        .await
        .map_err(|e| anyhow::anyhow!("DLQ second channel create failed: {e}"))?;

    tokio::try_join!(
        consume_dlq_queue(channel, DLQ_BACKEND_WANTS),
        consume_dlq_queue(channel2, DLQ_BOT_NOTIFICATIONS),
    )?;

    Ok(())
}

async fn consume_dlq_queue(channel: Channel, queue_name: &'static str) -> anyhow::Result<()> {
    let mut consumer = channel
        .basic_consume(
            queue_name,
            &format!("backend-dlq-{queue_name}"),
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await
        .map_err(|e| anyhow::anyhow!("DLQ consume start failed: {e}"))?;

    info!(queue = queue_name, "DLQ consumer started");

    while let Some(delivery) = consumer.next().await {
        match delivery {
            Ok(delivery) => {
                let payload = String::from_utf8_lossy(&delivery.data);
                error!(
                    queue = queue_name,
                    payload = %payload,
                    "DLQ message received"
                );

                metrics::counter!("dlq_messages_total", "queue" => queue_name).increment(1);

                if let Err(e) = delivery.ack(BasicAckOptions::default()).await {
                    error!(err = %e, queue = queue_name, "failed to ack DLQ message");
                }
            }
            Err(e) => {
                error!(err = %e, queue = queue_name, "DLQ delivery error");
            }
        }
    }

    Ok(())
}
