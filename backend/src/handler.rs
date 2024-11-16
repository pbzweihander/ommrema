mod api;
mod auth;
mod error;
mod middleware;

use poem::{endpoint::EmbeddedFilesEndpoint, EndpointExt, Route};
use rust_embed::Embed;

#[derive(Embed)]
#[folder = "../frontend/dist/"]
struct Frontend;

pub fn create_route() -> Route {
    let api = self::api::create_route();
    let auth = self::auth::create_route();

    Route::new()
        .nest("/api", api.with(self::middleware::Tracing))
        .nest("/auth", auth.with(self::middleware::Tracing))
        .nest("/", EmbeddedFilesEndpoint::<Frontend>::new())
}
