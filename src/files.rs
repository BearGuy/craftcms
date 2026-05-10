use image::codecs::jpeg::JpegEncoder;
use mime::Mime;
use std::fs::File;
use std::path::{Path, PathBuf};

use image::GenericImageView;

use crate::models::is_valid_slug;

const VARIANT_WIDTHS: [u32; 3] = [480, 960, 1600];

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
        if !is_valid_slug(slug) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Invalid image slug",
            ));
        }

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

        if let Err(e) = std::fs::write(&file_path, data) {
            println!("Error writing file: {:?}", e);
            return Err(e);
        }
        println!("Successfully wrote file: {:?}", file_path);

        self.generate_variants(&filename)?;
        Ok(filename)
    }

    pub fn delete_file(&self, filename: &str) -> std::io::Result<()> {
        let file_path = self.base_path.join(filename);

        // Debug info
        println!("Attempting to delete file: {:?}", file_path);

        if !file_path.exists() {
            println!("File doesn't exist, considering this a success");
            self.delete_variants(filename)?;
            return Ok(());
        }

        match std::fs::remove_file(&file_path) {
            Ok(_) => {
                self.delete_variants(filename)?;
                Ok(())
            }
            Err(e) => {
                println!("Error deleting file: {:?}", e);
                println!("File permissions: {:?}", std::fs::metadata(&file_path));
                Err(e)
            }
        }
    }

    pub fn rename_file(&self, old_filename: &str, new_filename: &str) -> std::io::Result<String> {
        if !is_valid_slug(new_filename) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Invalid image slug",
            ));
        }

        let old_path = self.base_path.join(old_filename);
        let extension = Path::new(old_filename)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("jpg");

        let new_filename = format!("{}.{}", new_filename, extension);
        let new_path = self.base_path.join(&new_filename);

        std::fs::rename(old_path, &new_path)?;
        self.generate_variants(&new_filename)?;
        self.delete_variants(old_filename)?;
        Ok(new_filename) // Return just the filename, not the full path
    }

    pub fn regenerate_variants(&self, filename: &str) -> std::io::Result<()> {
        self.delete_variants(filename)?;
        self.generate_variants(filename)
    }

    pub fn regenerate_missing_variants(&self, filename: &str) -> std::io::Result<bool> {
        if self.has_all_variants(filename) {
            return Ok(false);
        }

        self.generate_variants(filename)?;
        Ok(true)
    }

    fn has_all_variants(&self, filename: &str) -> bool {
        VARIANT_WIDTHS.iter().all(|width| {
            self.base_path
                .join(variant_filename(filename, *width))
                .exists()
        })
    }

    fn generate_variants(&self, filename: &str) -> std::io::Result<()> {
        let source_path = self.base_path.join(filename);
        let image = image::open(&source_path).map_err(to_io_error)?;
        let (width, height) = image.dimensions();
        for variant_width in VARIANT_WIDTHS {
            let variant_path = self
                .base_path
                .join(variant_filename(filename, variant_width));
            let variant = if width > variant_width {
                let variant_height = ((height as u64 * variant_width as u64) / width as u64)
                    .max(1)
                    .min(u32::MAX as u64) as u32;
                image.resize_exact(
                    variant_width,
                    variant_height,
                    image::imageops::FilterType::Lanczos3,
                )
            } else {
                image.clone()
            };
            let mut output = File::create(&variant_path)?;
            let rgb = variant.to_rgb8();
            JpegEncoder::new_with_quality(&mut output, 78)
                .encode_image(&rgb)
                .map_err(to_io_error)?;
        }
        Ok(())
    }

    fn delete_variants(&self, filename: &str) -> std::io::Result<()> {
        for width in VARIANT_WIDTHS {
            let variant_path = self.base_path.join(variant_filename(filename, width));
            if variant_path.exists() {
                std::fs::remove_file(variant_path)?;
            }
        }
        Ok(())
    }
}

fn variant_filename(filename: &str, width: u32) -> String {
    let path = Path::new(filename);
    let stem = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or(filename);
    format!("{}-{}.jpg", stem, width)
}

fn to_io_error(error: image::ImageError) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, error)
}

#[cfg(test)]
mod tests {
    use super::ImageFileManager;
    use image::{Rgb, RgbImage};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn save_file_generates_responsive_variants() {
        let dir = std::env::temp_dir().join(format!(
            "craftcms-images-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();

        let mut source = RgbImage::new(1800, 1200);
        for pixel in source.pixels_mut() {
            *pixel = Rgb([120, 80, 40]);
        }
        let source_path = dir.join("source.jpg");
        source.save(&source_path).unwrap();
        let bytes = std::fs::read(source_path).unwrap();

        let manager = ImageFileManager::new(&dir);
        let filename = manager
            .save_file(&bytes, "test-image", &mime::IMAGE_JPEG)
            .unwrap();

        assert_eq!(filename, "test-image.jpg");
        for width in [480, 960, 1600] {
            assert!(dir.join(format!("test-image-{width}.jpg")).exists());
        }

        std::fs::remove_dir_all(dir).unwrap();
    }
}
