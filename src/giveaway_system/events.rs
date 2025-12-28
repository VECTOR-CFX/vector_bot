use poise::serenity_prelude as serenity;
use crate::Data;
use rand::prelude::IndexedRandom;

pub async fn handle_event(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, crate::Error>,
    data: &Data,
) -> Result<(), crate::Error> {
    if let serenity::FullEvent::InteractionCreate { interaction } = event {
        match interaction {
            serenity::Interaction::Component(component) => {
                handle_component(ctx, component, data).await?;
            }
            serenity::Interaction::Modal(modal) => {
                handle_modal(ctx, modal, data).await?;
            }
            _ => {}
        }
    }
    Ok(())
}

async fn handle_component(
    ctx: &serenity::Context,
    component: &serenity::ComponentInteraction,
    data: &Data,
) -> Result<(), crate::Error> {
    let custom_id = &component.data.custom_id;
    let channel_id = component.channel_id;
    let user_id = component.user.id;

    if custom_id == "giveaway_create_btn" {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM giveaways WHERE channel_id = ? AND status = 'active')"
        )
        .bind(channel_id.get() as i64)
        .fetch_one(&data.db)
        .await?;

        if exists {
            component.create_response(ctx, serenity::CreateInteractionResponse::Message(
                serenity::CreateInteractionResponseMessage::new()
                    .content("Un giveaway est déjà en cours dans ce salon.")
                    .ephemeral(true)
            )).await?;
            return Ok(());
        }

        let modal = serenity::CreateModal::new("giveaway_create_modal", "Créer un Giveaway")
            .components(vec![
                serenity::CreateActionRow::InputText(serenity::CreateInputText::new(
                    serenity::InputTextStyle::Short, "Titre", "title"
                )),
                serenity::CreateActionRow::InputText(serenity::CreateInputText::new(
                    serenity::InputTextStyle::Paragraph, "Description", "description"
                )),
                serenity::CreateActionRow::InputText(serenity::CreateInputText::new(
                    serenity::InputTextStyle::Short, "Durée (ex: 1j, 1h, 30m)", "duration"
                ).placeholder("1j ou 1h ou 30m")),
                serenity::CreateActionRow::InputText(serenity::CreateInputText::new(
                    serenity::InputTextStyle::Short, "Récompense", "reward"
                )),
                serenity::CreateActionRow::InputText(serenity::CreateInputText::new(
                    serenity::InputTextStyle::Short, "Nombre de gagnants", "winners"
                ).value("1")),
            ]);

        component.create_response(ctx, serenity::CreateInteractionResponse::Modal(modal)).await?;
    } else if custom_id == "giveaway_delete_btn" {
        let giveaway: Option<(i64, i64, String)> = sqlx::query_as(
            "SELECT message_id, host_id, title FROM giveaways WHERE channel_id = ? AND status = 'active'"
        )
        .bind(channel_id.get() as i64)
        .fetch_optional(&data.db)
        .await?;

        if let Some((message_id, host_id, title)) = giveaway {
            sqlx::query("DELETE FROM giveaways WHERE message_id = ?")
                .bind(message_id)
                .execute(&data.db)
                .await?;

            sqlx::query("DELETE FROM giveaway_participants WHERE giveaway_message_id = ?")
                .bind(message_id)
                .execute(&data.db)
                .await?;

            let _ = channel_id.delete_message(ctx, serenity::MessageId::new(message_id as u64)).await;

            component.create_response(ctx, serenity::CreateInteractionResponse::Message(
                serenity::CreateInteractionResponseMessage::new()
                    .content("Giveaway supprimé avec succès.")
                    .ephemeral(true)
            )).await?;

            let log_channel = serenity::ChannelId::new(data.config.channels.giveaway_log_channel_id);
            let embed = serenity::CreateEmbed::new()
                .title("Giveaway Supprimé")
                .field("Titre", title, false)
                .field("Supprimé par", format!("<@{}>", user_id), true)
                .field("Créé par", format!("<@{}>", host_id), true)
                .color(0xe74c3c)
                .timestamp(serenity::Timestamp::now());
            
            let _ = log_channel.send_message(ctx, serenity::CreateMessage::new().embed(embed)).await;

        } else {
            component.create_response(ctx, serenity::CreateInteractionResponse::Message(
                serenity::CreateInteractionResponseMessage::new()
                    .content("Aucun giveaway actif dans ce salon.")
                    .ephemeral(true)
            )).await?;
        }
    } else if custom_id == "giveaway_join" {
        let message_id = component.message.id.get() as i64;
        
        let active: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM giveaways WHERE message_id = ? AND status = 'active')"
        )
        .bind(message_id)
        .fetch_one(&data.db)
        .await?;

        if !active {
            component.create_response(ctx, serenity::CreateInteractionResponse::Message(
                serenity::CreateInteractionResponseMessage::new()
                    .content("Ce giveaway est terminé.")
                    .ephemeral(true)
            )).await?;
            return Ok(());
        }

        let user_id_i64 = user_id.get() as i64;

        let joined: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM giveaway_participants WHERE giveaway_message_id = ? AND user_id = ?)"
        )
        .bind(message_id)
        .bind(user_id_i64)
        .fetch_one(&data.db)
        .await?;

        if joined {
            sqlx::query("DELETE FROM giveaway_participants WHERE giveaway_message_id = ? AND user_id = ?")
                .bind(message_id)
                .bind(user_id_i64)
                .execute(&data.db)
                .await?;
            
            component.create_response(ctx, serenity::CreateInteractionResponse::Message(
                serenity::CreateInteractionResponseMessage::new()
                    .content("Participation retirée.")
                    .ephemeral(true)
            )).await?;
        } else {
            sqlx::query("INSERT INTO giveaway_participants (giveaway_message_id, user_id) VALUES (?, ?)")
                .bind(message_id)
                .bind(user_id_i64)
                .execute(&data.db)
                .await?;
            
            component.create_response(ctx, serenity::CreateInteractionResponse::Message(
                serenity::CreateInteractionResponseMessage::new()
                    .content("Participation enregistrée !")
                    .ephemeral(true)
            )).await?;
        }

        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM giveaway_participants WHERE giveaway_message_id = ?"
        )
        .bind(message_id)
        .fetch_one(&data.db)
        .await?;

        let mut message = component.message.clone();
        if let Some(embed) = message.embeds.first().cloned() {
            let new_footer = serenity::CreateEmbedFooter::new(format!("Participants: {}", count));
            let new_embed = serenity::CreateEmbed::from(embed).footer(new_footer);
            
            message.edit(ctx, serenity::EditMessage::new().embed(new_embed)).await?;
        }
    }

    Ok(())
}

