use crate::commands;
use crate::database;
use crate::files::ImageFileManager;
use crate::instagram;
use crate::models::{
    is_valid_slug, AdminSettingsForm, CustomError, Image, ImageStatusForm, InstagramConnection,
    LoginCredentials,
};
use crate::services::instagram_importer;
use chrono::Utc;
use rusqlite::Connection;
use std::sync::{Arc, Mutex};
use warp::multipart::FormData;
use warp::Reply;

use bytes::Buf;
use futures::TryStreamExt;

pub async fn admin_create_image_handler(
    form: FormData,
    conn: Arc<Mutex<Connection>>,
    file_manager: Arc<ImageFileManager>,
) -> Result<impl Reply, warp::Rejection> {
    let (image, image_data) = process_image_form(form).await?;

    let (data, mime_type) = image_data.ok_or_else(|| {
        println!("No image data provided in form");
        warp::reject::custom(CustomError::new("No image data provided".to_string()))
    })?;

    let conn_guard = conn.lock().map_err(|e| {
        println!("Failed to acquire database lock: {}", e);
        CustomError::new(e.to_string())
    })?;

    commands::insert_image(&conn_guard, &file_manager, &data, &mime_type, image).map_err(|e| {
        println!("Error in insert_image command: {}", e);
        warp::reject::custom(CustomError::new(e.to_string()))
    })?;
    println!("Image inserted successfully");

    Ok(warp::reply::with_status(
        "Image created successfully!",
        warp::http::StatusCode::OK,
    ))
}

pub async fn admin_update_image_handler(
    slug: String,
    form: FormData,
    conn: Arc<Mutex<Connection>>,
    file_manager: Arc<ImageFileManager>,
) -> Result<impl Reply, warp::Rejection> {
    let (image, image_data) = process_image_form(form).await?;

    let conn_guard = conn.lock().map_err(|_| {
        warp::reject::custom(CustomError {
            message: "Internal server error".to_string(),
        })
    })?;

    commands::update_image(&conn_guard, &file_manager, &slug, image_data, image).map_err(|e| {
        warp::reject::custom(CustomError {
            message: format!("Failed to update image: {}", e),
        })
    })?;

    Ok(warp::reply::with_status(
        "Image updated successfully!",
        warp::http::StatusCode::OK,
    ))
}

pub async fn admin_delete_image_handler(
    slug: String,
    conn: Arc<Mutex<Connection>>,
    file_manager: Arc<ImageFileManager>,
) -> Result<impl Reply, warp::Rejection> {
    let conn_guard = conn.lock().map_err(|_| {
        warp::reject::custom(CustomError {
            message: "Internal server error".to_string(),
        })
    })?;

    commands::delete_image(&conn_guard, &file_manager, &slug).map_err(|e| {
        warp::reject::custom(CustomError {
            message: format!("Failed to delete image: {}", e),
        })
    })?;

    Ok(warp::reply::with_status(
        "Image deleted successfully!",
        warp::http::StatusCode::OK,
    ))
}

pub async fn admin_restore_image_handler(
    slug: String,
    conn: Arc<Mutex<Connection>>,
) -> Result<impl Reply, warp::Rejection> {
    let conn_guard = conn.lock().map_err(|_| {
        warp::reject::custom(CustomError {
            message: "Internal server error".to_string(),
        })
    })?;

    commands::restore_image(&conn_guard, &slug).map_err(|e| {
        warp::reject::custom(CustomError {
            message: format!("Failed to restore image: {}", e),
        })
    })?;

    Ok(warp::reply::with_status(
        "Image restored successfully!",
        warp::http::StatusCode::OK,
    ))
}

