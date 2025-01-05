use crate::database;
use crate::files::ImageFileManager;
use crate::models::Image;
use rusqlite::Connection;
use std::fs;

pub fn insert_image(
    conn: &Connection,
    file_manager: &ImageFileManager,
    image_data: &[u8],
    mime_type: &mime::Mime, // Take mime type as parameter
    image: Image,
) -> Result<(), Box<dyn std::error::Error>> {
    let filename = file_manager.save_file(image_data, &image.slug, mime_type)?;

    let image_to_save = Image { filename, ..image };

    database::insert_image(conn, &image_to_save).map_err(|e| {
        let _ = file_manager.delete_file(&image_to_save.filename);
        e.into()
    })
}

pub fn update_image(
    conn: &Connection,
    file_manager: &ImageFileManager,
    old_slug: &str,
    image_data: Option<(Vec<u8>, mime::Mime)>, // Take ownership instead of borrowing
    image: Image,
) -> Result<(), Box<dyn std::error::Error>> {
    let existing = database::get_image_by_slug(conn, old_slug)?;

    let filename = if let Some((data, mime_type)) = image_data {
        // Delete old file
        file_manager.delete_file(&existing.filename)?;

        // Save new file
        file_manager.save_file(&data, &image.slug, &mime_type)?
    } else {
        // Keep existing file but might need to rename if slug changed
        if old_slug != image.slug {
            file_manager.rename_file(&existing.filename, &image.slug)?
        } else {
            existing.filename
        }
    };

    let image_to_save = Image { filename, ..image };

    database::update_image(conn, old_slug, &image_to_save).map_err(|e| e.into())
}

pub fn delete_image(
    conn: &Connection,
    file_manager: &ImageFileManager,
    slug: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let image = database::get_image_by_slug(conn, slug)?;
    file_manager.delete_file(&image.filename)?;
    database::delete_image(conn, slug)?;
    Ok(())
}

/// CLI usage
pub fn insert_image_from_path(
    conn: &Connection,
    file_manager: &ImageFileManager,
    source_path: &str,
    image: Image,
) -> Result<(), Box<dyn std::error::Error>> {
    let image_data = fs::read(source_path)?;
    let mime_type = mime_guess::from_path(source_path).first_or_octet_stream();

    insert_image(conn, file_manager, &image_data, &mime_type, image)
}
