mod commands;

use poise::serenity_prelude as serenity;
use std::env;
use dotenv::dotenv;

pub struct Data {} 

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
    
    let intents = serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                commands::general::hello(),
            ],
            ..Default::default()
        })
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                println!("Enregistrement des commandes slash pour le serveur {}...", guild_id);
                poise::builtins::register_in_guild(ctx, &framework.options().commands, serenity::GuildId::new(guild_id)).await?;
                println!("Le bot est prêt ! Connecté en tant que {}", _ready.user.name);
                Ok(Data {})
            })
        })
        .build();

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
