use actix_web::{
    get,
    web::{self, Json},
    Responder, Result,
};

use crate::{
    db::{self},
    AppData,
};

#[get("/a/{name}")]
pub async fn index(data: web::Data<AppData>) -> Result<impl Responder> {
    let user = data
        .prisma
        .users()
        .find_first(vec![db::users::user_id::equals(
            "194861788926443520".to_string(),
        )])
        .exec()
        .await
        .unwrap();

    Ok(Json(user))
}
