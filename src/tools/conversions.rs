use sea_orm::{EntityTrait, ColumnTrait, QueryFilter, DatabaseConnection};
use tokio::time::{sleep, Duration};
use crate::models::conversion;
use sea_orm::QueryOrder;
use tokio::sync::watch;

/// Continuously processes the next pending/running conversion in the queue.
/// Will exit when the shutdown signal is received.
pub async fn process_conversion_queue(
  db: &DatabaseConnection,
  mut shutdown_rx: watch::Receiver<bool>,
) {
  loop {
    // Check for shutdown signal
    if *shutdown_rx.borrow() {
      println!("Conversion worker received shutdown signal. Exiting loop.");
      break;
    }

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
        conversion.process(db).await.unwrap_or_else(|e| {
          eprintln!("Error processing conversion id {}: {}", conversion.id, e);
        });
        println!("Processing conversion id: {}", conversion.id);
      }
      Ok(None) => {
        // No pending/running conversions, sleep before checking again
        tokio::select! {
          _ = sleep(Duration::from_secs(3)) => {},
          _ = shutdown_rx.changed() => {
            if *shutdown_rx.borrow() {
              println!("Conversion worker received shutdown signal during sleep. Exiting loop.");
              break;
            }
          }
        }
      }
      Err(e) => {
        eprintln!("Error querying conversions: {}", e);
        tokio::select! {
          _ = sleep(Duration::from_secs(5)) => {},
          _ = shutdown_rx.changed() => {
            if *shutdown_rx.borrow() {
              println!("Conversion worker received shutdown signal during error sleep. Exiting loop.");
              break;
            }
          }
        }
      }
    }
  }
}