pub async fn admin_update_status_handler(
    slug: String,
    form: ImageStatusForm,
    conn: Arc<Mutex<Connection>>,
) -> Result<impl Reply, warp::Rejection> {
    if !matches!(form.status.as_str(), "draft" | "published") {
        return Err(warp::reject::custom(CustomError::new(
            "Invalid status".to_string(),
        )));
    }

    let conn_guard = conn.lock().map_err(|_| {
        warp::reject::custom(CustomError {
            message: "Internal server error".to_string(),
        })
    })?;

    commands::update_image_status(&conn_guard, &slug, &form.status).map_err(|e| {
        warp::reject::custom(CustomError {
            message: format!("Failed to update image status: {}", e),
        })
    })?;

    Ok(warp::reply::with_status(
        "Image status updated successfully!",
        warp::http::StatusCode::OK,
    ))
}

pub async fn admin_update_settings_handler(
    form: AdminSettingsForm,
    conn: Arc<Mutex<Connection>>,
) -> Result<impl Reply, warp::Rejection> {
    if !matches!(form.default_import_status.as_str(), "draft" | "published") {
        return Err(warp::reject::custom(CustomError::new(
            "Invalid default import status".to_string(),
        )));
    }

    let verified_connection = if let Some(token) = form
        .instagram_access_token
        .as_ref()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
    {
        let identity = instagram::fetch_identity(token)
            .await
            .map_err(warp::reject::custom)?;

        Some((token.to_string(), identity))
    } else {
        None
    };

    let conn_guard = conn.lock().map_err(|_| {
        warp::reject::custom(CustomError {
            message: "Internal server error".to_string(),
        })
    })?;

    database::update_default_import_status(&conn_guard, &form.default_import_status).map_err(
        |e| {
            warp::reject::custom(CustomError {
                message: format!("Failed to update settings: {}", e),
            })
        },
    )?;

    if let Some((token, identity)) = verified_connection {
        let existing = database::get_instagram_connection(&conn_guard).map_err(|e| {
            warp::reject::custom(CustomError {
                message: format!("Failed to load existing Instagram connection: {}", e),
            })
        })?;

        let connection = InstagramConnection {
            instagram_user_id: identity.user_id,
            username: identity.username,
            access_token: token,
            token_expires_at: existing
                .as_ref()
                .and_then(|item| item.token_expires_at.clone()),
            connected_at: existing
                .map(|item| item.connected_at)
                .unwrap_or_else(|| Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()),
            last_sync_at: None,
        };

        database::save_instagram_connection(&conn_guard, &connection).map_err(|e| {
            warp::reject::custom(CustomError {
                message: format!("Failed to save Instagram token: {}", e),
            })
        })?;
    }

    Ok(warp::reply::with_status(
        "Settings updated successfully!",
        warp::http::StatusCode::OK,
    ))
}

pub async fn admin_instagram_disconnect_handler(
    conn: Arc<Mutex<Connection>>,
) -> Result<impl Reply, warp::Rejection> {
    let conn_guard = conn.lock().map_err(|_| {
        warp::reject::custom(CustomError {
            message: "Internal server error".to_string(),
        })
    })?;

    database::delete_instagram_connection(&conn_guard).map_err(|e| {
        warp::reject::custom(CustomError {
            message: format!("Failed to disconnect Instagram: {}", e),
        })
    })?;

    Ok(warp::reply::with_status(
        "Instagram disconnected successfully!",
        warp::http::StatusCode::OK,
    ))
}

pub async fn admin_instagram_refresh_handler(
    conn: Arc<Mutex<Connection>>,
) -> Result<impl Reply, warp::Rejection> {
    let result = instagram_importer::refresh_instagram_token(conn)
        .await
        .map_err(warp::reject::custom)?;

    Ok(warp::reply::with_status(
        format!(
            "Instagram token refreshed for @{} until {}.",
            result.username, result.token_expires_at
        ),
        warp::http::StatusCode::OK,
    ))
}

pub async fn admin_instagram_sync_handler(
    conn: Arc<Mutex<Connection>>,
    file_manager: Arc<ImageFileManager>,
) -> Result<impl Reply, warp::Rejection> {
    let result = instagram_importer::sync_instagram(conn, file_manager)
        .await
        .map_err(warp::reject::custom)?;

    Ok(warp::reply::with_status(
        result.message(),
        warp::http::StatusCode::OK,
    ))
}

