use poem::{endpoint::EmbeddedFilesEndpoint, Route};
use rust_embed::Embed;

#[derive(Embed)]
#[folder = "../frontend/dist/"]
struct Frontend;

pub fn create_route() -> Route {
    Route::new().nest("/", EmbeddedFilesEndpoint::<Frontend>::new())
}
