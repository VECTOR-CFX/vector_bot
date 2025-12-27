mod commands;
mod config;
mod ticket_system;

use poise::serenity_prelude as serenity;
use std::env;
use dotenv::dotenv;
use std::time::Instant;
use std::sync::{Arc, Mutex};
use sysinfo::System;
use std::collections::HashMap;
use ticket_system::structs::{TicketState, TicketStore};
use config::Config;
use tokio::sync::RwLock;

pub struct Data {
    pub start_time: Instant,
    pub system_info: Arc<Mutex<System>>,
    pub config: Config,
    pub ticket_store: Arc<RwLock<TicketStore>>,
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
    let ticket_store = TicketStore::load();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                commands::general::hello(),
                commands::info::info(),
                commands::ticket::rep(),
                commands::ticket::close(),
                commands::ticket::rename(),
            ],
            event_handler: |ctx, event, framework, data| {
                Box::pin(ticket_system::events::handle_event(ctx, event, framework, data))
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
                    ticket_store: Arc::new(RwLock::new(ticket_store)),
                    ticket_states: Arc::new(RwLock::new(HashMap::new())),
                };

                let store_clone = data.ticket_store.clone();
                let http_clone = ctx.http.clone();
                
                tokio::spawn(async move {
                    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(3600)); 
                    loop {
                        interval.tick().await;
                        check_inactive_tickets(&store_clone, &http_clone).await;
                    }
                });

                Ok(data)
            })
        })
        .build();
        

async fn check_inactive_tickets(
    store: &Arc<RwLock<TicketStore>>,
    http: &serenity::Http,
) {
    let now = chrono::Utc::now().timestamp();
    let tickets_to_close: Vec<u64> = {
        let store_read = store.read().await;
        store_read.tickets.iter()
            .filter(|(_, t)| now - t.last_activity > 172800)
            .map(|(uid, _)| *uid)
            .collect()
    };
    
    let tickets_to_remind: Vec<u64> = {
        let store_read = store.read().await;
        store_read.tickets.iter()
            .filter(|(_, t)| now - t.last_activity > 86400 && !t.has_been_reminded) 
            .map(|(uid, _)| *uid)
            .collect()
    };
    
    for uid in tickets_to_close {
        let ticket_opt = {
            let mut store_write = store.write().await;
            store_write.tickets.remove(&uid)
        };
        
        if let Some(ticket) = ticket_opt {
            let _ = store.write().await.save(); 
            
            let _ = serenity::ChannelId::new(ticket.channel_id).delete(http).await;
            
            let user_id = serenity::UserId::new(uid);
            if let Ok(dm) = user_id.create_dm_channel(http).await {
                let _ = dm.say(http, "Votre ticket a été fermé automatiquement suite à 48h d'inactivité.").await;
            }
        }
    }
    
    for uid in tickets_to_remind {
        let mut store_write = store.write().await;
        if let Some(ticket) = store_write.tickets.get_mut(&uid) {
            ticket.has_been_reminded = true;
            let _ = store_write.save();
            
            let user_id = serenity::UserId::new(uid);
            if let Ok(dm) = user_id.create_dm_channel(http).await {
                let _ = dm.say(http, "Bonjour, votre ticket est inactif depuis 24h. Avez-vous toujours besoin d'aide ? Sans réponse de votre part, il sera fermé dans 24h.").await;
            }
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
