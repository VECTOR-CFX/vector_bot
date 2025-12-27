use crate::{Context, Error};
use poise::serenity_prelude as serenity;

#[poise::command(slash_command, guild_only)]
pub async fn clear(
    ctx: Context<'_>,
    #[description = "Nombre de messages à supprimer (max 99)"] 
    #[min = 1] 
    #[max = 99] 
    amount: u64,
) -> Result<(), Error> {
    let data = ctx.data();
    
    let has_role = if let Some(member) = ctx.author_member().await {
        member.roles.contains(&serenity::RoleId::new(data.config.roles.staff_role_id))
    } else {
        false
    };

    if !has_role {
        ctx.send(poise::CreateReply::default()
            .content("Vous n'avez pas la permission d'utiliser cette commande.")
            .ephemeral(true)
        ).await?;
        return Ok(());
    }

    ctx.defer_ephemeral().await?;

    let channel = ctx.channel_id();
    
    let messages = channel.messages(&ctx, serenity::GetMessages::new().limit(amount as u8)).await?;
    
    if !messages.is_empty() {
        channel.delete_messages(&ctx, messages).await?;
        
        ctx.send(poise::CreateReply::default()
            .content(format!("{} messages supprimés.", amount))
            .ephemeral(true)
        ).await?;
    } else {
        ctx.send(poise::CreateReply::default()
            .content("Aucun message à supprimer.")
            .ephemeral(true)
        ).await?;
    }

    Ok(())
}
