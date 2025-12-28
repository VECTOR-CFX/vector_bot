mod commands;
mod config;
mod database;
mod ticket_system;
mod voice_system;
mod giveaway_system;

use poise::serenity_prelude as serenity;
use std::env;
use dotenv::dotenv;
use std::time::Instant;
use std::sync::{Arc, Mutex};
use sysinfo::System;
use std::collections::HashMap;
use ticket_system::structs::TicketState;
use config::Config;
use tokio::sync::RwLock;
use sqlx::{Pool, Sqlite};

pub struct Data {
    pub start_time: Instant,
    pub system_info: Arc<Mutex<System>>,
    pub config: Config,
    pub db: Pool<Sqlite>, 
    pub ticket_states: Arc<RwLock<HashMap<u64, TicketState>>>,
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let token = env::var("DISCORD_TOKEN").expect("Le token 'DISCORD_TOKEN' est manquant dans le fichier .env");
    let guild_id = env::var("DISCORD_GUILD_ID")
        .expect("L'ID du serveur 'DISCORD_GUILD_ID' est manquant dans le fichier .env")
        .parse::<u64>()
        .expect("L'ID du serveur doit être un nombre valide");
    
    let intents = serenity::GatewayIntents::non_privileged() 
        | serenity::GatewayIntents::DIRECT_MESSAGES 
        | serenity::GatewayIntents::MESSAGE_CONTENT 
        | serenity::GatewayIntents::GUILD_MESSAGES; 

    let config = Config::load().expect("Impossible de charger config.toml");
    
    let db = database::init_db().await.expect("Impossible d'initialiser la base de données");

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                commands::general::hello(),
                commands::info::info(),
                commands::ticket::rep(),
                commands::ticket::close(),
                commands::ticket::rename(),
                commands::moderation::clear(),
                commands::moderation::blticket(),
                commands::moderation::unblticket(),
                commands::profile::profil(),
                commands::giveaway::giveaway(),
            ],
            event_handler: |ctx, event, framework, data| {
                Box::pin(async move {
                    ticket_system::events::handle_event(ctx, event, framework, data).await?;
                    voice_system::events::handle_event(ctx, event, framework, data).await?;
                    giveaway_system::events::handle_event(ctx, event, framework, data).await?;
                    Ok(())
                })
            },
            ..Default::default()
        })
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                println!("Enregistrement des commandes slash pour le serveur {}...", guild_id);
                poise::builtins::register_in_guild(ctx, &framework.options().commands, serenity::GuildId::new(guild_id)).await?;
                
                ctx.set_activity(Some(serenity::ActivityData::streaming("DM FOR HELP", "https://twitch.tv/discord").expect("Erreur lors de la définition de l'activité")));

                println!("Le bot est prêt ! Connecté en tant que {}", _ready.user.name);
                
                let mut sys = System::new_all();
                sys.refresh_all();
                
                let data = Data {
                    start_time: std::time::Instant::now(),
                    system_info: Arc::new(Mutex::new(sys)),
                    config: config.clone(),
                    db: db.clone(),
                    ticket_states: Arc::new(RwLock::new(HashMap::new())),
                };

                let db_clone = data.db.clone();
                let http_clone = ctx.http.clone();
                
                tokio::spawn(async move {
                    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(3600)); 
                    loop {
                        interval.tick().await;
                        check_inactive_tickets(&db_clone, &http_clone).await;
                    }
                });

                let db_clone_gw = data.db.clone();
                let http_clone_gw = ctx.http.clone();
                let log_gw = config.channels.giveaway_log_channel_id;
                
                tokio::spawn(async move {
                    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5)); 
                    loop {
                        interval.tick().await;
                        giveaway_system::events::check_giveaways(&db_clone_gw, &http_clone_gw, log_gw).await;
                    }
                });

                Ok(data)
            })
        })
        .build();
        

async fn check_inactive_tickets(
    db: &Pool<Sqlite>,
    http: &serenity::Http,
) {
    let now = chrono::Utc::now().timestamp();
    
    let threshold_close = now - 172800;
    
    let tickets_to_close: Vec<(i64, i64)> = sqlx::query_as(
        "SELECT user_id, channel_id FROM tickets WHERE last_activity < ?"
    )
    .bind(threshold_close)
    .fetch_all(db)
    .await
    .unwrap_or_default();
    
    let threshold_remind = now - 86400;
    
    let tickets_to_remind: Vec<i64> = sqlx::query_scalar(
        "SELECT user_id FROM tickets WHERE last_activity < ? AND has_been_reminded = 0"
    )
    .bind(threshold_remind)
    .fetch_all(db)
    .await
    .unwrap_or_default();
    
    for (uid, channel_id) in tickets_to_close {
        let _ = sqlx::query("DELETE FROM tickets WHERE user_id = ?")
            .bind(uid)
            .execute(db)
            .await;
            
        let _ = serenity::ChannelId::new(channel_id as u64).delete(http).await;
        
        let user_id = serenity::UserId::new(uid as u64);
        if let Ok(dm) = user_id.create_dm_channel(http).await {
            let _ = dm.say(http, "Votre ticket a été fermé automatiquement suite à 48h d'inactivité.").await;
        }
    }
    
    for uid in tickets_to_remind {
        let _ = sqlx::query("UPDATE tickets SET has_been_reminded = 1 WHERE user_id = ?")
            .bind(uid)
            .execute(db)
            .await;
            
        let user_id = serenity::UserId::new(uid as u64);
        if let Ok(dm) = user_id.create_dm_channel(http).await {
            let _ = dm.say(http, "Bonjour, votre ticket est inactif depuis 24h. Avez-vous toujours besoin d'aide ? Sans réponse de votre part, il sera fermé dans 24h.").await;
        }
    }
}


    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;

    if let Err(why) = client {
        eprintln!("Erreur lors de la création du client : {:?}", why);
        return;
    }

    if let Err(why) = client.unwrap().start().await {
        eprintln!("Erreur lors de l'exécution du client : {:?}", why);
    }
}
