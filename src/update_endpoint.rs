use actix_web::{post, web, HttpResponse, Responder};
use async_nats::Client;
use protobuf::{Message, MessageField};
use serde::{Deserialize, Serialize};
use tracing::{error};
use crate::proto::account::Account;
use crate::proto::account_update::{UpdateAccount, UpdateAccountResponse};
use crate::state::State;
use crate::token_to_account::token_to_account;

#[derive(Deserialize)]
struct Request {
    first_name: Option<String>,
    birthday: Option<i64>,
}

#[derive(Serialize)]
struct Response {
    pub id: String,
    pub first_name: Option<String>,
    pub discord_id: Option<String>,
    pub birthday: Option<i64>,
}

#[post("/api/{token}/account")]
pub async fn update_endpoint(ctx: web::Data<State>, token: web::Path<String>,
                             body: web::Json<Request>) -> impl Responder {

    // get account
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
    let mut account = account.unwrap();

    // updated values
    if body.first_name.is_some() {
        account.first_name = body.first_name.clone();
    }
    if body.birthday.is_some() {
        account.birthday = body.birthday.clone();
    }

    // save
    let account = match save_account(ctx.nc.clone(), &account).await {
        Ok(account) => account,
        Err(e) => {
            error!("Error saving account: {}", e);
            return HttpResponse::InternalServerError().body("Internal Server Error");
        }
    };

    // response
    HttpResponse::Ok().json(Response {
        id: account.id,
        discord_id: account.discord_id,
        first_name: account.first_name,
        birthday: account.birthday,
    })
}

async fn save_account(nc: Client, account: &Account) -> anyhow::Result<Account> {

    let mut msg = UpdateAccount::new();
    msg.account = MessageField::some(account.clone());
    let encoded: Vec<u8> = msg.write_to_bytes()?;
    let result = nc.request("accounts.update", encoded.into()).await?;
    let response = UpdateAccountResponse::parse_from_bytes(&result.payload)?;

    if !response.success {
        match response.error {
            None => {
                anyhow::bail!("Error updating account");
            },
            Some(e) => {
                anyhow::bail!(e);
            }
        }
    }

    Ok(response.account.unwrap())
}