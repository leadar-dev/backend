use chrono::Utc;
use lapin::{
    options::{BasicPublishOptions, ExchangeDeclareOptions},
    types::FieldTable,
    BasicProperties, Channel, Connection, ConnectionProperties, ExchangeKind,
};
use serde_json::json;
use tracing::{debug, instrument};

use crate::models::want::KworkWantPayload;

const EXCHANGE_NAME: &str = "leadar.events";
const ROUTING_KEY_WANT_NEW: &str = "backend.want.new";

pub struct Publisher {
    channel: Channel,
}

impl Publisher {
    pub async fn new(broker_url: &str) -> anyhow::Result<Self> {
        let conn = Connection::connect(broker_url, ConnectionProperties::default())
            .await
            .map_err(|e| anyhow::anyhow!("publisher broker connect failed: {}", e))?;

        let channel = conn
            .create_channel()
            .await
            .map_err(|e| anyhow::anyhow!("publisher channel create failed: {}", e))?;

        channel
            .exchange_declare(
                EXCHANGE_NAME,
                ExchangeKind::Topic,
                ExchangeDeclareOptions {
                    durable: true,
                    ..Default::default()
                },
                FieldTable::default(),
            )
            .await
            .map_err(|e| anyhow::anyhow!("exchange declare failed: {}", e))?;

        Ok(Self { channel })
    }

    #[instrument(skip(self, payload), fields(want_id = %internal_id, source = %payload.source))]
    pub async fn publish_want_new(
        &self,
        payload: &KworkWantPayload,
        internal_id: i64,
    ) -> anyhow::Result<()> {
        let envelope = json!({
            "event": "backend.want.new",
            "version": 1,
            "timestamp": Utc::now().to_rfc3339(),
            "payload": {
                "want_id": internal_id,
                "source": payload.source,
                "name": payload.name,
                "price_limit": payload.price_limit,
                "possible_price_limit": payload.possible_price_limit,
                "category_id": payload.category_id,
                "url": payload.url,
                "date_create": payload.date_create,
                "date_expire": payload.date_expire,
            }
        });

        let body = serde_json::to_vec(&envelope)
            .map_err(|e| anyhow::anyhow!("serialize failed: {}", e))?;

        self.channel
            .basic_publish(
                EXCHANGE_NAME,
                ROUTING_KEY_WANT_NEW,
                BasicPublishOptions::default(),
                &body,
                BasicProperties::default()
                    .with_content_type("application/json".into()),
            )
            .await
            .map_err(|e| anyhow::anyhow!("publish failed: {}", e))?
            .await
            .map_err(|e| anyhow::anyhow!("publish confirm failed: {}", e))?;

        debug!(routing_key = ROUTING_KEY_WANT_NEW, "message published");
        Ok(())
    }
}
