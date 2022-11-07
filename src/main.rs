use std::{
    collections::HashMap,
    env,
    sync::{Arc, Mutex},
};

use actix_identity::{Identity, IdentityMiddleware};
use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::{
    cookie::Key,
    get,
    http::header::ContentType,
    middleware::Logger,
    web::{scope, Data},
    App, HttpResponse, HttpServer,
};
use dotenv::dotenv;
use oauth2::{basic::BasicClient, AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl};

mod db;
use db::PrismaClient;

mod routes;
use routes::{
    auth::{login, login_callback, logout},
    guilds::{
        get_guild, get_guild_config, get_guild_config_history, get_guild_infractions,
        get_guild_stats_messages, post_guild_config,
    },
    users::{get_user, get_user_guilds, get_user_self},
};

pub struct AppData {
    prisma: Arc<PrismaClient>, // PrismaClient doesn't implement Clone, so we can't use it without wrapping it
    oauth: BasicClient,
}

#[get("/")]
async fn index(identity: Option<Identity>, data: Data<AppData>) -> HttpResponse {
    let response = format!("Welcome to Speedboat API {}, powered by Actix!", "v0.1");

    if let Some(identity) = identity {
        let user: db::users::Data = data
            .prisma
            .users()
            .find_first(vec![db::users::user_id::equals(identity.id().unwrap())])
            .exec()
            .await
            .unwrap()
            .unwrap();

        HttpResponse::Ok()
            .content_type(ContentType::plaintext())
            .body(format!("🚀 Hello {}! {}", user.username, response))
    } else {
        HttpResponse::Ok()
            .content_type(ContentType::plaintext())
            .body(format!("🚀 {}", response))
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let prisma_client: Arc<PrismaClient> = Arc::new(db::new_client().await.unwrap());
    let oauth_client = BasicClient::new(
        ClientId::new(env::var("DISCORD_CLIENT_ID").expect("Missing client id")),
        Some(ClientSecret::new(
            env::var("DISCORD_CLIENT_SECRET").expect("Missing client secret"),
        )),
        AuthUrl::new(env::var("DISCORD_AUTH_URL").expect("Missing auth url")).unwrap(),
        Some(TokenUrl::new(env::var("DISCORD_TOKEN_URL").expect("Missing token url")).unwrap()),
    )
    .set_redirect_uri(
        RedirectUrl::new("http://localhost:3000/auth/login/callback".to_string())
            .expect("Invalid redirect URL"),
    );

    let code_verifiers: HashMap<String, String> = HashMap::new();

    HttpServer::new(move || {
        let session_mw = SessionMiddleware::builder(CookieSessionStore::default(), Key::generate())
            .cookie_secure(false) // Disable secure cookie for local testing
            .build();

        App::new()
            .wrap(IdentityMiddleware::default()) // This is wrapped last, this order is reversed... obviously
            .wrap(session_mw) // This is wrapped second, it needs to be available for the IdentityMiddleware
            .wrap(Logger::default()) // This is wrapped 1st
            .service(index)
            .service(scope("/admin").service(index))
            .service(
                scope("/auth")
                    .service(login)
                    .service(login_callback)
                    .service(logout),
            )
            .service(
                scope("/guilds")
                    .service(get_guild)
                    .service(get_guild_config)
                    .service(post_guild_config)
                    .service(get_guild_config_history)
                    .service(get_guild_infractions)
                    .service(get_guild_stats_messages),
            )
            .service(
                scope("/users")
                    .service(get_user_self)
                    .service(get_user)
                    .service(get_user_guilds),
            )
            .app_data(Data::new(AppData {
                prisma: prisma_client.clone(),
                oauth: oauth_client.clone(),
            }))
            .app_data(Data::new(Mutex::new(code_verifiers.clone())))
    })
    .bind(("127.0.0.1", 3000))?
    .run()
    .await
}
