use crate::config::Config;
use crate::models::CustomError;
use crate::templates::TEMPLATES;
use rusqlite::Connection;
use std::sync::{Arc, Mutex};
use tera::{Context, Tera};
use warp::Reply;

pub async fn home_handler(
    config: Arc<Config>,
    conn: Arc<Mutex<Connection>>,
) -> Result<impl Reply, warp::Rejection> {
    let mut context = Context::new();

    let conn_guard = conn.lock().map_err(|e| {
        eprintln!("Failed to lock mutex: {:?}", e);
        warp::reject::custom(CustomError {
            message: "Internal server error".to_string(),
        })
    })?;

    let images = crate::database::get_images(&conn_guard).map_err(|e| {
        eprintln!("Failed to get images: {:?}", e);
        warp::reject::custom(CustomError {
            message: "Failed to load images".to_string(),
        })
    })?;

    context.insert("title", &config.site.title);
    context.insert("description", &config.site.description);
    context.insert("images", &images);
    context.insert("site_name", &config.site.name);
    context.insert("base_url", &config.site.base_url);
    context.insert("detail_path", &config.routes.detail_path);
    context.insert("author", &config.meta.author);

    let rendered = TEMPLATES.render("home.html", &context).map_err(|e| {
        eprintln!("Template rendering error: {:?}", e);
        warp::reject::custom(CustomError {
            message: "Failed to render page".to_string(),
        })
    })?;

    Ok(warp::reply::html(rendered))
}

pub async fn post_detail_handler(
    slug: String,
    config: Arc<Config>,
    conn: Arc<Mutex<Connection>>,
) -> Result<impl Reply, warp::Rejection> {
    let tera = Tera::new("templates/**/*.html").expect("Failed to load templates");
    let mut context = Context::new();

    let conn_guard = conn.lock().map_err(|e| {
        eprintln!("Failed to lock mutex: {:?}", e);
        warp::reject::custom(CustomError {
            message: "Internal server error".to_string(),
        })
    })?;

    let image = crate::database::get_image_by_slug(&conn_guard, &slug).map_err(|e| {
        eprintln!("Failed to get image: {:?}", e);
        warp::reject::custom(CustomError {
            message: "Image not found".to_string(),
        })
    })?;

    context.insert(
        "title",
        &format!("{} - {}", image.alt, config.meta.creator_suffix),
    );
    context.insert("description", &image.description);
    context.insert("image", &image);
    context.insert("url", &config.get_detail_url(&slug));
    context.insert("site_name", &config.site.name);
    context.insert("base_url", &config.site.base_url);
    context.insert("author", &config.meta.author);

    let rendered = tera.render("post_detail.html", &context).map_err(|e| {
        eprintln!("Template rendering error: {:?}", e);
        warp::reject::custom(CustomError {
            message: "Failed to render page".to_string(),
        })
    })?;

    Ok(warp::reply::html(rendered))
}
