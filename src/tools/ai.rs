use serde::{Deserialize, Serialize};
use reqwest::Client;

#[derive(Debug, Serialize, Deserialize)]
pub struct ImageTags {
    pub tags: Vec<String>,
    pub description: String,
}

/// Generic OpenAI request executor
async fn make_request(
    api_key: &str,
    body: serde_json::Value,
) -> Result<serde_json::Value, String> {
    let client = Client::new();
    let body_str = serde_json::to_string(&body).unwrap();

    let res = client
        .post("https://api.openai.com/v1/chat/completions")
        .bearer_auth(api_key)
        .header("Content-Type", "application/json")
        .body(body_str)
        .send()
        .await;

    match res {
        Ok(response) => {
            let text = response.text().await.unwrap_or_default();
            match serde_json::from_str::<serde_json::Value>(&text) {
                Ok(json) => Ok(json),
                Err(e) => Err(format!("Failed to parse OpenAI response as JSON: {}\n{}", e, text)),
            }
        }
        Err(e) => Err(format!("Request to OpenAI failed: {}", e)),
    }
}

/// Generate tags + description for an image
pub async fn tag_image(image_url: &str) -> Result<ImageTags, String> {
    let api_key = std::env::var("OPENAI_API_KEY").unwrap_or_default();

    let body = serde_json::json!({
        "model": "gpt-4.1-mini", // supports vision
        "messages": [
            {
                "role": "system",
                "content": "You are an API that extracts descriptive tags and a short summary for images. This is so that the images can be found by relevant topic when creating travel vlog content. Do not include 'everyday' tags (man, woman, standing) unless they are clearly the focus or would be an interesting topic for a travel video. Respond in pure JSON only."
            },
            {
                "role": "user",
                "content": [
                    { "type": "text", "text": "Generate tags and a short description for this image." },
                    { "type": "image_url", "image_url": { "url": image_url } }
                ]
            }
        ],
        "response_format": { "type": "json_object" }
    });

    let json = make_request(&api_key, body).await?;

    if let Some(content) = json["choices"][0]["message"]["content"].as_str() {
        match serde_json::from_str::<ImageTags>(content) {
            Ok(parsed) => Ok(parsed),
            Err(e) => Err(format!("Failed to parse model JSON as ImageTags: {}\n{}", e, content)),
        }
    } else {
        Err(format!(
            "Could not find expected 'content' in response.\n{}",
            serde_json::to_string_pretty(&json).unwrap_or_default()
        ))
    }
}
