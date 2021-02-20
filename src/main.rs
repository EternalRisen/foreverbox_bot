// std
use std::{
    collections::{
        HashMap,
        HashSet,
    },
    env,
    sync::Arc,
};

// Serenity
use serenity::{async_trait, 
    client::bridge::gateway::ShardManager, 
    framework::{
        standard::{
            CommandResult,
            StandardFramework,
            macros::{
                group,
                hook
            }
        }
    },
    http::Http,
    model::{
        gateway::{
            Activity,
            Ready,
        },
        prelude::*,
    },
    prelude::*
};

//dotenv
use dotenv::dotenv;

// commands
mod commands;

use commands::{
    ping::*,
};

// Bot Setup
#[group]
#[commands(ping)]
struct General;

// Shard Manager
struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

// Command Counter
struct CommandCounter;

impl TypeMapKey for CommandCounter {
    type Value = HashMap<String, u64>;
}

// Event Handler
struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        ctx.set_activity(Activity::playing("Go to the forever box")).await;
        println!("{} is online.", ready.user.name);
    }
}

#[hook]
async fn before(ctx: &Context, _msg: &Message, command_name: &str) -> bool {
    //println!("Got command '{}' by user '{}'", command_name, _msg.author.name);

    // Increase command count after each command by an increment of 1
    let mut data = ctx.data.write().await;
    let counter = data.get_mut::<CommandCounter>().expect("Expected CommandCounter in TypeMap.");
    let entry = counter.entry(command_name.to_string()).or_insert(0);
    *entry += 1;

    true
}

#[hook]
async fn after(_ctx: &Context, _msg: &Message, command_name: &str, command_result: CommandResult) {
    match command_result {
        Ok(()) => return, //println!("Processed command '{}'", command_name),
        Err(why) => println!("Command '{}' returned error {:?}", command_name, why),
    }
}

#[tokio::main]
async fn main() {
    // Load .env
    dotenv().ok();

    // Get Token
    let token = env::var("TOKEN").expect("Failed to get variable TOKEN from environment.");

    // Get bot owners
    let http = Http::new_with_token(&token);
    let (owners, _bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);

            (owners, info.id)
        },
        Err(error) => panic!("Could not access application info: {:?}", error),
    };

    // Initialize framework
    let framework = StandardFramework::new()
        .configure(|c| c
            .owners(owners)
            .prefix("f!"))
        .before(before)
        .after(after)
        .group(&GENERAL_GROUP);

    // Initialize client
    let mut bot = Client::builder(token)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error with creating client.  is the token correct?");

    {
        let mut data = bot.data.write().await;
        data.insert::<CommandCounter>(HashMap::default());
        data.insert::<ShardManagerContainer>(bot.shard_manager.clone());
    }

    if let Err(error) = bot.start().await {
        println!("Client Error:  {:?}", error)
    }
}
