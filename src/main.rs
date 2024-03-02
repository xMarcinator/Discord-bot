extern crate dotenv;

use std::env;
use std::sync::Arc;

use actix_web::{App, get, HttpResponse, HttpServer, post, Responder, web};
use dotenv::dotenv;
use poise::serenity_prelude as serenity;
use serde::Deserialize;
use serenity::http::{Http, StatusCode};
use serenity::json::json;
use serenity::model::channel::Embed;
use serenity::model::prelude::{ChannelId, Webhook};
use serenity::prelude::*;

struct AppState {
    discord_http: Arc<Http>,
    webhook: Arc<RwLock<Option<Webhook>>>,
}

struct BotState {
    webhook: Arc<RwLock<Option<Webhook>>>,
}

#[derive(Deserialize)]
struct Info {
    msg: String,
    title: Option<String>,
    username: Option<String>,
    msg_type: Option<MsgType>,
}

#[derive(Deserialize, PartialEq, Debug)]
enum MsgType {
    Simple,
    Embed,
    Embed2,
}


#[get("/")]
async fn hello(data: web::Data<AppState>, info: web::Query<Info>) -> impl Responder {
    let hook = data.webhook.clone();

    let locked_hook = hook.read().await;

    let http = data.discord_http.clone();

    let webhook = match locked_hook.as_ref() {
        Some(webhook) => webhook,
        None => {
            return HttpResponse::build(StatusCode::SERVICE_UNAVAILABLE).body("Please first setup a Webhook in the Group");
        }
    };

    match &info.msg_type {
        Some(msg_type) if msg_type == &MsgType::Simple => {
            webhook.execute(http, false, |w| {
                if let Some(username) = &info.username {
                    w.username(&username);
                }

                w.content(&info.msg)
            }).await.unwrap();
        }
        Some(msg_type) if msg_type == &MsgType::Embed => {
            ChannelId(1166665522448961559).send_message(http, |m| {
                m.embed(|e| {
                    e.description(&info.msg);

                    if let Some(title) = &info.title {
                        e.title(title);
                    }

                    e.author(|a| {
                        if let Some(username) = &info.username {
                            a.name(username);
                        };
                        a
                    });
                    e
                });

                m
            }).await.unwrap();
        }
        Some(msg_type) if msg_type == &MsgType::Embed2 => {
            let embed = Embed::fake(|e| {
                e.description(&info.msg);

                if let Some(title) = &info.title {
                    e.title(title);
                }
                e
            });

            webhook.execute(http, false, move |w| {
                if let Some(username) = &info.username {
                    w.username(&username);
                }

                w.embeds(vec![
                    embed
                ]);

                w
            }).await.unwrap();
        }
        _ => println!("Message type: {:?}", info.msg_type),
    };


    HttpResponse::build(StatusCode::NO_CONTENT).finish()
}

#[post("/health")]
async fn health() -> impl Responder {
    HttpResponse::Ok()
}

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, BotState, Error>;

/// Displays your or another user's account creation date
#[poise::command(slash_command)]
async fn age(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let u = user.as_ref().unwrap_or_else(|| ctx.author());
    let response = format!("{}'s account was created at {}", u.name, u.created_at());
    ctx.say(response).await?;
    Ok(())
}

#[poise::command(slash_command)]
async fn ping(
    ctx: Context<'_>,
) -> Result<(), Error> {
    ctx.say("Pong!").await?;
    Ok(())
}

#[poise::command(slash_command)]
async fn set_channel(
    ctx: Context<'_>,
    #[description = "Channel to send messages"] channel: serenity::Channel,
) -> Result<(), Error> {
    let webhook = match channel.id().create_webhook(ctx, "test").await {
        Ok(webhook) => webhook,
        Err(err) => {
            ctx.say(format!("Error creating webhook: {:?}", err)).await?;
            return Ok(());
        }
    };

    let mut webhook_data = ctx.data().webhook.write().await;

    *webhook_data = Some(webhook);

    let response = format!("Channel set to {}", channel);
    ctx.say(response).await?;
    Ok(())
}


#[tokio::main]
async fn main() {
    dotenv().ok();

    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a DISCORD_TOKEN in the environment");
    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![age()],
            ..Default::default()
        })
        .token(token)
        .intents(intents)
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                println!("{:?} is connected!", ctx.http.get_current_user().await.unwrap());

                let webhook = Arc::new(RwLock::new(None));

                let app_state = web::Data::new(AppState {
                    discord_http: ctx.http.clone(),
                    webhook: webhook.clone(),
                });

                let bot_state = BotState {
                    webhook: webhook.clone(),
                };

                tokio::spawn(async move {
                    let server = HttpServer::new(move || {
                        App::new()
                            .app_data(app_state.clone())
                            .service(hello).service(health)
                    }).workers(1).bind(("127.0.0.1", 8080)).unwrap().run();

                    server.await.expect("TODO: panic message");
                });


                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(bot_state)
            })
        }).options(poise::FrameworkOptions {
        //on_error: |err| Box::pin(my_error_function(err)),
        // This is also where commands go
        commands: vec![
            ping(),
            set_channel(),
            // You can also modify a command by changing the fields of its Command instance
            poise::Command {
                // [override fields here]
                ..age()
            },
        ],
        ..Default::default()
    });

    framework.run().await.unwrap();
}