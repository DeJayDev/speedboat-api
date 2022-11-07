use actix_identity::Identity;
use actix_web::{
    get,
    web::{self, Data},
    HttpResponse,
};
use serde_json::json;

use crate::{
    db::{guilds, users},
    AppData,
};

#[get("/@me")]
pub async fn get_user_self(identity: Option<Identity>, data: Data<AppData>) -> HttpResponse {
    if let Some(identity) = identity {
        let user = data
            .prisma
            .users()
            .find_first(vec![users::user_id::equals(identity.id().unwrap())])
            .exec()
            .await
            .unwrap()
            .unwrap();

        HttpResponse::Ok().json(user)
    } else {
        HttpResponse::Unauthorized().json(json!({}))
    }
}

#[get("/<uid>")] // TODO: Admin Route
pub async fn get_user(
    identity: Option<Identity>,
    data: Data<AppData>,
    path: web::Path<(i64,)>,
) -> HttpResponse {
    if let Some(identity) = identity {
        let requester = data
            .prisma
            .users()
            .find_first(vec![users::user_id::equals(identity.id().unwrap())])
            .exec()
            .await
            .unwrap()
            .unwrap();

        if requester.admin {
            let user = data
                .prisma
                .users()
                .find_first(vec![users::user_id::equals(path.0.to_owned().to_string())])
                .exec()
                .await
                .unwrap()
                .unwrap();
            HttpResponse::Ok().json(user)
        } else {
            HttpResponse::Unauthorized().json(json!({}))
        }
    } else {
        HttpResponse::Unauthorized().json(json!({}))
    }
}

#[get("/@me/guilds")]
pub async fn get_user_guilds(identity: Option<Identity>, data: Data<AppData>) -> HttpResponse {
    match identity {
        Some(identity) => {
            let _guilds = data.prisma.guilds().find_many(vec![]).exec().await.unwrap();
            let guilds: Vec<guilds::Data> = _guilds
                .into_iter()
                .filter(|guild| {
                    let config = guild.config.as_ref().unwrap();
                    // stop here if the config is empty
                    if config.is_object() {
                        return false;
                    }
                    config["web"][&identity.id().unwrap()].is_null()
                })
                .collect();

            return HttpResponse::Ok().json(guilds);
        }
        _ => {
            return HttpResponse::Unauthorized().json(json!({}));
        }
    }
}
