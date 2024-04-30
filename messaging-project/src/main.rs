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
        } else if &args[0] == &"chats" {
            let config = rt.block_on(ret_chats(args, &tok)).unwrap();
            println!("All Chats{:?}", config);
        } else if &args[0] == &"username" && &args[2] == &"password"{
            let config = rt.block_on(login_user(args)).unwrap();
            tok = config;
            println!("Successfully signed in!");
        } else if &args[0] == &"createuser" {
            let config = rt.block_on(create_user_account(args)).unwrap();
            println!("Create User Status, {:?}", config);
            println!("202 : successful user creation");
        } else if &args[0] == &":q" {
            break;
        } else if &args[0] == &"-h" || &args[0] == &"-help" {
            println!("COMMANDS: ");
            println!("createuser <username> <password> - creates a user/account");
            println!("messages <username> - shows the messages connected to a user");
            println!("send <username>:<message> - sends message the colon is necessary to successfully send message");
            println!("chats - command will show chats associated with logged in user");
            println!("username <username> password <password> - signs in user");
            println!(":q - quits all processes");
        } else {
            println!("Perform a -h or -help command to view valid inputs.");
            break;
        }
    }
}
