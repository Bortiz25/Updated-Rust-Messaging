use super::requests_and_responses::{
    CreateChatRequestBody, CreateMessageGCRequestBody, CreateMessageRequestBody, LoginRequestBody,
};
use crate::{
    auth::authorize,
    handlers::{self},
};
use sqlx::PgPool;
use std::sync::Arc;
use warp::{http::header::HeaderMap, reject, Filter, Rejection, Reply};

#[derive(Debug)]
struct JwtError;
impl reject::Reject for JwtError {}

pub fn routes(
    pool: Arc<PgPool>,
) -> impl Filter<Extract = impl Reply, Error = std::convert::Infallible> + Clone {
    get_user(pool.clone())
        .or(login(pool.clone()))
        .or(create_user(pool.clone()))
        .or(create_chat(pool.clone()))
        .or(create_message(pool.clone()))
        .or(get_chats(pool.clone()))
        .or(get_messages(pool.clone()))
        .or(get_user_with_token(pool.clone()))
        .or(create_message_gc(pool.clone()))
        .or(get_chats_gc(pool.clone()))
        .or(get_messages_gc(pool.clone()))
        .recover(handlers::handle_rejection)
}

fn get_user(pool: Arc<PgPool>) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("users" / i32)
        .and(warp::get())
        .and(with_db(pool))
        .and_then(handlers::get_user)
}

fn login(pool: Arc<PgPool>) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("login")
        .and(warp::post())
        .and(warp::body::json::<LoginRequestBody>())
        .and(with_db(pool))
        .and_then(handlers::login)
}

fn create_user(pool: Arc<PgPool>) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("users")
        .and(warp::post())
        .and(warp::body::json::<LoginRequestBody>())
        .and(with_db(pool))
        .and_then(handlers::create_user)
}

fn create_chat(pool: Arc<PgPool>) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("chats")
        .and(warp::post())
        .and(with_auth())
        .and(warp::body::json::<CreateChatRequestBody>())
        .and(with_db(pool))
        .and_then(handlers::create_chat)
}

fn create_message(
    pool: Arc<PgPool>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("chats" / String)
        .and(warp::post())
        .and(with_auth())
        .and(warp::body::json::<CreateMessageRequestBody>())
        .and(with_db(pool))
        .and_then(handlers::create_message)
}

fn get_chats(pool: Arc<PgPool>) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("chats")
        .and(warp::get())
        .and(with_auth())
        .and(with_db(pool))
        .and_then(handlers::get_chats)
}

fn get_messages(pool: Arc<PgPool>) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("chats" / String / "messages")
        .and(warp::get())
        .and(with_auth())
        .and(with_db(pool))
        .and_then(handlers::get_messages)
}

fn get_user_with_token(
    pool: Arc<PgPool>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("users")
        .and(warp::get())
        .and(warp::header::headers_cloned())
        .and(with_db(pool))
        .and_then(handlers::get_user_with_token)
}

fn create_message_gc(
    pool: Arc<PgPool>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("gchats")
        .and(warp::post())
        .and(with_auth())
        .and(warp::body::json::<CreateMessageGCRequestBody>())
        .and(with_db(pool))
        .and_then(handlers::create_message_gc)
}

fn get_chats_gc(pool: Arc<PgPool>) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
  warp::path!("gchats")
    .and(warp::get())
    .and(with_auth())
    .and(with_db(pool))
    .and_then(handlers::get_chats_gc)
}

fn get_messages_gc(pool: Arc<PgPool>) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
  warp::path!("gchats" / i32)
  .and(warp::get())
  .and(with_auth())
  .and(with_db(pool))
  .and_then(handlers::get_messages_gc)
}

fn with_db(
    pool: Arc<PgPool>,
) -> impl Filter<Extract = (Arc<PgPool>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || pool.clone())
}

fn with_auth() -> impl Filter<Extract = (i32,), Error = std::convert::Infallible> + Clone {
    warp::header::headers_cloned().map(move |headers: HeaderMap| {
        let user_id_res = authorize(headers);
        match user_id_res {
            Ok(suid) => {
                let ouid = suid.parse::<i32>();
                match ouid {
                    Ok(uid) => uid,
                    Err(_) => -1,
                }
            }
            Err(_) => -1,
        }
    })
}
