use std::fmt::{ self, format };
use std::collections::HashMap;
use std::future::Future;
use reqwest::{ redirect, Client, Error, Response, StatusCode };
use serde::{Deserialize, Serialize};


//TODO: better overall formatting of data being pulled from database
// ######################################################################
//                          HELPER FUNCTIONS
// ######################################################################

// helper function for posting messages
async fn message_post_helper(
    username: &str,
    message: &str,
    token: &str
) -> Result<StatusCode, &'static str> {
    println!("{}", username);
    let url: String = format!("http://localhost:8001/chats/{}/", username);
    let mut map = HashMap::new();
    map.insert("message", message);
    println!("{:?}", &map);

    let bearer = "Bearer ".to_owned();

    let client = reqwest::Client::new();
    let res = client
        .post(url)
        .json(&map)
        .header("Authorization", bearer + token)
        .header("content-type", "application/json")
        .send().await;
    let final_res = match res {
        Ok(r) => {
            r
        }
        Err(_) => {
            return Err("Error: posting request");
        }
    };
    let res = final_res.status();
    Ok(res)
}

// message getter function 
async fn message_get_helper(username: &str, token: &str) -> Result<String, Error> {
    let url: String = format!("http://localhost:8001/chats/{}/messages", username);
    let client = reqwest::Client::new();
    let bearer = "Bearer ".to_owned();
    let get_res = client
        .get(&url)
        .header("Authorization", bearer + token)
        .send().await?;

    let final_result = get_res.text().await?;
    Ok(final_result)
}

// function which get's chats
async fn get_chats_helper(token: &str) -> Result<Response, &'static str> {
    let url: String = format!("http://localhost:8001/chats");
    let input = "Bearer ".to_owned();

    let client = reqwest::Client::new();
    let res = client
        .get(url)
        .header("Authorization", input + token)
        .send().await;
    let final_res = match res {
        Ok(r) => r,
        Err(_) => {
            return Err("Error: posting request");
        }
    };
    Ok(final_res)
}

async fn create_chat_helper(user: &str, token: &str) -> Result<StatusCode, &'static str> {
    let url: String = format!("http://localhost:8001/chats/");
    let mut map = HashMap::new();
    map.insert("buddy_id", user);

    let client = reqwest::Client::new();
    let bearer = "Bearer ".to_owned();
    let res = client.post(url).json(&map).header("Authorization", bearer + token).send().await;
    let final_res = match res {
        Ok(r) => r,
        Err(_) => {
            return Err("Error: posting request");
        }
    };
    let result = final_res.status();
    Ok(result)
}

// helper function to post users 
async fn user_post_helper(username: &str, password: &str) -> Result<StatusCode, &'static str> {
    let url: String = format!("http://localhost:8001/users/");
    let mut map = HashMap::new();
    map.insert("username", username);
    map.insert("password", password);

    let client = reqwest::Client::new();
    let res = client.post(url).json(&map).send().await;
    let final_res = match res {
        Ok(r) => r,
        Err(_) => {
            return Err("Error: posting request");
        }
    };
    let result = final_res.status();
    Ok(result)
}

// login helper function
async fn login_post_helper(username: &str, password: &str) -> Result<Response, &'static str> {
    let url: String = format!("http://localhost:8001/login/");
    let mut map = HashMap::new();
    map.insert("username", username);
    map.insert("password", password);

    let client = reqwest::Client::new();
    let res = client.post(url).json(&map).send().await;

    let final_res = match res {
        Ok(r) => r,
        Err(_) => {
            return Err("Error: posting request");
        }
    };
    Ok(final_res)
}

#[derive(Deserialize, Debug, Serialize)]
struct GroupChatMessage<'a> {
    buddies: Vec<&'a str>,
    message: &'a str,
}

impl<'a> GroupChatMessage<'a> {
    fn build(buddies: Vec<&'a str>, message: &'a str) -> GroupChatMessage<'a> {
        GroupChatMessage { buddies: buddies, message: &message }
    }
}
// async fn group_chat_message_create(user1: &str, user2: &str, mes: &str, token: &str) -> Result<StatusCode, &'static str>{
//     let url: String = format!("http://localhost:8001/gchats/");
//     //let mut map = GroupChatMessage;
//     let bud = vec![user1, user2];
//     let map = GroupChatMessage::build(bud, mes);

//     let bearer = "Bearer ".to_owned();

