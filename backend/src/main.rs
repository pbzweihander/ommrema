use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod handler;

#[tokio::main]
async fn main() {
    color_eyre::install().unwrap();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_error::ErrorLayer::default())
        .init();

    let app = crate::handler::create_route();

    poem::Server::new(poem::listener::TcpListener::bind("0.0.0.0:3000"))
        .run(app)
        .await
        .expect("failed to serve HTTP");
}
