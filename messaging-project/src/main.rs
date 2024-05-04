//use std::fmt::Error;
use std::process;
use std::io;
use tokio::runtime;

use messaging_project::{
    messages_connected_user,
    send_message,
    create_user_account,
    UserCommand,
    login_user,
    ret_chats,
    send_group_message,
    group_get_chats,
    group_messages_get,
    create_chat
};

fn main() {
    let mut buf = String::new();
    let mut tok: String = String::new();

    loop {
        buf = String::new();
        let _stream = io::stdin().read_line(&mut buf);
        buf.pop();
        let new_args: Vec<&str> = buf.split("\"").collect();
        //let mut args: Vec<&str> = new_args[0].split(" ").collect();

        // args_first used for messages
        let args_first: Vec<&str> = new_args[0].split(":").collect();
        let mut args: Vec<&str> = args_first[0].split(" ").collect();
        if new_args.len() > 1 {
            args.push(new_args[1]);
        }

        let rt = runtime::Runtime::new().unwrap();
        //let check = &the_args[0];
        if &args[0] == &"send" {
            let config = rt.block_on(send_message(args, &args_first[1],&tok)).unwrap();
            println!("Message: {:?}", config);
        } else if &args[0] == &"messages" {
            let config = rt.block_on(messages_connected_user(args, &tok)).unwrap();
            let message_list:Vec<&str> = config.split("{").collect();

            println!("All messages:");
            for i in message_list.iter(){
                println!("{:?}", i);
            };
        } else if &args[0] == &"groupmessages" {
            let config = rt.block_on(group_messages_get(args, &tok)).unwrap();
            let message_list:Vec<&str> = config.split("{").collect();

            println!("All group messages:");
            for i in message_list.iter(){
                if i != &"["{
                println!("{:?}", i);
                }
            };
        } else if &args[0] == &"chats" {
            let config = rt.block_on(ret_chats(args, &tok)).unwrap();
            let chat_list:Vec<&str> = config.split("{").collect();

            println!("All Chats");
            for i in chat_list.iter(){
                if i != &"["{
                println!("{:?}", i);
                }
            }
        } else if &args[0] == &"sendGroup" {
            let config = rt.block_on(send_group_message(args,&args_first[1], &tok)).unwrap();

            println!("Group Message: {:?}", config);
        } else if &args[0] == &"createchat" {
            let config = rt.block_on(create_chat(args, &tok)).unwrap();
            println!("create chat status: {:?}", config);
        } else if &args[0] == &"groupchats" {
            let config = rt.block_on(group_get_chats(args, &tok)).unwrap();
            let chat_list:Vec<&str> = config.split("{").collect();

            println!("All Group Chats: ");
            for i in chat_list.iter() {
                if i != &"["{
                    println!("{:?}", i)
                }
            }
            
        } else if &args[0] == &"username" && &args[2] == &"password"{
            let config = rt.block_on(login_user(args)).unwrap();
            tok = config;
            println!("Successfully signed in!");
        } else if &args[0] == &"createuser" {
            let config = rt.block_on(create_user_account(args)).unwrap();
            println!("Create User Status, {:?}", config);
            println!("202 : successful user creation");
        } else if &args[0] == &"quit" {
            break;
        } else if &args[0] == &"-h" || &args[0] == &"-help" {
            println!("COMMANDS: ");
            println!("createuser <username> <password> - creates a user/account");
            println!("messages <username> - shows the messages connected to a user");
            println!("send <username>:<message> - sends message the colon is necessary to successfully send message");
            println!("chats - command will show chats associated with logged in user");
            println!("username <username> password <password> - signs in user");
            println!("sendGroup <username> <username>:<message> - creates a groupchat with only 3 people");
            println!("groupmessages <chat_id> - returns the group chats with the corresponding id number");
            println!("groupchats - returns all group chats the current user is asscoiateed with");
            println!("quit - quits all processes");
        } else {
            println!("Perform a -h or -help command to view valid inputs.");
        }
    }
}