//     let client = reqwest::Client::new();
//     let res = client
//         .post(url)
//         .json(&map)
//         .header("Authorization", bearer + token)
//         .header("content-type", "application/json")
//         .send().await;
//     let final_res = match res {
//         Ok(r) => {
//             r
//         }
//         Err(_) => {
//             return Err("Error: posting request");
//         }
//     };
//     let res = final_res.status();
//     Ok(res)
// }

async fn group_chat_message_create(args:Vec<&str>, mes: &str, token: &str) -> Result<StatusCode, &'static str>{
    let url: String = format!("http://localhost:8001/gchats/");
    //let mut map = GroupChatMessage;
    let mut bud = Vec::new();
    for i in args.iter(){
        if i != &"sendGroup"{
            bud.push(i.to_owned());
        }
    }
    println!("{:?}", &bud);
    let map = GroupChatMessage {buddies: bud, message: mes};

    let bearer = "Bearer ".to_owned();

    let client = reqwest::Client::new();
    let res = client
        .post(url)
        .json(&map)
        .header("Authorization", bearer + token)
        .header("content-type", "application/json")
        .send().await;
    let final_res = match res {
        Ok(r) => {
            r
        }
        Err(_) => {
            return Err("Error: posting request");
        }
    };
    let res = final_res.status();
    Ok(res)
}

async fn group_get_chats_helper(token: &str) -> Result<Response, &'static str> {
    let url: String = format!("http://localhost:8001/gchats/");
    let input = "Bearer ".to_owned();

    let client = reqwest::Client::new();
    let res = client
        .get(url)
        .header("Authorization", input + token)
        .send().await;
    let final_res = match res {
        Ok(r) => r,
        Err(_) => {
            return Err("Error: posting request");
        }
    };
    Ok(final_res)
}

async fn group_messages_get_helper(id: u32, token: &str) -> Result<String, &'static str> {
    let url: String = format!("http://localhost:8001/gchats/{}", id);
    let input = "Bearer ".to_owned();

    let client = reqwest::Client::new();
    let res = client
        .get(url)
        .header("Authorization", input + token)
        .send().await;
    let final_res = match res {
        Ok(r) => r,
        Err(_) => {
            return Err("Error: posting request");
        }
    };
    let mess_result = final_res.text().await;

    let res = match mess_result {
        Ok(r) => r,
        Err(_) => {
            return Err("Error: posting request")
        }
    };
    Ok(res)
}

// ######################################################################
//                              MAIN FUNCTIONS
// ######################################################################

pub async fn create_chat(args: Vec<&str>, token: &str) -> Result<StatusCode, &'static str> {
    let username: String = args[1].to_string();

    let res = create_chat_helper(&username, &token).await?;

    Ok(res)
}

pub async fn group_messages_get(args: Vec<&str>, token: &str) -> Result<String, &'static str> {
    if args.len() < 2 {
        return Err("not enough arguments");
    }
    if args.len() > 2 {
        return Err("too many arguments");
    }

    let user: u32 = args[1].to_string().parse::<u32>().unwrap();

    let mes = group_messages_get_helper(user, token).await;
    let result: Result<String, &str> = match mes {
        Ok(r) => Ok(r),
        Err(_) => {
            return Err("Error: fetching request");
        }
    };
    let final_result = match result {
        Ok(r) => r,
        Err(_) => {
            return Err("Error: Error in fetching the text of messages");
        }
    };
    let ret_result: String = final_result.split("\"").collect();
    let split_res:String= ret_result.split("},").collect();

    return Ok(split_res);
}

pub async fn group_get_chats(args: Vec<&str>, token: &str) -> Result<String, &'static str>{
    if args.len() < 1 {
        return Err("not enough arguments")
    }
    if args.len() >1 {
        return Err("too many arguments")
    }

    let messages = group_get_chats_helper(&token).await?;

    let code = messages.text().await;
    let text_ret = match code {
        Ok(r) => r,
        Err(_) => {
            return Err("Error: fetching request unsuccessful");
        }
    };

    let ret_result: String = text_ret.split("\"").collect();
    let split_res:String= ret_result.split("},").collect();
    return Ok(split_res);
}

pub async fn send_group_message(args: Vec<&str>, mes: &str, token: &str) -> Result<String, &'static str> {
    //let user1: String = args[1].to_string();
    //let user2: String = args[2].to_string();
    let message_return = &mes; 

    let outward_mes = group_chat_message_create(args, &mes, token).await?;
    println!("The Message status code: {:?}", outward_mes);
    return Ok(message_return.to_string());
}

