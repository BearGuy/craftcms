use crate::commands;
use crate::database;
use crate::files::ImageFileManager;
use crate::instagram;
use crate::models::{CustomError, InstagramConnection};
use chrono::Utc;
use rusqlite::Connection;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Copy)]
pub struct InstagramSyncResult {
    pub imported: usize,
    pub skipped_existing: usize,
    pub skipped_deleted: usize,
}

#[derive(Debug, Clone)]
pub struct InstagramRefreshResult {
    pub username: String,
    pub token_expires_at: String,
}

impl InstagramSyncResult {
    pub fn message(&self) -> String {
        format!(
            "Instagram sync complete: imported {}, skipped existing {}, skipped deleted {}.",
            self.imported, self.skipped_existing, self.skipped_deleted
        )
    }
}

pub async fn sync_instagram(
    conn: Arc<Mutex<Connection>>,
    file_manager: Arc<ImageFileManager>,
) -> Result<InstagramSyncResult, CustomError> {
    let (settings, connection) = {
        let conn_guard = lock_connection(&conn)?;
        let settings = database::get_app_settings(&conn_guard)
            .map_err(|e| CustomError::new(format!("Failed to load settings: {e}")))?;
        let connection = database::get_instagram_connection(&conn_guard)
            .map_err(|e| CustomError::new(format!("Failed to load Instagram connection: {e}")))?
            .ok_or_else(|| {
                CustomError::new("Connect Instagram in settings before syncing".to_string())
            })?;
        (settings, connection)
    };

    let media = instagram::fetch_recent_media(&connection.access_token).await?;
    let flattened = instagram::flatten_media(&media);

    let mut result = InstagramSyncResult {
        imported: 0,
        skipped_existing: 0,
        skipped_deleted: 0,
    };

    for item in flattened {
        if let Some(deleted) = existing_media_deleted(&conn, &item.source_media_id)? {
            if deleted {
                result.skipped_deleted += 1;
            } else {
                result.skipped_existing += 1;
            }
            continue;
        }

        let (bytes, mime_type) = instagram::download_media(&item.media_url).await?;

        let inserted = {
            let conn_guard = lock_connection(&conn)?;
            match database::get_image_by_source_media_id(&conn_guard, &item.source_media_id) {
                Ok(existing) => {
                    if existing.deleted_at.is_some() {
                        result.skipped_deleted += 1;
                    } else {
                        result.skipped_existing += 1;
                    }
                    false
                }
                Err(rusqlite::Error::QueryReturnedNoRows) => {
                    let slug = unique_slug_for_import(&conn_guard, &item).map_err(|e| {
                        CustomError::new(format!("Failed to create import slug: {e}"))
                    })?;
                    let image = instagram::image_from_instagram_media(
                        &item,
                        &settings.default_import_status,
                        slug,
                    );

                    commands::insert_image(&conn_guard, &file_manager, &bytes, &mime_type, image)
                        .map_err(|e| {
                        CustomError::new(format!("Failed to import Instagram media: {e}"))
                    })?;
                    true
                }
                Err(e) => {
                    return Err(CustomError::new(format!(
                        "Failed during Instagram dedupe check: {e}"
                    )));
                }
            }
        };

        if inserted {
            result.imported += 1;
        }
    }

    let conn_guard = lock_connection(&conn)?;
    database::update_instagram_last_sync(
        &conn_guard,
        &Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    )
    .map_err(|e| CustomError::new(format!("Failed to record Instagram sync time: {e}")))?;

    Ok(result)
}

pub async fn refresh_instagram_token(
    conn: Arc<Mutex<Connection>>,
) -> Result<InstagramRefreshResult, CustomError> {
    let existing = {
        let conn_guard = lock_connection(&conn)?;
        database::get_instagram_connection(&conn_guard)
            .map_err(|e| CustomError::new(format!("Failed to load Instagram connection: {e}")))?
            .ok_or_else(|| {
                CustomError::new("Save an Instagram token before refreshing".to_string())
            })?
    };

    let refreshed = instagram::refresh_access_token(&existing.access_token).await?;
    let identity = instagram::fetch_identity(&refreshed.access_token).await?;

    let updated_connection = InstagramConnection {
        instagram_user_id: identity.user_id,
        username: identity.username.clone(),
        access_token: refreshed.access_token,
        token_expires_at: Some(refreshed.token_expires_at.clone()),
        connected_at: existing.connected_at,
        last_sync_at: existing.last_sync_at,
    };

    let conn_guard = lock_connection(&conn)?;
    database::save_instagram_connection(&conn_guard, &updated_connection)
        .map_err(|e| CustomError::new(format!("Failed to save refreshed Instagram token: {e}")))?;

    Ok(InstagramRefreshResult {
        username: identity.username,
        token_expires_at: refreshed.token_expires_at,
    })
}

fn existing_media_deleted(
    conn: &Arc<Mutex<Connection>>,
    source_media_id: &str,
) -> Result<Option<bool>, CustomError> {
    let conn_guard = lock_connection(conn)?;
    match database::get_image_by_source_media_id(&conn_guard, source_media_id) {
        Ok(existing) => Ok(Some(existing.deleted_at.is_some())),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(CustomError::new(format!(
            "Failed during Instagram dedupe check: {e}"
        ))),
    }
}

fn lock_connection(
    conn: &Arc<Mutex<Connection>>,
) -> Result<std::sync::MutexGuard<'_, Connection>, CustomError> {
    conn.lock()
        .map_err(|_| CustomError::new("Internal server error".to_string()))
}

fn unique_slug_for_import(
    conn: &Connection,
    item: &instagram::FlattenedInstagramMedia,
) -> Result<String, Box<dyn std::error::Error>> {
    let base_caption_slug = instagram::slugify(
        item.caption
            .lines()
            .find(|line| !line.trim().is_empty())
            .unwrap_or("instagram-import"),
    );
    let media_suffix: String = item
        .source_media_id
        .chars()
        .rev()
        .take(8)
        .collect::<String>()
        .chars()
        .rev()
        .collect();

    let mut candidate = if base_caption_slug.is_empty() {
        format!("instagram-{}", media_suffix)
    } else {
        base_caption_slug
    };

    if !database::slug_exists(conn, &candidate)? {
        return Ok(candidate);
    }

    candidate = format!("{}-{}", candidate, media_suffix);
    if !database::slug_exists(conn, &candidate)? {
        return Ok(candidate);
    }

    let mut counter = 2;
    loop {
        let fallback = format!("{}-{}", candidate, counter);
        if !database::slug_exists(conn, &fallback)? {
            return Ok(fallback);
        }
        counter += 1;
    }
}