async fn handle_modal(
    ctx: &serenity::Context,
    modal: &serenity::ModalInteraction,
    data: &Data,
) -> Result<(), crate::Error> {
    if modal.data.custom_id != "giveaway_create_modal" {
        return Ok(());
    }

    let mut title = String::new();
    let mut description = String::new();
    let mut duration_str = String::new();
    let mut reward = String::new();
    let mut winners_str = String::new();

    for row in &modal.data.components {
        if let serenity::ActionRowComponent::InputText(input) = &row.components[0] {
            match input.custom_id.as_str() {
                "title" => title = input.value.clone().unwrap_or_default(),
                "description" => description = input.value.clone().unwrap_or_default(),
                "duration" => duration_str = input.value.clone().unwrap_or_default(),
                "reward" => reward = input.value.clone().unwrap_or_default(),
                "winners" => winners_str = input.value.clone().unwrap_or_default(),
                _ => {}
            }
        }
    }

    let duration_secs = match parse_duration(&duration_str) {
        Some(s) => s,
        None => {
            modal.create_response(ctx, serenity::CreateInteractionResponse::Message(
                serenity::CreateInteractionResponseMessage::new()
                    .content("Format de durée invalide. Utilisez 1j, 1h, 30m.")
                    .ephemeral(true)
            )).await?;
            return Ok(());
        }
    };

    let winner_count = winners_str.parse::<i32>().unwrap_or(1);
    let end_time = chrono::Utc::now().timestamp() + duration_secs;

    let embed = serenity::CreateEmbed::new()
        .title(&title)
        .description(&description)
        .field("Récompense", &reward, true)
        .field("Gagnants", winner_count.to_string(), true)
        .field("Fin", format!("<t:{}:R>", end_time), true)
        .color(0x3498db)
        .footer(serenity::CreateEmbedFooter::new("Participants: 0"));

    let button = serenity::CreateButton::new("giveaway_join")
        .label("Participer")
        .style(serenity::ButtonStyle::Primary);

    modal.create_response(ctx, serenity::CreateInteractionResponse::Message(
        serenity::CreateInteractionResponseMessage::new()
            .embed(embed)
            .components(vec![serenity::CreateActionRow::Buttons(vec![button])])
    )).await?;

    let message = modal.get_response(ctx).await?;
    let message_id = message.id.get() as i64;
    let channel_id = modal.channel_id.get() as i64;
    let host_id = modal.user.id.get() as i64;

    sqlx::query(
        "INSERT INTO giveaways (message_id, channel_id, host_id, title, description, reward, winner_count, end_time, status)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, 'active')"
    )
    .bind(message_id)
    .bind(channel_id)
    .bind(host_id)
    .bind(&title)
    .bind(&description)
    .bind(&reward)
    .bind(winner_count)
    .bind(end_time)
    .execute(&data.db)
    .await?;

    let log_channel = serenity::ChannelId::new(data.config.channels.giveaway_log_channel_id);
    let log_embed = serenity::CreateEmbed::new()
        .title("Nouveau Giveaway Créé")
        .field("Titre", &title, false)
        .field("Créateur", format!("<@{}>", host_id), true)
        .field("Salon", format!("<#{}>", channel_id), true)
        .field("Récompense", &reward, true)
        .field("Durée", &duration_str, true)
        .field("Gagnants", winner_count.to_string(), true)
        .color(0x2ecc71)
        .timestamp(serenity::Timestamp::now());
    
    let _ = log_channel.send_message(ctx, serenity::CreateMessage::new().embed(log_embed)).await;

    Ok(())
}

