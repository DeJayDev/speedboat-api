use actix_web::{
    get, post,
    web::{self, Data, Json},
    Responder, Result,
};

use crate::{db::guilds, AppData};

#[get("/<gid>")]
pub async fn get_guild(data: Data<AppData>, path: web::Path<(i64,)>) -> Result<impl Responder> {
    let guild = data
        .prisma
        .guilds()
        .find_first(vec![guilds::guild_id::equals(path.0)])
        .exec()
        .await
        .unwrap();

    Ok(Json(guild))
}

#[get("/<gid>/config")]
pub async fn get_guild_config(_data: Data<AppData>, path: web::Path<(i64,)>) -> String {
    format!("Want config for guild {}", path.0.to_string())
}

#[post("/<gid>/config")]
pub async fn post_guild_config(_data: Data<AppData>, path: web::Path<(i64,)>) -> String {
    format!("Want to update config for guild {}", path.0.to_string())
}

#[get("/<gid>/config/history")]
pub async fn get_guild_config_history(_data: Data<AppData>, path: web::Path<(i64,)>) -> String {
    format!("Want config history for guild {}", path.0.to_string())
}

#[get("/<gid>/infractions")]
pub async fn get_guild_infractions(_data: Data<AppData>, path: web::Path<(i64,)>) -> String {
    format!("Want infractions for guild {}", path.0.to_string())
}

#[get("/<gid>/stats/messages")]
pub async fn get_guild_stats_messages(_data: Data<AppData>, path: web::Path<(i64,)>) -> String {
    format!("Want message stats for guild {}", path.0.to_string())
}
