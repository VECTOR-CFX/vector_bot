use crate::{Context, Error};
use poise::serenity_prelude as serenity;

#[poise::command(slash_command, guild_only)]
pub async fn profil(
    ctx: Context<'_>,
    user: Option<serenity::User>,
) -> Result<(), Error> {
    let target_user = user.as_ref().unwrap_or(ctx.author());
    let data = ctx.data();

    let member = ctx.guild_id().unwrap().member(ctx, target_user.id).await?;

    let created_at = target_user.created_at().format("%d/%m/%Y").to_string();
    let joined_at = member.joined_at.map(|d| d.format("%d/%m/%Y").to_string()).unwrap_or("Inconnue".to_string());

    let is_staff = member.roles.contains(&serenity::RoleId::new(data.config.roles.staff_role_id));
    let is_client = member.roles.contains(&serenity::RoleId::new(data.config.roles.client_role_id));

    let is_blacklisted = {
        let store = data.ticket_store.read().await;
        store.blacklist.contains_key(&target_user.id.get())
    };

    let staff_status = if is_staff { "Oui" } else { "Non" };
    let client_status = if is_client { "Oui" } else { "Non" };
    let blacklist_status = if is_blacklisted { "Oui" } else { "Non" };

    let avatar_url = member.face();

    ctx.send(poise::CreateReply::default()
        .ephemeral(true)
        .embed(serenity::CreateEmbed::new()
            .title(format!("Profil de {} ({})", target_user.name, target_user.id))
            .thumbnail(avatar_url)
            .color(0x3498db)
            .field("Mention", format!("<@{}>", target_user.id), true)
            .field("Création du compte", created_at, true)
            .field("Rejoint le serveur", joined_at, true)
            .field("Staff", staff_status, true)
            .field("Client", client_status, true)
            .field("Blacklist Ticket", blacklist_status, true)
            .footer(serenity::CreateEmbedFooter::new("VECTOR"))
            .timestamp(serenity::Timestamp::now())
        )
    ).await?;

    Ok(())
}
