use crate::{auth::authorize, requests_and_responses::{CreateChatRequestBody, CreateMessageRequestBody, LoginRequestBody, TokenResponse}};
use super::models::User;
use std::sync::Arc;
use sqlx::PgPool;
use warp::{ http::{HeaderMap, HeaderValue, StatusCode}, reject, reply::{self, with_status}, Rejection, Reply};
use serde::Serialize;
use super::auth::create_jwt;
use super::errors;
use std::future::Future;
use super::requests_and_responses::{UserResponse, Chat, ExistsQuery, Message, ChatResponse, CreateMessageGCRequestBody};

#[derive(Serialize)]
struct EmptyJson {}

pub async fn handle_rejection(err: Rejection) -> Result<impl Reply, std::convert::Infallible> {
  if err.is_not_found() {
    Ok(reply::with_status("NOT_FOUND", StatusCode::NOT_FOUND))
  } else if let Some(_) = err.find::<errors::ConflictError>() {
    Ok(reply::with_status("CONFLICT", StatusCode::CONFLICT))
  } else if let Some(_) = err.find::<errors::AuthorizationError>() {
    Ok(reply::with_status("NOT AUTHORIZED", StatusCode::UNAUTHORIZED))
  } else {
    Ok(reply::with_status("INTERNAL_SERVER_ERROR", StatusCode::INTERNAL_SERVER_ERROR))
  }
}

async fn handle_query<F, T>(query_future: F) -> Result<T, Rejection> 
where F: Future<Output = Result<T, sqlx::Error>> {
  let query_res = query_future.await;
  match query_res {
    Ok(res) => Ok(res),
    Err(e) => match e {
      sqlx::Error::RowNotFound => Err(reject::not_found()),
      _ => Err(reject::custom(errors::DatabaseError {}))
    },
  }
}

async fn handle_query_exists<F>(query_future: F) -> bool
where F: Future<Output = Result<ExistsQuery, sqlx::Error>> {
  let query = handle_query(query_future).await;
  match query {
    Ok(res) => res.exists.unwrap_or(false),
    Err(_) => false
  }
  
}

async fn check_auth(user_id: i32, pool: Arc<PgPool>) -> Result<(), Rejection> {
  let exists = handle_query_exists(sqlx::query_as!(ExistsQuery, "SELECT EXISTS (SELECT 1 FROM users WHERE user_id=$1) AS exists", user_id).fetch_one(&*pool)).await;
  if exists {
    Ok(())
  } else {
    Err(reject::custom(errors::AuthorizationError {}))
  }
}


pub async fn get_user(user_id: i32, pool: Arc<PgPool>) -> Result<impl Reply, Rejection> {
  let user = handle_query(sqlx::query_as!( User, 
    "SELECT * FROM users WHERE user_id = $1", user_id
  ).fetch_one(&*pool)).await?;
  let res = UserResponse{ user_id: user.user_id, username: user.username };
  Ok(reply::json(&res))
}

pub async fn login(body: LoginRequestBody, pool: Arc<PgPool>) -> Result<reply::WithStatus<impl Reply>, Rejection> {
  let username: String = body.username;
  let password: String = body.password;
  let user = handle_query(sqlx::query_as!(User, "SELECT * FROM users WHERE username=$1 AND password=$2", username, password).fetch_one(&*pool)).await?;

  let otoken = create_jwt(user.user_id);
  let token = match otoken {
    Ok(token) => token,
    Err(_) => return Err(reject::custom(errors::JwtError))
  };

  let response_body = TokenResponse {token: String::from(token)};
  Ok(reply::with_status(reply::json(&response_body), StatusCode::CREATED))
}

pub async fn create_user(body: LoginRequestBody, pool: Arc<PgPool>) -> Result<reply::WithStatus<impl Reply>, Rejection> {
  let username: String = body.username;
  let password: String = body.password;
  let username_exists = handle_query_exists(sqlx::query_as!(ExistsQuery, "SELECT EXISTS(SELECT 1 FROM users WHERE username=$1) AS exists", username).fetch_one(&*pool)).await;
  if username_exists {
    return Err(reject::custom(errors::ConflictError {}))
  };
  let created_user = handle_query(sqlx::query_as!(User, "INSERT INTO users (username, password) VALUES ($1, $2) RETURNING *", username, password).fetch_one(&*pool)).await?;
  let res = UserResponse {user_id: created_user.user_id, username: created_user.username};
  Ok(reply::with_status(reply::json(&res), StatusCode::CREATED))
}

pub async fn get_chats(user_id: i32, pool: Arc<PgPool>) -> Result<reply::WithStatus<reply::Json>, Rejection> {
  check_auth(user_id, pool.clone()).await?;
  let chats = handle_query(sqlx::query_as!( ChatResponse,
    "SELECT c.chat_id, m.message AS last_message, u.username AS with
    FROM (SELECT * FROM user_to_chat fu2c WHERE fu2c.user_id=$1) u2c
    JOIN chats c ON c.chat_id = u2c.chat_id
    JOIN (
      SELECT *, ROW_NUMBER() OVER (PARTITION BY chat_id ORDER BY message_id DESC) as rn FROM messages
    ) m ON c.chat_id = m.chat_id AND m.rn = 1
    JOIN (
      SELECT * FROM user_to_chat u2c2 WHERE u2c2.user_id != $1
    ) b ON c.chat_id = b.chat_id
    JOIN users u ON u.user_id = b.user_id
    ", user_id
  ).fetch_all(&*pool)).await?;
  
  Ok(reply::with_status(reply::json(&chats), StatusCode::OK))
}

