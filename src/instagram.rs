use crate::models::{CustomError, Image};
use chrono::{Duration, Utc};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct InstagramMeResponse {
    id: Option<String>,
    user_id: Option<String>,
    username: Option<String>,
}

#[derive(Debug)]
pub struct InstagramIdentity {
    pub user_id: String,
    pub username: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MediaChildren {
    data: Vec<InstagramMedia>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct InstagramMedia {
    pub id: String,
    pub caption: Option<String>,
    pub media_type: String,
    pub media_url: Option<String>,
    pub permalink: Option<String>,
    pub timestamp: Option<String>,
    pub children: Option<MediaChildren>,
}

#[derive(Debug, Deserialize)]
struct MediaResponse {
    data: Vec<InstagramMedia>,
}

#[derive(Debug, Deserialize)]
struct RefreshTokenResponse {
    access_token: String,
    expires_in: i64,
}

#[derive(Debug)]
pub struct FlattenedInstagramMedia {
    pub source_media_id: String,
    pub media_url: String,
    pub caption: String,
    pub permalink: Option<String>,
    pub timestamp: Option<String>,
}

#[derive(Debug)]
pub struct RefreshedInstagramToken {
    pub access_token: String,
    pub token_expires_at: String,
}

pub async fn fetch_identity(access_token: &str) -> Result<InstagramIdentity, CustomError> {
    let client = reqwest::Client::new();
    let me = client
        .get("https://graph.instagram.com/me")
        .query(&[
            ("fields", "id,user_id,username"),
            ("access_token", access_token),
        ])
        .send()
        .await
        .map_err(|e| CustomError::new(format!("Failed to fetch Instagram account details: {e}")))?;

    if !me.status().is_success() {
        let body = me.text().await.unwrap_or_default();
        return Err(CustomError::new(format!(
            "Instagram account lookup failed: {body}"
        )));
    }

    let me: InstagramMeResponse = me
        .json()
        .await
        .map_err(|e| CustomError::new(format!("Invalid Instagram account response: {e}")))?;

    Ok(InstagramIdentity {
        user_id: me
            .user_id
            .or(me.id)
            .unwrap_or_else(|| "unknown".to_string()),
        username: me.username.unwrap_or_else(|| "instagram".to_string()),
    })
}

pub async fn fetch_recent_media(access_token: &str) -> Result<Vec<InstagramMedia>, CustomError> {
    let client = reqwest::Client::new();
    let response = client
        .get("https://graph.instagram.com/me/media")
        .query(&[
            (
                "fields",
                "id,caption,media_type,media_url,permalink,timestamp,children{id,media_type,media_url,permalink,timestamp}",
            ),
            ("limit", "50"),
            ("access_token", access_token),
        ])
        .send()
        .await
        .map_err(|e| CustomError::new(format!("Failed to fetch Instagram media: {e}")))?;

    if !response.status().is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(CustomError::new(format!(
            "Instagram media request failed: {body}"
        )));
    }

    let media: MediaResponse = response
        .json()
        .await
        .map_err(|e| CustomError::new(format!("Invalid Instagram media response: {e}")))?;

    Ok(media.data)
}

pub async fn refresh_access_token(
    access_token: &str,
) -> Result<RefreshedInstagramToken, CustomError> {
    let client = reqwest::Client::new();
    let response = client
        .get("https://graph.instagram.com/refresh_access_token")
        .query(&[
            ("grant_type", "ig_refresh_token"),
            ("access_token", access_token),
        ])
        .send()
        .await
        .map_err(|e| CustomError::new(format!("Failed to refresh Instagram token: {e}")))?;

    if !response.status().is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(CustomError::new(format!(
            "Instagram token refresh failed: {body}"
        )));
    }

    let refreshed: RefreshTokenResponse = response
        .json()
        .await
        .map_err(|e| CustomError::new(format!("Invalid Instagram token refresh response: {e}")))?;

    let expires_at = (Utc::now() + Duration::seconds(refreshed.expires_in))
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();

    Ok(RefreshedInstagramToken {
        access_token: refreshed.access_token,
        token_expires_at: expires_at,
    })
}

pub fn flatten_media(items: &[InstagramMedia]) -> Vec<FlattenedInstagramMedia> {
    let mut flattened = Vec::new();
    for item in items {
        match item.media_type.as_str() {
            "IMAGE" => {
                if let Some(media_url) = &item.media_url {
                    flattened.push(FlattenedInstagramMedia {
                        source_media_id: item.id.clone(),
                        media_url: media_url.clone(),
                        caption: item.caption.clone().unwrap_or_default(),
                        permalink: item.permalink.clone(),
                        timestamp: item.timestamp.clone(),
                    });
                }
            }
            "CAROUSEL_ALBUM" => {
                if let Some(children) = &item.children {
                    for child in &children.data {
                        if child.media_type == "IMAGE" {
                            if let Some(media_url) = &child.media_url {
                                flattened.push(FlattenedInstagramMedia {
                                    source_media_id: child.id.clone(),
                                    media_url: media_url.clone(),
                                    caption: item.caption.clone().unwrap_or_default(),
                                    permalink: item.permalink.clone().or(child.permalink.clone()),
                                    timestamp: item.timestamp.clone().or(child.timestamp.clone()),
                                });
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
    flattened
}

pub async fn download_media(
    media_url: &str,
) -> Result<(Vec<u8>, mime::Mime), CustomError> {
    let response = reqwest::get(media_url)
        .await
        .map_err(|e| CustomError::new(format!("Failed to download Instagram media: {e}")))?;

    if !response.status().is_success() {
        return Err(CustomError::new(format!(
            "Instagram media download failed with status {}",
            response.status()
        )));
    }

    let mime_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<mime::Mime>().ok())
        .unwrap_or_else(|| mime_guess::from_path(media_url).first_or_octet_stream());

    let bytes = response
        .bytes()
        .await
        .map_err(|e| CustomError::new(format!("Failed to read Instagram media bytes: {e}")))?;

    Ok((bytes.to_vec(), mime_type))
}

pub fn slugify(input: &str) -> String {
    let mut slug = String::new();
    let mut previous_dash = false;

    for ch in input.chars().flat_map(|c| c.to_lowercase()) {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch);
            previous_dash = false;
        } else if !previous_dash {
            slug.push('-');
            previous_dash = true;
        }
    }

    slug.trim_matches('-').to_string()
}

pub fn image_from_instagram_media(
    item: &FlattenedInstagramMedia,
    default_status: &str,
    slug: String,
) -> Image {
    let title = item
        .caption
        .lines()
        .find(|line| !line.trim().is_empty())
        .unwrap_or("Instagram import")
        .trim()
        .chars()
        .take(120)
        .collect::<String>();

    Image {
        alt: title,
        description: item.caption.trim().to_string(),
        slug,
        keywords: Vec::new(),
        filename: String::new(),
        status: default_status.to_string(),
        deleted_at: None,
        source: "instagram".to_string(),
        source_media_id: Some(item.source_media_id.clone()),
        source_permalink: item.permalink.clone(),
        source_timestamp: item.timestamp.clone(),
    }
}