fn parse_duration(input: &str) -> Option<i64> {
    let mut total_seconds = 0;
    let parts: Vec<&str> = input.split(',').collect();
    
    for part in parts {
        let part = part.trim().to_lowercase();
        if part.is_empty() { continue; }
        
        let val_str: String = part.chars().filter(|c| c.is_digit(10)).collect();
        if val_str.is_empty() { continue; }
        let val: i64 = val_str.parse().ok()?;
        
        let unit_str: String = part.chars().filter(|c| !c.is_digit(10) && !c.is_whitespace()).collect();
        
        if unit_str.starts_with('j') || unit_str.starts_with('d') {
            total_seconds += val * 86400;
        } else if unit_str.starts_with('h') {
            total_seconds += val * 3600;
        } else if unit_str.starts_with('m') {
            total_seconds += val * 60;
        } else if unit_str.starts_with('s') {
            total_seconds += val;
        }
    }
    
    if total_seconds == 0 { return None; }
    Some(total_seconds)
}

pub async fn check_giveaways(db: &sqlx::Pool<sqlx::Sqlite>, http: &serenity::Http, log_channel_id: u64) {
    let now = chrono::Utc::now().timestamp();
    
    let ended_giveaways: Vec<(i64, i64, i64, String, String, String, i32, i64)> = sqlx::query_as(
        "SELECT message_id, channel_id, host_id, title, description, reward, winner_count, end_time FROM giveaways WHERE status = 'active' AND end_time <= ?"
    )
    .bind(now)
    .fetch_all(db)
    .await
    .unwrap_or_default();

    for (message_id, channel_id, _host_id, title, description, reward, winner_count, end_time) in ended_giveaways {
        let _ = sqlx::query("UPDATE giveaways SET status = 'ended' WHERE message_id = ?")
            .bind(message_id)
            .execute(db)
            .await;

        let participants: Vec<i64> = sqlx::query_scalar(
            "SELECT user_id FROM giveaway_participants WHERE giveaway_message_id = ?"
        )
        .bind(message_id)
        .fetch_all(db)
        .await
        .unwrap_or_default();

        let channel = serenity::ChannelId::new(channel_id as u64);
        let message_id_serenity = serenity::MessageId::new(message_id as u64);

        if let Ok(mut msg) = channel.message(http, message_id_serenity).await {
            let new_embed = serenity::CreateEmbed::new()
                .title(format!("Giveaway Terminé : {}", title))
                .description(description)
                .field("Récompense", reward.clone(), true)
                .field("Gagnants", winner_count.to_string(), true)
                .field("Fin", format!("<t:{}:f>", end_time), true)
                .color(0x95a5a6)
                .footer(serenity::CreateEmbedFooter::new(format!("Participants: {}", participants.len())));
            
            let _ = msg.edit(http, serenity::EditMessage::new()
                .embed(new_embed)
                .components(vec![])
            ).await;
        }

        if participants.is_empty() {
            let _ = channel.say(http, format!("Le giveaway **{}** est terminé. Aucun participant.", title)).await;
            continue;
        }

        let winners: Vec<_> = {
            let mut rng = rand::rng();
            participants
                .choose_multiple(&mut rng, winner_count as usize)
                .cloned()
                .collect()
        };

        let winner_mentions: Vec<String> = winners.iter().map(|id| format!("<@{}>", id)).collect();
        let winners_text = winner_mentions.join(", ");

        let win_embed = serenity::CreateEmbed::new()
            .title("Félicitations !")
            .description(format!("Le giveaway **{}** est terminé !\n\n**Gagnant(s):** {}\n**Récompense:** {}", title, winners_text, reward))
            .color(0xf1c40f)
            .timestamp(serenity::Timestamp::now());

        let _ = channel.send_message(http, serenity::CreateMessage::new().content(&winners_text).embed(win_embed)).await;

        for winner_id in &winners {
            let user = serenity::UserId::new(*winner_id as u64);
            if let Ok(dm) = user.create_dm_channel(http).await {
                let _ = dm.say(http, format!("Bravo ! Vous avez gagné le giveaway **{}** pour **{}**. Un staff vous contactera bientôt.", title, reward)).await;
            }
        }

        let log_channel = serenity::ChannelId::new(log_channel_id);
        let log_embed = serenity::CreateEmbed::new()
            .title("Giveaway Terminé")
            .field("Titre", title, false)
            .field("Gagnant(s)", winners_text, false)
            .field("Participants", participants.len().to_string(), true)
            .color(0xf1c40f)
            .timestamp(serenity::Timestamp::now());
        
        let _ = log_channel.send_message(http, serenity::CreateMessage::new().embed(log_embed)).await;
    }
}
