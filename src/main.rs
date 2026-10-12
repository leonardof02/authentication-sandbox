mod auth;
mod credentials;
mod health;
mod shared;
mod users;

use shared::infrastructure::database;
use shared::infrastructure::{AppState, Server, WebauthnConfig};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "auth_server=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let database_url = std::env::var("DATABASE_URL")?;
    let jwt_secret = std::env::var("JWT_SECRET").map_err(|_| "JWT_SECRET is not set")?;
    let webauthn = WebauthnConfig {
        rp_id: std::env::var("WEBAUTHN_RP_ID").map_err(|_| "WEBAUTHN_RP_ID is not set")?,
        origin: std::env::var("WEBAUTHN_ORIGIN").map_err(|_| "WEBAUTHN_ORIGIN is not set")?,
    };
    let db = database::pool::connect(&database_url).await?;

    Server::new(3000, AppState::new(db, &jwt_secret, &webauthn)?)
        .run()
        .await?;
    Ok(())
}
