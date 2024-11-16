use std::hash::Hasher;

use bytes::Bytes;
use chrono::{DateTime, Utc};
use eyre::Context;
use futures_util::TryStreamExt;
use object_store::{GetResultPayload, ObjectStore};
use poem::{
    http::StatusCode,
    web::{Json, Path},
    Route,
};
use serde::Serialize;
use tokio::io::AsyncReadExt;
use twox_hash::XxHash3_64;
use uuid::Uuid;

use crate::{
    config::{CONFIG, OBJECT_STORE},
    handler::{auth::User, error::WrapRespErr},
    types::{Mod, References, Repository},
};

#[derive(Debug, Serialize)]
struct ModResp {
    name: String,
    last_modified: DateTime<Utc>,
    size: usize,
}

#[poem::handler]
#[tracing::instrument]
async fn list_mods(_: User) -> Result<Json<Vec<ModResp>>, (StatusCode, eyre::Report)> {
    let mut output = Vec::new();
    let mut stream = OBJECT_STORE.list(None);
    while let Some(meta) = stream.try_next().await.wrap_resp_err(
        StatusCode::INTERNAL_SERVER_ERROR,
        "failed to read object store",
    )? {
        if let Some(name) = meta.location.filename() {
            output.push(ModResp {
                name: name.to_string(),
                last_modified: meta.last_modified,
                size: meta.size,
            });
        }
    }
    output.sort_by_key(|m| m.last_modified);
    output.reverse();
    Ok(Json(output))
}

#[tracing::instrument]
async fn reindex() -> eyre::Result<()> {
    let mut mods = Vec::new();
    let mut stream = OBJECT_STORE.list(None);
    while let Some(meta) = stream
        .try_next()
        .await
        .wrap_err("failed to read object store")?
    {
        if let Some(name) = meta.location.filename() {
            let res = OBJECT_STORE
                .get(&meta.location)
                .await
                .wrap_err("failed to get object from object store")?;

            let mut hasher = XxHash3_64::new();

            match res.payload {
                GetResultPayload::File(file, _) => {
                    let mut file = tokio::fs::File::from_std(file);
                    let mut buf = Vec::new();
                    file.read_to_end(&mut buf)
                        .await
                        .wrap_err("failed to read file")?;
                    hasher.write(&buf);
                }
                GetResultPayload::Stream(mut stream) => {
                    while let Some(buf) = stream
                        .try_next()
                        .await
                        .wrap_err("failed to read stream from object store")?
                    {
                        hasher.write(&buf);
                    }
                }
            }

            let hash = hasher.finish();

            mods.push(Mod {
                ident: name.to_string(),
                file: meta.location.to_string(),
                bytes: meta.size,
                xxhsum: format!("{:x}", hash),
            });
        }
    }
    mods.sort_by_key(|m| m.ident.clone());

    let references = References {
        count: mods.len(),
        mods,
    };
    let repository = Repository {
        uuid: Uuid::new_v5(&Uuid::NAMESPACE_DNS, CONFIG.public_url.as_str().as_bytes()),
        title: CONFIG.title.clone(),
        downpath: String::new(),
        references,
    };
    let index =
        quick_xml::se::to_string(&repository).wrap_err("failed to serialize repository index")?;

    OBJECT_STORE
        .put(&object_store::path::Path::from("index.omx"), index.into())
        .await
        .wrap_err("failed to put index to object store")?;

    Ok(())
}

#[poem::handler]
#[tracing::instrument]
async fn request_reindex(_: User) -> Result<(), (StatusCode, eyre::Report)> {
    reindex()
        .await
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error))
}

#[poem::handler]
#[tracing::instrument]
async fn upload_mod(
    _: User,
    Path(name): Path<String>,
    body: Bytes,
) -> Result<(), (StatusCode, eyre::Report)> {
    let name = name.strip_suffix(".ozp").unwrap_or(&name);
    let filename = format!("{name}.ozp");
    OBJECT_STORE
        .put(&object_store::path::Path::from(filename), body.into())
        .await
        .wrap_resp_err(
            StatusCode::INTERNAL_SERVER_ERROR,
            "failed to put object to object store",
        )?;

    reindex()
        .await
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error))
}

pub fn create_route() -> Route {
    Route::new()
        .at("/mod", poem::get(list_mods))
        .at("/reindex", poem::post(request_reindex))
        .at("/mod/:name", poem::post(upload_mod))
}