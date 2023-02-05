use std::sync::Arc;

use anyhow::{anyhow, Context};
use deadpool_postgres::Pool;
use domain_schedule_models::dto::v1::ScheduleType;
use log::info;
use tokio_postgres::Row;

use crate::models::Peer;

pub struct PeerRepository {
    db_pool: Arc<Pool>,
}

#[derive(Debug, Clone)]
pub enum PlatformId {
    Telegram(i64),
    Vk(i64),
}

impl PeerRepository {
    pub fn new(db_pool: Arc<Pool>) -> Self {
        Self { db_pool }
    }

    pub async fn init_peer_tables(&self) -> anyhow::Result<()> {
        let client = self.db_pool.get().await?;
        let stmt = include_str!("../../sql/create_peer.pgsql");
        client
            .query(stmt, &[])
            .await
            .with_context(|| "Error during tables 'peer' creation")?;
        let stmt = include_str!("../../sql/create_peer_by_platform.pgsql");
        client
            .query(stmt, &[])
            .await
            .with_context(|| "Error during tables 'peer_by_platform' creation")?;
        info!("Tables 'peer' and 'peer_by_platform' initialization passed successfully");
        Ok(())
    }

    pub async fn get_peer_by_platform_id(&self, platform_id: PlatformId) -> anyhow::Result<Peer> {
        let client = self.db_pool.get().await?;
        let (platform, id) = match platform_id {
            PlatformId::Telegram(id) => ("telegram", id),
            PlatformId::Vk(id) => ("vk", id),
        };
        let stmt = format!(
            include_str!("../../sql/select_or_insert_peer.pgsql"),
            platform = platform,
            id = id
        );
        client
            .query(&stmt, &[])
            .await
            .with_context(|| "Error selecting peer from db")?
            .pop()
            .and_then(map_from_db_model)
            .ok_or_else(|| anyhow!("Error mapping peer from db"))
    }

    pub async fn save_peer(&self, peer: Peer) -> anyhow::Result<()> {
        let client = self.db_pool.get().await?;
        let stmt = format!(
            include_str!("../../sql/update_peer.pgsql"),
            id = peer.id,
            selected_schedule = peer.selected_schedule,
            selecting_schedule = peer.selecting_schedule,
        );
        client
            .query(&stmt, &[])
            .await
            .with_context(|| "Error updating peer in db")?;
        Ok(())
    }
}

fn map_from_db_model(row: Row) -> Option<Peer> {
    Some(Peer {
        id: row.try_get("id").ok()?,
        selected_schedule: row.try_get("selected_schedule").ok()?,
        selected_schedule_type: row
            .try_get::<_, String>("selected_schedule_type")
            .ok()
            .and_then(|v| v.parse::<ScheduleType>().ok())?,
        selecting_schedule: row.try_get("selecting_schedule").ok()?,
    })
}
