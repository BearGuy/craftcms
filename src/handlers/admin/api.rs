use crate::commands;
use crate::files::ImageFileManager;
use crate::models::{CustomError, Image, LoginCredentials};
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

                if !bytes.is_empty() && bytes.len() > 10_000_000 {
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
        },
        image_data,
    ))
}
