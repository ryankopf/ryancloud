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
        "model": "gpt-5-nano",
        "messages": [
            {"role": "system", "content": "You are an API that extracts descriptive tags and a short summary for images via JSON."},
            {"role": "user", "content": format!("Generate tags and a short description for this image: {}", image_url)}
        ],
        "tools": [
            {
                "type": "function",
                "function": {
                    "name": "tag_image",
                    "description": "Extract tags and a short description for an image.",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "tags": {
                                "type": "array",
                                "items": {"type": "string"},
                                "description": "Relevant tags/keywords describing the image (e.g., people, luggage, blue hair, Narita Express, train station)."
                            },
                            "description": {
                                "type": "string",
                                "description": "One or two sentences describing the image contents."
                            }
                        },
                        "required": ["tags", "description"]
                    }
                }
            }
        ],
        "tool_choice": {"type": "function", "function": {"name": "tag_image"}},
        "response_format": {"type": "json_object"}
    });

    let json = make_request(&api_key, body).await?;

    if let Some(arg_str) = json["choices"][0]["message"]["tool_calls"][0]["function"]["arguments"].as_str() {
        match serde_json::from_str::<ImageTags>(arg_str) {
            Ok(parsed) => Ok(parsed),
            Err(e) => Err(format!("Failed to parse arguments as ImageTags: {}\n{}", e, arg_str)),
        }
    } else {
        Err(format!(
            "Could not find expected 'arguments' in response.\n{}",
            serde_json::to_string_pretty(&json).unwrap_or_default()
        ))
    }
}
