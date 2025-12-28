use expo_push_notification_client::{Expo, ExpoClientOptions, ExpoPushMessage, RichContent};
use lambda_http::{Body, Error, Request, Response};
use serde_json::{json, Value};
use supabase_rs::SupabaseClient;
use std::env;
use http::header::HeaderValue;
use http::StatusCode;
use dotenv::dotenv;

#[derive(Debug)]
enum SupabaseError {
    Initialization,
    FetchTokens,
}

impl From<SupabaseError> for Error {
    fn from(error: SupabaseError) -> Self {
        match error {
            SupabaseError::Initialization => Error::from("Failed to initialize SupabaseClient"),
            SupabaseError::FetchTokens => Error::from("Failed to fetch tokens from Supabase"),
        }
    }
}

async fn initialize_supabase_client() -> Result<SupabaseClient, Error> {
    dotenv().ok();

    let supabase_url = env::var("SUPABASE_URL").map_err(|e| {
        eprintln!("Error loading SUPABASE_URL: {:?}", e);
        SupabaseError::Initialization
    })?;
    let supabase_key = env::var("SUPABASE_KEY").map_err(|e| {
        eprintln!("Error loading SUPABASE_KEY: {:?}", e);
        SupabaseError::Initialization
    })?;

    let client = SupabaseClient::new(supabase_url, supabase_key).map_err(|e| {
        eprintln!("Error initializing SupabaseClient: {:?}", e);
        SupabaseError::Initialization
    })?;
    Ok(client)
}

async fn fetch_expo_push_tokens(client: &SupabaseClient) -> Result<Vec<String>, Error> {
    let response = client.select("users").execute().await.map_err(|e| {
        eprintln!("Error fetching expo push tokens: {:?}", e);
        SupabaseError::FetchTokens
    })?;

    let tokens = response
        .iter()
        .filter_map(|row| row["expo_push_token"].as_str().map(|s| s.to_string()))
        .collect::<Vec<String>>();
    println!("fetched expo push tokens from supabase {:?}", tokens);
    Ok(tokens)
}

async fn extract_body(req: &Request) -> Result<Value, Error> {
    let body_str = match req.body() {
        Body::Text(s) => s.to_string(),
        Body::Binary(b) => String::from_utf8(b.to_vec()).map_err(|e| {
            eprintln!("Error converting body to string: {:?}", e);
            Error::from(e)
        })?,
        _ => {
            eprintln!("Unsupported body type");
            return Err(Error::from("Unsupported body type"));
        }
    };

    let json_body: Value = serde_json::from_str(&body_str).map_err(|e| {
        eprintln!("Error parsing JSON body: {:?}", e);
        Error::from(e)
    })?;
    Ok(json_body)
}

pub(crate) async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
    let expected_key = env::var("API_KEY").expect("API_KEY not set");
    let expected_key_value = HeaderValue::from_str(&expected_key)
        .map_err(|_| Error::from("Invalid API_KEY environment variable"))?;

    let client_key = event.headers().get("x-api-key");

    if client_key != Some(&expected_key_value) {
        return Ok(Response::builder()
            .status(StatusCode::FORBIDDEN)
            .body("Forbidden: Invalid API Key".into())?);
    }

    println!(
        "This is an Expo push notification API ver: {}",
        env!("CARGO_PKG_VERSION"),
    );
    println!("Request Headers: {:?}", event.headers());

    let expo = Expo::new(ExpoClientOptions {
        access_token: Some(env::var("EXPO_ACCESS_TOKEN").expect("EXPO_ACCESS_TOKEN to be set")),
    });

    let mut title = "25日だよ".to_string();
    let mut body = "パートナーに請求しよう".to_string();
    let mut expo_push_tokens = vec![];

    match event.method().as_str() {
        "GET" => {
            let supabase_client = initialize_supabase_client().await?;
            expo_push_tokens = fetch_expo_push_tokens(&supabase_client).await?;
        }
        "POST" => {
            let json_body = extract_body(&event).await?;

            if let Some(t) = json_body["title"].as_str() {
                title = t.to_string();
            } else {
                eprintln!("Title is required");
                return Ok(Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .header("Content-Type", "application/json")
                    .body(
                        json!({
                            "error": "Title is required"
                        })
                        .to_string()
                        .into(),
                    )?);
            }

            if let Some(b) = json_body["body"].as_str() {
                body = b.to_string();
            } else {
                eprintln!("Body is required");
                return Ok(Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .header("Content-Type", "application/json")
                    .body(
                        json!({
                            "error": "Body is required"
                        })
                        .to_string()
                        .into(),
                    )?);
            }

            if let Some(token) = json_body["expo_push_token"].as_str() {
                if Expo::is_expo_push_token(token) {
                    expo_push_tokens.push(token.to_string());
                } else {
                    return Ok(Response::builder()
                        .status(StatusCode::BAD_REQUEST)
                        .header("Content-Type", "application/json")
                        .body(
                            json!({
                                "error": "Invalid expo push token"
                            })
                            .to_string()
                            .into(),
                        )?);
                }
            } else {
                eprintln!("expo_push_token is required for POST requests");
                 return Ok(Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .header("Content-Type", "application/json")
                    .body(
                        json!({
                            "error": "expo_push_token is required"
                        })
                        .to_string()
                        .into(),
                    )?);
            }
            println!("Title: {}", title);
            println!("Body: {}", body);
            println!("expo_push_tokens: {:?}", expo_push_tokens);
        }
        _ => {
            return Ok(Response::builder()
                .status(StatusCode::METHOD_NOT_ALLOWED)
                .header("Content-Type", "application/json")
                .body(
                    json!({
                        "error": "Method not allowed"
                    })
                    .to_string()
                    .into(),
                )?);
        }
    }

    println!("Building push notification");
    let expo_push_message = ExpoPushMessage::builder(expo_push_tokens)
        .title(title)
        .body(body)
        .rich_content(RichContent {
            image: Some("https://picsum.photos/200/300".to_string()),
        })
        .build()
        .map_err(|e: expo_push_notification_client::ValidationError| {
            eprintln!("Error building ExpoPushMessage: {:?}", e);
            Error::from(e)
        })?;

    println!("Sending push notification");
    match expo.send_push_notifications(expo_push_message).await {
        Ok(_) => Ok(Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .body(
                json!({
                    "message": "Push notification sent successfully"
                })
                .to_string()
                .into(),
            )?),
        Err(e) => {
            eprintln!("Failed to send push notification: {:?}", e);
            Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .header("Content-Type", "application/json")
                .body(
                    json!({
                        "error": "Failed to send push notification"
                    })
                    .to_string()
                    .into(),
                )?)
        }
    }
}
