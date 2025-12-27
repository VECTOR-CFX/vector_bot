use poise::serenity_prelude as serenity;
use crate::Data;
use crate::ticket_system::structs::{TicketState, TicketCategory, TicketInfo};

pub async fn handle_event(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, crate::Error>,
    data: &Data,
) -> Result<(), crate::Error> {
    match event {
        serenity::FullEvent::Message { new_message } => {
            if new_message.author.bot {
                return Ok(());
            }

            if new_message.guild_id.is_none() {
                handle_dm(ctx, new_message, data).await?;
            }
        }
        serenity::FullEvent::InteractionCreate { interaction } => {
            if let serenity::Interaction::Component(component) = interaction {
                handle_component(ctx, component, data).await?;
            }
        }
        _ => {}
    }
    Ok(())
}

async fn handle_dm(
    ctx: &serenity::Context,
    msg: &serenity::Message,
    data: &Data,
) -> Result<(), crate::Error> {
    let user_id = msg.author.id.get();

    {
        let store = data.ticket_store.read().await;
        if let Some(info) = store.blacklist.get(&user_id) {
            let embed = serenity::CreateEmbed::new()
                .title("Accès refusé")
                .description(format!("Vous avez été blacklisté du système de ticket.\n**Raison:** {}", info.reason))
                .color(0xe74c3c);
            
            msg.channel_id.send_message(ctx, serenity::CreateMessage::new().embed(embed)).await?;
            return Ok(());
        }
    }

    let ticket_channel_id = {
        let store = data.ticket_store.read().await;
        store.tickets.get(&user_id).map(|t| t.channel_id)
    };

    if let Some(channel_id_u64) = ticket_channel_id {
        let channel_id = serenity::ChannelId::new(channel_id_u64);
        
        let content = format!("**{}**: {}", msg.author.name, msg.content);
        channel_id.say(ctx, content).await?;
        
        msg.react(ctx, serenity::ReactionType::Unicode("✅".to_string())).await?;

        {
            let mut store = data.ticket_store.write().await;
            if let Some(ticket) = store.tickets.get_mut(&user_id) {
                ticket.last_activity = chrono::Utc::now().timestamp();
                ticket.has_been_reminded = false;
                store.save();
            }
        }
        return Ok(());
    }

    let state = {
        let states = data.ticket_states.read().await;
        states.get(&user_id).cloned()
    };

    match state {
        Some(TicketState::WritingMessage { language, category }) => {
            create_ticket(ctx, msg, data, language, category).await?;
            
            let mut states = data.ticket_states.write().await;
            states.remove(&user_id);
        }
        Some(_) => {

        }
        None => {

            let mut states = data.ticket_states.write().await;
            states.insert(user_id, TicketState::ChoosingLanguage);

            let buttons = vec![
                serenity::CreateButton::new("lang_fr").label("Français").style(serenity::ButtonStyle::Primary),
                serenity::CreateButton::new("lang_en").label("English").style(serenity::ButtonStyle::Secondary),
            ];

            let embed = serenity::CreateEmbed::new()
                .title("Support Vector Bot")
                .description("Please select your language / Veuillez choisir votre langue")
                .color(0x5865F2);

            msg.channel_id.send_message(ctx, serenity::CreateMessage::new()
                .embed(embed)
                .components(vec![serenity::CreateActionRow::Buttons(buttons)])
            ).await?;
        }
    }

    Ok(())
}

