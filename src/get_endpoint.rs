use actix_web::{get, web, HttpResponse, Responder};
use serde::Serialize;
use tracing::error;
use crate::state::State;
use crate::token_to_account::token_to_account;

#[derive(Serialize)]
struct Response {
    pub id: String,
    pub first_name: Option<String>,
    pub discord_id: Option<String>,
    pub birthday: Option<i64>,
}

#[get("/api/{token}/account")]
pub async fn get_endpoint(ctx: web::Data<State>, token: web::Path<String>) -> impl Responder {

    let account = match token_to_account(ctx.nc.clone(), &token).await {
        Ok(account) => account,
        Err(e) => {
            error!("Error looking up account:  {}", e);
            return HttpResponse::InternalServerError().body("Internal Server Error");
        },
    };

    if account.is_none() {
        return HttpResponse::NotFound().body("Account not found");
    }
    let account = account.unwrap();

    HttpResponse::Ok().json(Response {
        id: account.id,
        discord_id: account.discord_id,
        first_name: account.first_name,
        birthday: account.birthday,
    })
}