use crate::models::{CustomError, Image, LoginCredentials};
use rusqlite::Connection;
use std::sync::{Arc, Mutex};
use warp::multipart::FormData;
use warp::Reply;

use bytes::Buf;
use futures::TryStreamExt;

pub async fn admin_create_image_handler(
    mut form: FormData,
    conn: Arc<Mutex<Connection>>,
) -> Result<impl Reply, warp::Rejection> {
    let mut alt = String::new();
    let mut description = String::new();
    let mut slug = String::new();
    let mut keywords = String::new();
    let mut image_data = Vec::new();

    while let Ok(Some(part)) = form.try_next().await {
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
                keywords = String::from_utf8(bytes).map_err(|e| {
                    warp::reject::custom(CustomError {
                        message: e.to_string(),
                    })
                })?;
            }
            "image" => {
                image_data = part
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
            }
            _ => (),
        }
    }

    // Validate the image data
    if image_data.is_empty() {
        return Err(warp::reject::custom(CustomError {
            message: "No image data provided".to_string(),
        }));
    }

    if image_data.len() > 5_000_000 {
        return Err(warp::reject::custom(CustomError {
            message: "File too large".to_string(),
        }));
    }

    let image = Image {
        alt,
        description,
        slug,
        keywords: keywords.split(',').map(|s| s.trim().to_string()).collect(),
        image_data,
    };

    let conn = conn.lock().unwrap();
    crate::database::insert_image(&conn, &image).expect("Failed to insert image");

    Ok(warp::reply::with_status(
        "Image created successfully!",
        warp::http::StatusCode::OK,
    ))
}

pub async fn admin_update_image_handler(
    slug: String,
    mut form: FormData,
    conn: Arc<Mutex<Connection>>,
) -> Result<impl Reply, warp::Rejection> {
    let mut alt = String::new();
    let mut description = String::new();
    let mut new_slug = String::new();
    let mut keywords = String::new();
    let mut image_data: Option<Vec<u8>> = None;

    while let Ok(Some(part)) = form.try_next().await {
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
                new_slug = String::from_utf8(bytes).map_err(|e| {
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
                keywords = String::from_utf8(bytes).map_err(|e| {
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
                        warp::reject::custom(CustomError {
                            message: e.to_string(),
                        })
                    })?;

                // Only set image_data if we actually received file data
                if !bytes.is_empty() {
                    if bytes.len() > 5_000_000 {
                        return Err(warp::reject::custom(CustomError {
                            message: "File too large".to_string(),
                        }));
                    }
                    image_data = Some(bytes);
                }
            }
            _ => (),
        }
    }

    let conn_guard = conn.lock().map_err(|e| {
        eprintln!("Failed to lock mutex: {:?}", e);
        warp::reject::custom(CustomError {
            message: "Internal server error".to_string(),
        })
    })?;

    // Get the existing image to preserve image_data if no new image was uploaded
    let existing_image = crate::database::get_image_by_slug(&conn_guard, &slug).map_err(|e| {
        eprintln!("Failed to get existing image: {:?}", e);
        warp::reject::custom(CustomError {
            message: "Image not found".to_string(),
        })
    })?;

    let updated_image = Image {
        alt,
        description,
        slug: new_slug,
        keywords: keywords.split(',').map(|s| s.trim().to_string()).collect(),
        image_data: image_data.unwrap_or(existing_image.image_data),
    };

    crate::database::update_image(&conn_guard, &slug, &updated_image).map_err(|e| {
        eprintln!("Failed to update image: {:?}", e);
        warp::reject::custom(CustomError {
            message: "Failed to update image".to_string(),
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
) -> Result<impl Reply, warp::Rejection> {
    let conn = conn.lock().unwrap();
    crate::database::delete_image(&conn, &slug).map_err(|e| {
        eprintln!("Failed to delete image: {:?}", e);
        warp::reject::custom(CustomError {
            message: "Failed to delete image".to_string(),
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
