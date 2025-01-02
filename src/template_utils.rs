use crate::models::CustomError;
use tera::Context;
use warp::Reply;

pub async fn render_template(
    template_name: &str,
    context: &Context,
) -> Result<impl Reply, warp::Rejection> {
    crate::templates::TEMPLATES
        .render(template_name, context)
        .map(warp::reply::html)
        .map_err(|e| {
            eprintln!("Template rendering error: {:?}", e);
            warp::reject::custom(CustomError {
                message: "Failed to render page".to_string(),
            })
        })
}