async fn handle_component(
    ctx: &serenity::Context,
    component: &serenity::ComponentInteraction,
    data: &Data,
) -> Result<(), crate::Error> {
    let user_id = component.user.id.get();
    let custom_id = &component.data.custom_id;

    let state = {
        let states = data.ticket_states.read().await;
        states.get(&user_id).cloned()
    };

    if custom_id.starts_with("lang_") {
        let lang = if custom_id == "lang_fr" { "FR" } else { "EN" };
        
        {
            let mut states = data.ticket_states.write().await;
            states.insert(user_id, TicketState::ChoosingCategory { language: lang.to_string() });
        }

        let options = vec![
            serenity::CreateSelectMenuOption::new("Partenariat / Partnership", "cat_partnership"),
            serenity::CreateSelectMenuOption::new("Recrutement / Recruitment", "cat_recruitment"),
            serenity::CreateSelectMenuOption::new("Support", "cat_support"),
            serenity::CreateSelectMenuOption::new("Autres / Other", "cat_other"),
        ];

        let select_menu = serenity::CreateSelectMenu::new("category_select", serenity::CreateSelectMenuKind::String { options });
        
        let text = if lang == "FR" { "Veuillez choisir une catégorie :" } else { "Please select a category:" };
        
        component.create_response(ctx, serenity::CreateInteractionResponse::UpdateMessage(
            serenity::CreateInteractionResponseMessage::new()
                .content("")
                .embed(serenity::CreateEmbed::new().description(text).color(0x3498db))
                .components(vec![serenity::CreateActionRow::SelectMenu(select_menu)])
        )).await?;

    } else if custom_id == "category_select" {
        if let serenity::ComponentInteractionDataKind::StringSelect { values } = &component.data.kind {
            if let Some(value) = values.first() {
                let category = match value.as_str() {
                    "cat_partnership" => TicketCategory::Partnership,
                    "cat_recruitment" => TicketCategory::Recruitment,
                    "cat_support" => TicketCategory::Support,
                    _ => TicketCategory::Other,
                };

                let lang = if let Some(TicketState::ChoosingCategory { language }) = state {
                    language
                } else {
                    "FR".to_string()
                };

                {
                    let mut states = data.ticket_states.write().await;
                    states.insert(user_id, TicketState::WritingMessage { 
                        language: lang.clone(), 
                        category: category.clone() 
                    });
                }

                let text = if lang == "FR" { 
                    format!("Vous avez choisi **{}**. Veuillez maintenant décrire votre demande en un seul message.", category.to_string())
                } else { 
                    format!("You chose **{:?}**. Please describe your request in a single message.", category)
                };

                component.create_response(ctx, serenity::CreateInteractionResponse::UpdateMessage(
                    serenity::CreateInteractionResponseMessage::new()
                        .content("")
                        .embed(serenity::CreateEmbed::new().description(text).color(0x2ecc71))
                        .components(vec![]) 
                )).await?;
            }
        }
    }

    Ok(())
}

async fn create_ticket(
    ctx: &serenity::Context,
    msg: &serenity::Message,
    data: &Data,
    language: String,
    category: TicketCategory,
) -> Result<(), crate::Error> {
    let user_id = msg.author.id.get();
    let guild_id = std::env::var("DISCORD_GUILD_ID")?.parse::<u64>()?;
    let guild_id = serenity::GuildId::new(guild_id);

    let count = {
        let mut store = data.ticket_store.write().await;
        let cat_key = format!("{:?}", category);
        let count = store.counts.entry(cat_key.clone()).or_insert(0);
        *count += 1;
        *count
    }; 

    let category_id = match category {
        TicketCategory::Partnership => data.config.categories.partnership,
        TicketCategory::Recruitment => data.config.categories.recruitment,
        TicketCategory::Support => data.config.categories.support,
        TicketCategory::Other => data.config.categories.other,
    };

    let channel_name = format!("{}-{}", msg.author.name, count);
    
    let builder = serenity::CreateChannel::new(channel_name)
        .kind(serenity::ChannelType::Text)
        .category(serenity::ChannelId::new(category_id))
        .topic(format!("Ticket de {} | ID: {}", msg.author.name, user_id));

    let channel = guild_id.create_channel(ctx, builder).await?;

    let embed = serenity::CreateEmbed::new()
        .title(format!("Nouveau Ticket #{}", count))
        .field("Utilisateur", format!("<@{}> ({})", user_id, msg.author.name), true)
        .field("Catégorie", category.to_string(), true)
        .field("Langue", &language, true)
        .field("Message Initial", &msg.content, false)
        .color(0xe67e22)
        .timestamp(serenity::Timestamp::now());

    let content = format!("<@&{}>", data.config.roles.staff_role_id);

    channel.send_message(ctx, serenity::CreateMessage::new()
        .content(content)
        .embed(embed)
    ).await?;

    {
        let mut store = data.ticket_store.write().await;
        store.tickets.insert(user_id, TicketInfo {
            user_id,
            channel_id: channel.id.get(),
            category: category.clone(),
            created_at: chrono::Utc::now().timestamp(),
            initial_message: msg.content.clone(),
            last_activity: chrono::Utc::now().timestamp(),
            has_been_reminded: false,
        });
        store.save();
    }

    let confirmation_message = if language == "FR" {
        "Votre ticket a été créé avec succès ! Un membre du staff va vous répondre bientôt."
    } else {
        "Your ticket has been successfully created ! A staff member will answer you shortly."
    };
    
    msg.channel_id.say(ctx, confirmation_message).await?;

    Ok(())
}