pub async fn create_chat(user_id: i32, body: CreateChatRequestBody, pool: Arc<PgPool>) -> Result<reply::WithStatus<impl Reply>, Rejection> {
  check_auth(user_id, pool.clone()).await?;
  let buddy_id = body.buddy_id;
  let user = handle_query(sqlx::query_as!( User, 
    "SELECT * FROM users WHERE user_id = $1", user_id
  ).fetch_one(&*pool)).await?;

  let buddy = handle_query(sqlx::query_as!( User, 
    "SELECT * FROM users WHERE username = $1", buddy_id
  ).fetch_one(&*pool)).await?;
  // check if chat already exists -- checks that there the two users have one shared chat, which won't work if we ever implement group chats
  let chat_exists = handle_query_exists(sqlx::query_as!(ExistsQuery,
    "SELECT COUNT(DISTINCT c.chat_id) > 0 AS exists
     FROM chats c
     JOIN user_to_chat uc1 ON c.chat_id = uc1.chat_id AND uc1.user_id = $1
     JOIN user_to_chat uc2 ON c.chat_id = uc2.chat_id AND uc2.user_id = $2
   ", user.user_id, buddy.user_id).fetch_one(&*pool)).await;
   
   if chat_exists {
    return Err(reject::custom(errors::ConflictError {}))
   };
  
  // create new chat
  let chat = handle_query(sqlx::query_as!(Chat, "INSERT INTO chats DEFAULT VALUES RETURNING chat_id").fetch_one(&*pool)).await?;
  // connect users to new chat
  let _user_connection_success = handle_query(sqlx::query!("INSERT INTO user_to_chat (user_id, chat_id) VALUES ($1, $2)", user.user_id, chat.chat_id).execute(&*pool)).await?;
  let _buddy_connection_success = handle_query(sqlx::query!("INSERT INTO user_to_chat (user_id, chat_id) VALUES ($1, $2)", buddy.user_id, chat.chat_id).execute(&*pool)).await?;

  Ok(reply::with_status("Created", StatusCode::CREATED))
}

pub async fn create_message(buddy_username: String, user_id: i32, body: CreateMessageRequestBody, pool: Arc<PgPool>) -> Result<reply::WithStatus<impl Reply>, Rejection> {
  check_auth(user_id, pool.clone()).await?;
  let message = body.message;

  let buddy = handle_query(sqlx::query_as!(User, "SELECT * FROM users WHERE username=$1", buddy_username).fetch_one(&*pool)).await?;
  let chat = handle_query(sqlx::query_as!(Chat, "SELECT (c.chat_id) 
                                          FROM chats c 
                                          JOIN user_to_chat uc1
                                            ON c.chat_id = uc1.chat_id AND uc1.user_id=$1
                                          JOIN user_to_chat uc2
                                            ON c.chat_id = uc2.chat_id AND uc2.user_id=$2", user_id, buddy.user_id
                                    ).fetch_optional(&*pool)).await?;

  let chat_id = match chat {
    Some(c) => c.chat_id,
    None => {
      let new_chat = handle_query(sqlx::query_as!(Chat, "INSERT INTO chats DEFAULT VALUES RETURNING *").fetch_one(&*pool)).await?;
      let _user_conn_success = handle_query(sqlx::query!("INSERT INTO user_to_chat (user_id, chat_id) VALUES ($1, $2)", user_id, new_chat.chat_id).execute(&*pool)).await?;
      let _buddy_conn_success = handle_query(sqlx::query!("INSERT INTO user_to_chat (user_id, chat_id) VALUES ($1, $2)", buddy.user_id, new_chat.chat_id).execute(&*pool)).await?;
      new_chat.chat_id
    }
  };

  let _create_successful = handle_query(sqlx::query!(
    "INSERT INTO messages (chat_id, sent_from, message) VALUES ($1, $2, $3)", chat_id, user_id, message
  ).execute(&*pool)).await?;
  Ok(reply::with_status("CREATED", StatusCode::CREATED))
}

pub async fn get_messages(buddy_username: String, user_id: i32, pool: Arc<PgPool>) -> Result<reply::WithStatus<reply::Json>, Rejection> {
  check_auth(user_id, pool.clone()).await?;
  let buddy = handle_query(sqlx::query_as!(User, "SELECT * FROM users WHERE username=$1", buddy_username).fetch_one(&*pool)).await?;
  
  let chat = handle_query(sqlx::query_as!(Chat, 
                              "SELECT (c.chat_id) 
                              FROM chats c 
                              JOIN user_to_chat uc1
                                ON c.chat_id = uc1.chat_id AND uc1.user_id=$1
                              JOIN user_to_chat uc2
                                ON c.chat_id = uc2.chat_id AND uc2.user_id=$2
                              ", user_id, buddy.user_id
                            ).fetch_one(&*pool)).await?;

  let messages = handle_query(sqlx::query_as!(Message, "SELECT * FROM messages WHERE chat_id = $1", chat.chat_id).fetch_all(&*pool)).await?;
  Ok(with_status(reply::json(&messages), StatusCode::OK))
}

pub async fn get_user_with_token(headers: HeaderMap<HeaderValue>, pool: Arc<PgPool>) -> Result<reply::WithStatus<reply::Json>, Rejection> {
  let user_id_res = authorize(headers);
  let user_id = match user_id_res {
    Ok(suid) => {
      let ouid = suid.parse::<i32>();
      match ouid {
        Ok(uid) => uid,
        Err(_) => return Err(reject::custom(errors::JwtError))
      }
    },
    Err(_) => return Err(reject::custom(errors::JwtError))
  };
  let user = handle_query(sqlx::query_as!(User, "SELECT * FROM users WHERE user_id=$1", user_id).fetch_one(&*pool)).await?;
  let res = UserResponse{ user_id: user.user_id, username: user.username };
  Ok(with_status(reply::json(&res), StatusCode::OK))
}

pub async fn create_message_gc(user_id: i32, body: CreateMessageGCRequestBody, pool: Arc<PgPool>) -> Result<reply::WithStatus<impl Reply>, Rejection> {
  check_auth(user_id, pool.clone()).await?;
  let buddies = handle_query(sqlx::query_as!(User, "SELECT * FROM users WHERE username=ANY($1)", &body.buddies).fetch_all(&*pool)).await?;
  let mut ids = buddies.into_iter().map(|u| {u.user_id}).collect::<Vec<i32>>();
  ids.push(user_id);
  ids.sort();
  let chat = handle_query(sqlx::query_as!(Chat, 
                                          "SELECT c.chat_id
                                           FROM chats c
                                           JOIN user_to_chat u2c ON c.chat_id = u2c.chat_id
                                           GROUP BY c.chat_id
                                           HAVING ARRAY_AGG(u2c.user_id ORDER BY u2c.user_id) = $1
                                          ", &ids
                                    ).fetch_optional(&*pool)).await?;
  
  let chat_id = match chat {
    Some(c) => c.chat_id,
    None => {
      let new_chat = handle_query(sqlx::query_as!(Chat, "INSERT INTO chats DEFAULT VALUES RETURNING *").fetch_one(&*pool)).await?;
      for id in ids {
        handle_query(sqlx::query!("INSERT INTO user_to_chat (user_id, chat_id) VALUES ($1, $2)", id, new_chat.chat_id).execute(&*pool)).await?;
      }
      new_chat.chat_id
    }
  };

  let _create_successful = handle_query(sqlx::query!(
    "INSERT INTO messages (chat_id, sent_from, message) VALUES ($1, $2, $3)", chat_id, user_id, body.message
  ).execute(&*pool)).await?;
  Ok(reply::with_status("CREATED", StatusCode::CREATED))
}

pub struct ChatResponseGCMaybe {
    pub chat_id: i32,
    pub with: Option<Vec<String>>,
    pub last_message: Option<Vec<String>>,
}

#[derive(Serialize, Debug)]
pub struct ChatResponseGC {
  pub chat_id: i32,
  pub with: Vec<String>,
  pub last_message: String,
}



pub async fn get_chats_gc (user_id: i32, pool: Arc<PgPool>) -> Result<reply::WithStatus<impl Reply>, Rejection> {
  check_auth(user_id, pool.clone()).await?;
  let chats = handle_query(sqlx::query_as!( ChatResponseGCMaybe,
    "
    SELECT c.chat_id, ARRAY_AGG(m.message) AS last_message, ARRAY_AGG(u.username) AS with
    FROM (SELECT * FROM user_to_chat fu2c WHERE fu2c.user_id=$1) u2c
    JOIN chats c ON c.chat_id = u2c.chat_id
    JOIN (
      SELECT *, ROW_NUMBER() OVER (PARTITION BY chat_id ORDER BY message_id DESC) as rn FROM messages
    ) m ON c.chat_id = m.chat_id AND m.rn = 1
    JOIN (
      SELECT * FROM user_to_chat u2c2 WHERE u2c2.user_id != $1
    ) b ON c.chat_id = b.chat_id
    JOIN users u ON u.user_id = b.user_id
    GROUP BY c.chat_id
    ", user_id
  ).fetch_all(&*pool)).await?;

  let filtered_chats: Vec<ChatResponseGC> = chats.into_iter().filter(|c| {
    c.with.is_some() && c.last_message.is_some()
  }).map(|c| {
    ChatResponseGC {
      chat_id: c.chat_id, 
      with: c.with.unwrap(), 
      last_message: c.last_message.unwrap().first().unwrap_or(&String::from("")).to_string()}
  }).collect();

  Ok(reply::with_status(reply::json(&filtered_chats), StatusCode::OK))
}