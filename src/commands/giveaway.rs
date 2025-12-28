use poise::serenity_prelude as serenity;
use crate::{Context, Error};

#[poise::command(slash_command, guild_only)]
pub async fn giveaway(ctx: Context<'_>) -> Result<(), Error> {
    let staff_role = ctx.data().config.roles.staff_role_id;
    let has_role = ctx.author().has_role(ctx.http(), ctx.guild_id().unwrap(), serenity::RoleId::new(staff_role)).await?;
    
    if !has_role {
        ctx.send(poise::CreateReply::default().content("Vous n'avez pas la permission.").ephemeral(true)).await?;
        return Ok(());
    }

    let embed = serenity::CreateEmbed::new()
        .title("Gestion des Giveaways")
        .description("Sélectionnez une action :")
        .color(0x3498db);

    let buttons = vec![
        serenity::CreateButton::new("giveaway_create_btn").label("Create").style(serenity::ButtonStyle::Success),
        serenity::CreateButton::new("giveaway_delete_btn").label("Delete").style(serenity::ButtonStyle::Danger),
    ];

    ctx.send(poise::CreateReply::default()
        .embed(embed)
        .components(vec![serenity::CreateActionRow::Buttons(buttons)])
        .ephemeral(true)
    ).await?;

    Ok(())
}
