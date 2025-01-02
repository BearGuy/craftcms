use crate::admin_assets::AdminAssets;
use lazy_static::lazy_static;
use tera::Tera;

lazy_static! {
    pub static ref TEMPLATES: Tera = {
        let mut tera = Tera::new("templates/**/*").unwrap();

        for file_path in AdminAssets::iter() {
            if file_path.ends_with(".html") {
                if let Some(content) = AdminAssets::get(&file_path) {
                    if let Ok(template_str) = String::from_utf8(content.data.to_vec()) {
                        // Add with admin/ prefix to avoid naming conflicts
                        tera.add_raw_template(&format!("admin/{}", file_path), &template_str)
                            .unwrap_or_else(|e| eprintln!("Failed to add template {}: {}", file_path, e));
                    }
                }
            }
        }

        tera
    };
}