pub async fn admin_login_handler(
    credentials: LoginCredentials,
    conn: Arc<Mutex<Connection>>,
) -> Result<impl Reply, warp::Rejection> {
    let conn_guard = conn.lock().map_err(|_| {
        warp::reject::custom(CustomError {
            message: "Internal server error".to_string(),
        })
    })?;

    // Verify credentials
    let is_valid =
        crate::database::verify_user(&conn_guard, &credentials.email, &credentials.password)
            .map_err(|_| {
                warp::reject::custom(CustomError {
                    message: "Authentication error".to_string(),
                })
            })?;

    if !is_valid {
        return Err(warp::reject::custom(CustomError {
            message: "Invalid credentials".to_string(),
        }));
    }

    // Get user ID
    let user_id: i64 = conn_guard
        .query_row(
            "SELECT id FROM users WHERE email = ?",
            [&credentials.email],
            |row| row.get(0),
        )
        .map_err(|_| {
            warp::reject::custom(CustomError {
                message: "User not found".to_string(),
            })
        })?;

    // Create session
    let session_id = crate::database::create_session(&conn_guard, user_id).map_err(|_| {
        warp::reject::custom(CustomError {
            message: "Failed to create session".to_string(),
        })
    })?;

    // Create response with session cookie
    let cookie = format!("session={}; Path=/; HttpOnly; SameSite=Strict", session_id);

    Ok(warp::reply::with_header(
        warp::reply::with_status("Login successful", warp::http::StatusCode::OK),
        "Set-Cookie",
        cookie,
    ))
}

pub async fn admin_logout_handler(
    session_id: Option<String>,
    conn: Arc<Mutex<Connection>>,
) -> Result<impl Reply, warp::Rejection> {
    // If we have a session cookie, delete it from the database
    if let Some(session_id) = session_id {
        let conn_guard = conn.lock().map_err(|_| {
            warp::reject::custom(CustomError {
                message: "Internal server error".to_string(),
            })
        })?;

        // Delete the session from the database
        if let Err(e) = crate::database::delete_session(&conn_guard, &session_id) {
            eprintln!("Failed to delete session: {:?}", e);
        }
    }

    // Clear the session cookie
    let cookie = "session=; Path=/; HttpOnly; SameSite=Strict; Max-Age=0";

    Ok(warp::reply::with_header(
        warp::reply::with_status("Logged out successfully", warp::http::StatusCode::OK),
        "Set-Cookie",
        cookie,
    ))
}

