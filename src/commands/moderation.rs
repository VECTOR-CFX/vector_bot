use crate::{Context, Error};
use poise::serenity_prelude as serenity;
use crate::ticket_system::structs::BlacklistInfo;

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

#[poise::command(slash_command, guild_only)]
pub async fn blticket(
    ctx: Context<'_>,
    #[description = "L'utilisateur à blacklist"] user: serenity::User,
    #[description = "La raison du blacklist"] reason: String,
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

    {
        let mut store = data.ticket_store.write().await;
        store.blacklist.insert(user.id.get(), BlacklistInfo {
            reason: reason.clone(),
            by: ctx.author().id.get(),
            date: chrono::Utc::now().timestamp(),
        });
        store.save();
    }

    ctx.send(poise::CreateReply::default()
        .content(format!("**{}** a été blacklisté des tickets pour la raison : *{}*", user.name, reason))
        .ephemeral(true)
    ).await?;

    Ok(())
}

#[poise::command(slash_command, guild_only)]
pub async fn unblticket(
    ctx: Context<'_>,
    #[description = "L'utilisateur à unblacklist"] user: serenity::User,
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

    let removed = {
        let mut store = data.ticket_store.write().await;
        let removed = store.blacklist.remove(&user.id.get());
        store.save();
        removed.is_some()
    };

    if removed {
        ctx.send(poise::CreateReply::default()
            .content(format!("**{}** a été retiré de la blacklist ticket.", user.name))
            .ephemeral(true)
        ).await?;
    } else {
        ctx.send(poise::CreateReply::default()
            .content(format!("**{}** n'était pas dans la blacklist.", user.name))
            .ephemeral(true)
        ).await?;
    }

    Ok(())
}
