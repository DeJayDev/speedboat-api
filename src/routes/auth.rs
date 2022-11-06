use crate::AppData;
use actix_identity::Identity;
use actix_session::SessionExt;
use actix_web::{get, http::header::LOCATION, web::Data, HttpRequest, HttpResponse, Responder};
use actix_web::{web, HttpMessage};
use oauth2::reqwest::async_http_client;
use oauth2::{
    AuthorizationCode, CsrfToken, PkceCodeChallenge, PkceCodeVerifier, Scope, TokenResponse,
};
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::sync::Mutex;
use url::Url;

#[derive(Deserialize)]
pub struct AuthRequest {
    code: String,
    state: String,
}

#[derive(Deserialize)]
pub struct DiscordUser {
    pub id: String,
}

#[get("/login")]
pub async fn login(
    app_data: Data<AppData>,
    code_verifiers: Data<Mutex<HashMap<String, String>>>,
) -> HttpResponse {
    // Create a PKCE code verifier and SHA-256 encode it as a code challenge.
    let (pkce_code_challenge, pkce_code_verifier) = PkceCodeChallenge::new_random_sha256();
    // Generate the authorization URL to which we'll redirect the user.
    let (auth_url, csrf_token) = &app_data
        .oauth
        .authorize_url(CsrfToken::new_random)
        // Set the desired scopes.
        .add_scope(Scope::new("identify".to_string()))
        // Set the PKCE code challenge.
        .set_pkce_challenge(pkce_code_challenge)
        .url();

    let mut code_v = code_verifiers.lock().unwrap();
    let csrf = csrf_token.secret().to_string();
    let verifier = pkce_code_verifier.secret().to_string();

    code_v.insert(csrf, verifier);

    HttpResponse::Found()
        .append_header((LOCATION, auth_url.to_string()))
        .finish()
}

#[get("/login/callback")]
pub async fn login_callback(
    req: HttpRequest,
    data: Data<AppData>,
    params: web::Query<AuthRequest>,
    code_verifiers: Data<Mutex<HashMap<String, String>>>,
) -> HttpResponse {
    let code = AuthorizationCode::new(params.code.clone());
    let state = CsrfToken::new(params.state.clone());

    let cv = code_verifiers.lock().unwrap();
    let verifier = cv.get(state.secret());
    if verifier.is_none() {
        return HttpResponse::BadRequest().body("Could not find CSRF token, try again?");
    }
    let verifier = PkceCodeVerifier::new(verifier.unwrap().to_string());

    // Exchange the code with a token.
    let token = &data
        .oauth
        .exchange_code(code)
        .set_pkce_verifier(verifier)
        .request_async(async_http_client)
        .await
        .expect("exchange_code failed");
    let url = Url::parse(
        format!(
            "{}/users/@me",
            env::var("DISCORD_API_URL")
                .expect("Discord base url not set")
                .as_str(),
        )
        .as_str(),
    );

    let client = reqwest::Client::new();
    let res = client
        .get(url.unwrap())
        .header(
            "Authorization",
            format!("Bearer {}", token.access_token().secret()),
        )
        .send()
        .await
        .unwrap();

    let user: DiscordUser = res.json::<DiscordUser>().await.unwrap();

    Identity::login(&req.extensions(), user.id).unwrap();

    HttpResponse::Found()
        .append_header((LOCATION, "/".to_string()))
        .finish()
}

#[get("/logout")]
pub async fn logout(user: Identity) -> impl Responder {
    user.logout();
    HttpResponse::Found()
        .append_header((LOCATION, "/".to_string()))
        .finish()
}
