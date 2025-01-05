use mime::Mime;
use std::path::{Path, PathBuf};

pub struct ImageFileManager {
    base_path: PathBuf,
}

impl ImageFileManager {
    pub fn new(base_path: impl Into<PathBuf>) -> Self {
        Self {
            base_path: base_path.into(),
        }
    }

    pub fn save_file(
        &self,
        data: &[u8],
        slug: &str,
        mime_type: &Mime,
    ) -> Result<String, std::io::Error> {
        println!("Saving file with mime type: {:?}", mime_type);
        let extension = match (mime_type.type_(), mime_type.subtype()) {
            (mime::IMAGE, mime::JPEG) => "jpg",
            (mime::IMAGE, mime::PNG) => "png",
            _ => {
                println!("Unrecognized mime type, defaulting to jpg: {:?}", mime_type);
                "jpg"
            }
        };

        let filename = format!("{}.{}", slug, extension);
        let file_path = self.base_path.join(&filename);

        println!("Attempting to save file: {:?}", file_path);
        println!("Data length: {} bytes", data.len());

        match std::fs::write(&file_path, data) {
            Ok(_) => {
                println!("Successfully wrote file: {:?}", file_path);
                Ok(filename)
            }
            Err(e) => {
                println!("Error writing file: {:?}", e);
                Err(e)
            }
        }
    }

    pub fn delete_file(&self, filename: &str) -> std::io::Result<()> {
        let file_path = self.base_path.join(filename);

        // Debug info
        println!("Attempting to delete file: {:?}", file_path);

        if !file_path.exists() {
            println!("File doesn't exist, considering this a success");
            return Ok(());
        }

        match std::fs::remove_file(&file_path) {
            Ok(_) => Ok(()),
            Err(e) => {
                println!("Error deleting file: {:?}", e);
                println!("File permissions: {:?}", std::fs::metadata(&file_path));
                Err(e)
            }
        }
    }

    pub fn rename_file(&self, old_filename: &str, new_filename: &str) -> std::io::Result<String> {
        let old_path = self.base_path.join(old_filename);
        let extension = Path::new(old_filename)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("jpg");

        let new_filename = format!("{}.{}", new_filename, extension);
        let new_path = self.base_path.join(&new_filename);

        std::fs::rename(old_path, &new_path)?;
        Ok(new_filename) // Return just the filename, not the full path
    }
}
