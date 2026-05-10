use serde::{Deserialize, Serialize, Serializer};

pub struct User {
    pub id: i64,
    pub email: String,
    pub password_hash: String,
    pub access_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Image {
    pub alt: String,
    pub description: String,
    pub slug: String,
    pub keywords: Vec<String>,
    pub filename: String,
    pub status: String,
    pub deleted_at: Option<String>,
    pub source: String,
    pub source_media_id: Option<String>,
    pub source_permalink: Option<String>,
    pub source_timestamp: Option<String>,
}

impl Serialize for Image {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        struct ImageView<'a> {
            alt: &'a str,
            description: &'a str,
            slug: &'a str,
            keywords: &'a [String],
            filename: &'a str,
            status: &'a str,
            deleted_at: &'a Option<String>,
            source: &'a str,
            source_media_id: &'a Option<String>,
            source_permalink: &'a Option<String>,
            source_timestamp: &'a Option<String>,
            image_url: String,
            srcset: String,
            variant_480_url: String,
            variant_960_url: String,
            variant_1600_url: String,
        }

        ImageView {
            alt: &self.alt,
            description: &self.description,
            slug: &self.slug,
            keywords: &self.keywords,
            filename: &self.filename,
            status: &self.status,
            deleted_at: &self.deleted_at,
            source: &self.source,
            source_media_id: &self.source_media_id,
            source_permalink: &self.source_permalink,
            source_timestamp: &self.source_timestamp,
            image_url: self.image_url(),
            srcset: self.srcset(),
            variant_480_url: self.variant_url(480),
            variant_960_url: self.variant_url(960),
            variant_1600_url: self.variant_url(1600),
        }
        .serialize(serializer)
    }
}

impl Image {
    pub fn image_url(&self) -> String {
        format!("/images/{}", self.filename)
    }

    pub fn variant_filename(&self, width: u32) -> String {
        let stem = std::path::Path::new(&self.filename)
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or(&self.slug);
        format!("{}-{}.jpg", stem, width)
    }

    pub fn variant_url(&self, width: u32) -> String {
        format!("/images/{}", self.variant_filename(width))
    }

    pub fn srcset(&self) -> String {
        [480, 960, 1600]
            .iter()
            .map(|width| format!("{} {}w", self.variant_url(*width), width))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

#[derive(Deserialize)]
pub struct ImageForm {
    pub alt: String,
    pub description: String,
    pub slug: String,
    pub keywords: String,
    pub image_data: Vec<u8>,
}

#[derive(Debug, Deserialize)]
pub struct AdminImageQuery {
    pub filter: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ImageStatusForm {
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct AdminSettingsForm {
    pub default_import_status: String,
    pub instagram_access_token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppSettings {
    pub default_import_status: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            default_import_status: "draft".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InstagramConnection {
    pub instagram_user_id: String,
    pub username: String,
    pub access_token: String,
    pub token_expires_at: Option<String>,
    pub connected_at: String,
    pub last_sync_at: Option<String>,
}

#[derive(Deserialize)]
pub struct ImageInput {
    #[serde(alias = "url", alias = "Url")]
    #[serde(rename = "URL")]
    pub url: String,
    #[serde(alias = "alt", alias = "Alt")]
    pub alt: String,
    #[serde(alias = "description", alias = "Description")]
    pub description: String,
    #[serde(alias = "slug", alias = "Slug")]
    pub slug: String,
    #[serde(alias = "keywords", alias = "Keywords")]
    pub keywords: Vec<String>,
    #[serde(alias = "type", alias = "Type")]
    pub image_type: String,
}

use std::fmt;
use warp::reject::Reject;

#[derive(Debug)]
pub struct CustomError {
    pub message: String,
}

impl CustomError {
    pub fn new(message: String) -> CustomError {
        CustomError { message }
    }
}

impl fmt::Display for CustomError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for CustomError {}

impl Reject for CustomError {}

pub fn is_valid_slug(slug: &str) -> bool {
    !slug.is_empty()
        && slug.len() <= 160
        && slug
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        && !slug.starts_with('-')
        && !slug.ends_with('-')
        && !slug.contains("--")
}

#[cfg(test)]
mod tests {
    use super::is_valid_slug;

    #[test]
    fn slug_validation_accepts_safe_slugs() {
        assert!(is_valid_slug("muddy-venture-piece-01"));
        assert!(is_valid_slug("abc123"));
    }

    #[test]
    fn slug_validation_rejects_path_and_ambiguous_slugs() {
        for slug in [
            "",
            "../escape",
            "has/slash",
            "has space",
            "HasUppercase",
            "-leading",
            "trailing-",
            "double--dash",
            "emoji-💌",
        ] {
            assert!(!is_valid_slug(slug), "{slug} should be rejected");
        }
    }
}

#[derive(Deserialize)]
pub struct LoginCredentials {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub access_token: String,
}

#[derive(Debug)]
pub struct RedirectToLogin;
impl Reject for RedirectToLogin {}
