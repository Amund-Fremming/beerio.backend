use axum::Router;
use dotenvy::dotenv;
use std::env;
use tower_http::cors::{AllowHeaders, AllowMethods, AllowOrigin, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod db;
mod handlers;
mod models;
#[cfg(test)]
mod tests;

#[tokio::main]
async fn main() {
    dotenv().ok();

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = db::create_pool(&database_url).await;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    // Cleanup job: every hour delete rooms that have been inactive for 48+ hours.
    // Players are removed automatically via ON DELETE CASCADE.
    tokio::spawn({
        let pool = pool.clone();
        async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(60 * 60));
            loop {
                interval.tick().await;
                match sqlx::query(
                    "DELETE FROM rooms WHERE updated_at < NOW() - INTERVAL '48 hours'",
                )
                .execute(&pool)
                .await
                {
                    Ok(r) if r.rows_affected() > 0 => {
                        tracing::info!("Cleanup: removed {} inactive room(s)", r.rows_affected());
                    }
                    Ok(_) => {}
                    Err(e) => tracing::error!("Cleanup job error: {e}"),
                }
            }
        }
    });

    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::any())
        .allow_methods(AllowMethods::any())
        .allow_headers(AllowHeaders::any());

    let api = handlers::router().with_state(pool);
    let app: Router = Router::new().nest("/api", api).layer(cors);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::info!("Listening on http://0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}