// function which returns chats from database 
pub async fn ret_chats(args: Vec<&str>, token: &str) -> Result<String, &'static str> {
    if args.len() < 1 {
        return Err("not enough arguments");
    }
    if args.len() > 1 {
        return Err("too many arguments");
    }

    let messages = get_chats_helper(&token).await?;

    let code = messages.text().await;
    let text_ret = match code {
        Ok(r) => r,
        Err(_) => {
            return Err("Error: fetching request unsuccessful");
        }
    };

    let ret_result: String = text_ret.split("\"").collect();
    let split_res:String= ret_result.split("},").collect();

    return Ok(split_res);
}

// creates account for user 
pub async fn create_user_account(args: Vec<&str>) -> Result<StatusCode, &'static str> {
    if args.len() < 3 {
        return Err("not enough arguments");
    }
    if args.len() > 3 {
        return Err("too many arguments");
    }
    let username: String = args[1].to_string();
    let password: String = args[2].to_string();

    let res = user_post_helper(&username, &password).await?;

    Ok(res)
}

// gets user information from the database
#[derive(Deserialize, Debug)]
pub struct User {
    pub user_id: u32,
    pub username: String,
    pub password: String,
}
pub struct UserCommand {
    pub command: String,
    pub username: u32,
}

impl UserCommand {
    pub async fn build(args: Vec<&str>) -> Result<UserCommand, &'static str> {
        if args.len() < 2 {
            return Err("not enough arguments");
        }
        if args.len() > 2 {
            return Err("too many arguments");
        }
        let command: String = args[0].to_string();
        let username: u32 = args[1].to_string().parse::<u32>().unwrap();

        let user: UserCommand;
        user = UserCommand { command, username };
        let url = format!("http://localhost:8001/users/{}", user.username);

        let get_res = reqwest::get(url).await;

        let final_result = match get_res {
            Ok(r) => r.json::<User>().await,
            Err(_) => {
                return Err("Error: fetching request");
            }
        };

        let final_result_result = match final_result {
            Ok(r) => r,
            Err(_) => {
                return Err("Error: fetching request");
            }
        };
        println!("{:?}", final_result_result);
        Ok(user)
    }
}

impl fmt::Debug for UserCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.command, self.username)
    }
}

// pub async fn send_message(args: Vec<&str>, token: &str) -> Result<String, &'static str> {
//     let user: String = args[1].to_string();
//     let message: String = args[2].to_string();
//     let message_return: String = args[2].to_string();

//     let outward_mes = message_post_helper(&user, &message, token).await?;
//     println!("The Message status code: {:?}", outward_mes);
//     return Ok(message_return);
// }

// sending messages with more than one word
pub async fn send_message(args: Vec<&str>, mes: &str, token: &str) -> Result<String, &'static str> {
    let user: String = args[1].to_string();
    let message_return = &mes; 

    let outward_mes = message_post_helper(&user, &mes, token).await?;
    println!("The Message status code: {:?}", outward_mes);
    return Ok(message_return.to_string());
}

// returns all messages associated with given user
pub async fn messages_connected_user(args: Vec<&str>, token: &str) -> Result<String, &'static str> {
    if args.len() < 2 {
        return Err("not enough arguments");
    }
    if args.len() > 2 {
        return Err("too many arguments");
    }

    let user: String = args[1].to_string();

    let mes = message_get_helper(&user, token).await;
    let result: Result<String, &str> = match mes {
        Ok(r) => Ok(r),
        Err(_) => {
            return Err("Error: fetching request");
        }
    };
    let final_result = match result {
        Ok(r) => r,
        Err(_) => {
            return Err("Error: Error in fetching the text of messages");
        }
    };
    let ret_result: String = final_result.split("\"").collect();
    let split_res:String= ret_result.split("},").collect();

    return Ok(split_res);
}

// function logs user in within a foramt
pub async fn login_user(args: Vec<&str>) -> Result<String, &'static str>{
    if args.len() < 4 {
        return Err("not enough arguments");
    }
    if args.len() > 4 {
        return Err("too many arguments");
    }
    let username: String = args[1].to_string();
    let password: String = args[3].to_string();

    let val = login_post_helper(&username, &password).await?.text().await;
    let fin = match val {
        Ok(v) => v,
        Err(_) => {
            return Err("Login Failed");
        }
    };
    let tok: Vec<&str> = fin.split("{").collect();
    let tok_step: Vec<&str> = tok[1].split("\"").collect();
    //let final_tok: Vec<&str> = tok_step[].split(":").collect();
    let final_final: String = tok_step[3].to_string();
    Ok(final_final)
}
