use crate::{Context, Error};

#[poise::command(slash_command)]
pub async fn hello(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("👋 Bonjour ! Je suis un bot Discord en Rust, et je suis opérationnel !").await?;
    Ok(())
}
