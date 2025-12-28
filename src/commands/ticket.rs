use crate::{Context, Error};
use poise::serenity_prelude as serenity;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use crate::ticket_system::structs::TicketInfo;

#[poise::command(slash_command, guild_only)]
pub async fn rep(
    ctx: Context<'_>,
    #[description = "Le message à envoyer à l'utilisateur"] message: String,
) -> Result<(), Error> {
    let data = ctx.data();
    let channel_id = ctx.channel_id();

    let ticket: Option<TicketInfo> = sqlx::query_as(
        "SELECT * FROM tickets WHERE channel_id = ?"
    )
    .bind(channel_id.get() as i64)
    .fetch_optional(&data.db)
    .await?;
    
    if let Some(ticket) = ticket {
        let user_id = serenity::UserId::new(ticket.user_id as u64);
        
        let dm_channel = user_id.create_dm_channel(&ctx).await?;
        
        let content = format!("**Support**: {}", message);
        dm_channel.say(&ctx, content).await?;
        
        ctx.send(poise::CreateReply::default()
            .content(format!("Message envoyé à <@{}> : {}", ticket.user_id, message))
            .ephemeral(true)
        ).await?;
        
        ctx.channel_id().say(&ctx, format!("**Staff ({}):** {}", ctx.author().name, message)).await?;
        
    } else {
        ctx.send(poise::CreateReply::default()
            .content("Ce salon n'est pas un ticket actif.")
            .ephemeral(true)
        ).await?;
    }
    
    Ok(())
}

#[poise::command(slash_command, guild_only)]
pub async fn close(ctx: Context<'_>) -> Result<(), Error> {
    let data = ctx.data();
    let channel_id = ctx.channel_id();
    
    let ticket: Option<TicketInfo> = sqlx::query_as(
        "SELECT * FROM tickets WHERE channel_id = ?"
    )
    .bind(channel_id.get() as i64)
    .fetch_optional(&data.db)
    .await?;

    if let Some(ticket) = ticket {
        sqlx::query("DELETE FROM tickets WHERE user_id = ?")
            .bind(ticket.user_id)
            .execute(&data.db)
            .await?;

        ctx.defer().await?;

        let user_id = serenity::UserId::new(ticket.user_id as u64);
        
        let mut messages = ctx.channel_id().messages(&ctx, serenity::GetMessages::new().limit(100)).await?;
        messages.reverse(); 

        let open_date = chrono::DateTime::from_timestamp(ticket.created_at, 0)
            .unwrap_or_default()
            .format("%d/%m/%Y %H:%M:%S")
            .to_string();
        let close_date = chrono::Local::now().format("%d/%m/%Y %H:%M:%S").to_string();

        let mut transcript = String::new();
        transcript.push_str(&format!("=== TRANSCRIPT TICKET ===\n"));
        transcript.push_str(&format!("Utilisateur : {} (ID: {})\n", user_id, ticket.user_id));
        transcript.push_str(&format!("Catégorie : {}\n", ticket.category));
        transcript.push_str(&format!("Ouvert le : {}\n", open_date));
        transcript.push_str(&format!("Fermé le : {}\n", close_date));
        transcript.push_str(&format!("Message Initial : {}\n", ticket.initial_message));
        transcript.push_str("=========================\n\n");

        for msg in &messages {
            if !msg.content.is_empty() {
                let time = msg.timestamp.format("%H:%M:%S");
                transcript.push_str(&format!("[{}] {}: {}\n", time, msg.author.name, msg.content));
            }
        }
        
        let file_name = format!("transcript-{}.txt", ticket.user_id);
        let mut file = File::create(&file_name).await?;
        file.write_all(transcript.as_bytes()).await?;
        
        let log_channel = serenity::ChannelId::new(data.config.channels.log_channel_id);
        let log_embed = serenity::CreateEmbed::new()
            .title("Ticket Fermé")
            .field("Utilisateur", format!("<@{}>", ticket.user_id), true)
            .field("Fermé par", format!("<@{}>", ctx.author().id), true)
            .field("Ouverture", open_date.clone(), true)
            .field("Fermeture", close_date.clone(), true)
            .field("Messages", messages.len().to_string(), true)
            .color(0xe74c3c)
            .timestamp(serenity::Timestamp::now());

        let file_content = tokio::fs::read(&file_name).await?;
        let attachment = serenity::CreateAttachment::bytes(file_content, &file_name);

        log_channel.send_message(&ctx, serenity::CreateMessage::new()
            .embed(log_embed)
            .add_file(attachment)
        ).await?;

        let dm_channel = user_id.create_dm_channel(&ctx).await?;
        dm_channel.send_message(&ctx, serenity::CreateMessage::new().embed(
            serenity::CreateEmbed::new()
                .title("Ticket Fermé")
                .description("Votre ticket a été fermé par l'équipe de support.")
                .field("Date d'ouverture", open_date, true)
                .field("Date de fermeture", close_date, true)
                .field("Votre demande initiale", ticket.initial_message, false)
                .footer(serenity::CreateEmbedFooter::new("Si vous avez besoin d'aide à nouveau, n'hésitez pas à nous recontacter."))
                .color(0xe74c3c)
        )).await?;

        ctx.channel_id().delete(&ctx).await?;
        
        let _ = tokio::fs::remove_file(file_name).await;

    } else {
        ctx.send(poise::CreateReply::default()
            .content("❌ Ce salon n'est pas un ticket actif.")
            .ephemeral(true)
        ).await?;
    }

    Ok(())
}

#[poise::command(slash_command, guild_only)]
pub async fn rename(
    ctx: Context<'_>,
    #[description = "Nouveau nom du salon"] new_name: String,
) -> Result<(), Error> {
    let data = ctx.data();
    let channel_id = ctx.channel_id();

    let is_ticket: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM tickets WHERE channel_id = ?)"
    )
    .bind(channel_id.get() as i64)
    .fetch_one(&data.db)
    .await?;

    if is_ticket {
        channel_id.edit(&ctx, serenity::EditChannel::new().name(new_name)).await?;
        
        ctx.send(poise::CreateReply::default()
            .content("Salon renommé avec succès.")
            .ephemeral(true)
        ).await?;
    } else {
        ctx.send(poise::CreateReply::default()
            .content("Ce salon n'est pas un ticket actif.")
            .ephemeral(true)
        ).await?;
    }

    Ok(())
}