async fn process_image_form(
    mut form: FormData,
) -> Result<(Image, Option<(Vec<u8>, mime::Mime)>), warp::Rejection> {
    println!("Starting to process image form");

    let mut alt = String::new();
    let mut description = String::new();
    let mut slug = String::new();
    let mut keywords_str = String::new();
    let mut status = "published".to_string();
    let mut image_data = None;

    while let Ok(Some(part)) = form.try_next().await {
        println!("Processing form part: {}", part.name());

        let mime_type = if part.name() == "image" {
            let content_type = part.content_type();
            println!("Image content type from form: {:?}", content_type);

            content_type
                .map(|ct| {
                    let parsed = ct
                        .parse::<mime::Mime>()
                        .unwrap_or(mime::APPLICATION_OCTET_STREAM);
                    println!("Parsed mime type: {:?}", parsed);
                    parsed
                })
                .unwrap_or_else(|| {
                    println!("No content type provided, using default");
                    mime::APPLICATION_OCTET_STREAM
                })
        } else {
            mime::APPLICATION_OCTET_STREAM
        };

        match part.name() {
            "alt" => {
                let bytes = part
                    .stream()
                    .try_fold(Vec::new(), |mut vec, data| {
                        vec.extend_from_slice(data.chunk());
                        async move { Ok(vec) }
                    })
                    .await
                    .map_err(|e| {
                        warp::reject::custom(CustomError {
                            message: e.to_string(),
                        })
                    })?;
                alt = String::from_utf8(bytes).map_err(|e| {
                    warp::reject::custom(CustomError {
                        message: e.to_string(),
                    })
                })?;
            }
            "description" => {
                let bytes = part
                    .stream()
                    .try_fold(Vec::new(), |mut vec, data| {
                        vec.extend_from_slice(data.chunk());
                        async move { Ok(vec) }
                    })
                    .await
                    .map_err(|e| {
                        warp::reject::custom(CustomError {
                            message: e.to_string(),
                        })
                    })?;
                description = String::from_utf8(bytes).map_err(|e| {
                    warp::reject::custom(CustomError {
                        message: e.to_string(),
                    })
                })?;
            }
            "slug" => {
                let bytes = part
                    .stream()
                    .try_fold(Vec::new(), |mut vec, data| {
                        vec.extend_from_slice(data.chunk());
                        async move { Ok(vec) }
                    })
                    .await
                    .map_err(|e| {
                        warp::reject::custom(CustomError {
                            message: e.to_string(),
                        })
                    })?;
                slug = String::from_utf8(bytes).map_err(|e| {
                    warp::reject::custom(CustomError {
                        message: e.to_string(),
                    })
                })?;
            }
            "keywords" => {
                let bytes = part
                    .stream()
                    .try_fold(Vec::new(), |mut vec, data| {
                        vec.extend_from_slice(data.chunk());
                        async move { Ok(vec) }
                    })
                    .await
                    .map_err(|e| {
                        warp::reject::custom(CustomError {
                            message: e.to_string(),
                        })
                    })?;
                keywords_str = String::from_utf8(bytes).map_err(|e| {
                    warp::reject::custom(CustomError {
                        message: e.to_string(),
                    })
                })?;
            }
            "status" => {
                let bytes = part
                    .stream()
                    .try_fold(Vec::new(), |mut vec, data| {
                        vec.extend_from_slice(data.chunk());
                        async move { Ok(vec) }
                    })
                    .await
                    .map_err(|e| {
                        warp::reject::custom(CustomError {
                            message: e.to_string(),
                        })
                    })?;
                status = String::from_utf8(bytes).map_err(|e| {
                    warp::reject::custom(CustomError {
                        message: e.to_string(),
                    })
                })?;
            }
            "image" => {
                let bytes = part
                    .stream()
                    .try_fold(Vec::new(), |mut vec, data| {
                        vec.extend_from_slice(data.chunk());
                        async move { Ok(vec) }
                    })
                    .await
                    .map_err(|e| {
                        println!("Error reading image data: {:?}", e);
                        warp::reject::custom(CustomError {
                            message: e.to_string(),
                        })
                    })?;

                println!("Received image data: {} bytes", bytes.len());

                if bytes.is_empty() {
                    println!("Ignoring empty image upload");
                    continue;
                }

                if bytes.len() > 10_000_000 {
                    println!("File too large: {} bytes", bytes.len());
                    return Err(warp::reject::custom(CustomError::new(
                        "File too large".to_string(),
                    )));
                }

                println!(
                    "Successfully stored image data with mime type: {:?}",
                    mime_type
                );
                image_data = Some((bytes, mime_type));
            }
            _ => (),
        }
    }

    if alt.is_empty() || slug.is_empty() {
        return Err(warp::reject::custom(CustomError::new(
            "Missing required fields".to_string(),
        )));
    }

    if !is_valid_slug(&slug) {
        return Err(warp::reject::custom(CustomError::new(
            "Invalid slug. Use lowercase letters, numbers, and single hyphens only.".to_string(),
        )));
    }

    if !matches!(status.as_str(), "draft" | "published") {
        return Err(warp::reject::custom(CustomError::new(
            "Invalid image status".to_string(),
        )));
    }

    // Convert keywords string to Vec<String>
    let keywords = keywords_str
        .split(',')
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.trim().to_string())
        .collect();

    Ok((
        Image {
            alt,
            description,
            slug,
            keywords,
            filename: String::new(), // Will be set by command
            status,
            deleted_at: None,
            source: "manual".to_string(),
            source_media_id: None,
            source_permalink: None,
            source_timestamp: None,
        },
        image_data,
    ))
}
