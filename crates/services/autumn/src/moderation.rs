use reqwest::Client;
use revolt_database::ImageModeration;

lazy_static::lazy_static! {
    static ref HTTP_CLIENT: Client = Client::builder()
        .timeout(std::time::Duration::from_secs(8))
        .build()
        .expect("failed to build http client");
}

#[derive(serde::Deserialize)]
struct ModerationResponse {
    nsfw: Option<bool>,
    nsfw_label: Option<String>,
    nsfw_score: Option<f32>,
    profanity: Option<bool>,
    profanity_matches: Option<Vec<String>>,
}

pub async fn scan_image(
    bytes: &[u8],
    content_type: &str,
) -> Option<ImageModeration> {
    let url = std::env::var("BLINQ_IMAGE_MODERATION_URL").ok()?;

    let response = HTTP_CLIENT
        .post(url)
        .header("content-type", content_type)
        .body(bytes.to_vec())
        .send()
        .await;

    let response = match response {
        Ok(value) => value,
        Err(err) => {
            tracing::warn!("Image moderation request failed: {err}");
            return None;
        }
    };

    if !response.status().is_success() {
        tracing::warn!(
            "Image moderation request returned status {}",
            response.status()
        );
        return None;
    }

    let payload = match response.json::<ModerationResponse>().await {
        Ok(value) => value,
        Err(err) => {
            tracing::warn!("Failed to parse moderation response: {err}");
            return None;
        }
    };

    Some(ImageModeration {
        nsfw: payload.nsfw,
        nsfw_label: payload.nsfw_label,
        nsfw_score: payload.nsfw_score,
        profanity: payload.profanity,
        profanity_matches: payload.profanity_matches,
    })
}
