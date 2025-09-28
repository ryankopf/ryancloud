use sea_orm::{EntityTrait, ColumnTrait, QueryFilter, DatabaseConnection};
use tokio::time::{sleep, Duration};
use crate::models::conversion;
use sea_orm::QueryOrder;

/// Continuously processes the next pending/running conversion in the queue.
pub async fn process_conversion_queue(db: &DatabaseConnection) {
    loop {
        // Find the next conversion that is not completed or failed
        let next_conversion = conversion::Entity::find()
            .filter(
              conversion::Column::Status
                .is_in(vec!["pending", "running"])
            )
            .order_by(conversion::Column::TimeRequested, sea_orm::Order::Asc)
            .one(db)
            .await;

        match next_conversion {
            Ok(Some(conversion)) => {
              // Call your process function (to be implemented)
              conversion.process(db).await.unwrap_or_else(|e| {
                eprintln!("Error processing conversion id {}: {}", conversion.id, e);
              });
              println!("Processing conversion id: {}", conversion.id);
            }
            Ok(None) => {
              // No pending/running conversions, sleep before checking again
              sleep(Duration::from_secs(3)).await;
            }
            Err(e) => {
              eprintln!("Error querying conversions: {}", e);
              sleep(Duration::from_secs(5)).await;
            }
        }
    }
}