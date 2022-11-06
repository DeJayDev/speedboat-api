use actix_web::{
    get,
    http::header::ContentType,
    web::{self, Json},
    HttpResponse, Responder, Result,
};

use crate::db::{users, PrismaClient};

#[get("/@me")]
pub async fn get_user_self(client: web::Data<PrismaClient>) -> Result<impl Responder> {
    let user = client
        .users()
        .find_first(vec![users::user_id::equals(
            "194861788926443520".to_string(),
        )])
        .exec()
        .await
        .unwrap();

    Ok(Json(user))
}

#[get("/<uid>")] // TODO: Admin Route
pub async fn get_user(
    client: web::Data<PrismaClient>,
    path: web::Path<(String,)>,
) -> Result<impl Responder> {
    let user = client
        .users()
        .find_first(vec![users::user_id::equals(path.0.to_owned())])
        .exec()
        .await
        .unwrap();

    Ok(Json(user))
}

#[get("/@me/guilds")]
pub async fn get_user_guilds() -> HttpResponse {
    HttpResponse::Ok()
        .content_type(ContentType::plaintext())
        .body("TODO LOL")
}
