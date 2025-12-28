use poise::serenity_prelude as serenity;
use crate::Data;

pub async fn handle_event(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, crate::Error>,
    data: &Data,
) -> Result<(), crate::Error> {
    match event {
        serenity::FullEvent::VoiceStateUpdate { old, new } => {
            handle_voice_update(ctx, old.as_ref(), new, data).await?;
        }
        serenity::FullEvent::ChannelDelete { channel, .. } => {
            handle_channel_delete(ctx, channel, data).await?;
        }
        _ => {}
    }
    Ok(())
}

async fn handle_channel_delete(
    ctx: &serenity::Context,
    channel: &serenity::GuildChannel,
    data: &Data,
) -> Result<(), crate::Error> {
    let is_managed = {
        let store = data.voice_store.read().await;
        store.channels.contains_key(&channel.id.get())
    };

    if is_managed {
        let owner_id = {
            let mut store = data.voice_store.write().await;
            let owner = store.channels.remove(&channel.id.get());
            store.save();
            owner
        };

        if let Some(uid) = owner_id {
            let log_channel = serenity::ChannelId::new(data.config.channels.voice_log_channel_id);
            let embed = serenity::CreateEmbed::new()
                .title("Vocal Supprimé Manuellement")
                .description(format!("Le salon vocal <#{}> a été supprimé (probablement par son propriétaire ou un admin).", channel.id))
                .field("Ancien Propriétaire", format!("<@{}>", uid), true)
                .color(0xe74c3c)
                .timestamp(serenity::Timestamp::now());
            
            log_channel.send_message(ctx, serenity::CreateMessage::new().embed(embed)).await?;
        }
    }

    Ok(())
}

async fn handle_voice_update(
    ctx: &serenity::Context,
    old: Option<&serenity::VoiceState>,
    new: &serenity::VoiceState,
    data: &Data,
) -> Result<(), crate::Error> {
    if let Some(channel_id) = new.channel_id {
        if data.config.channels.jtc_channel_ids.contains(&channel_id.get()) {
            create_voice_channel(ctx, new, data).await?;
        }
    }

    if let Some(old_state) = old {
        if let Some(channel_id) = old_state.channel_id {
            let is_temp_channel = {
                let store = data.voice_store.read().await;
                store.channels.contains_key(&channel_id.get())
            };

            if is_temp_channel {
                let channel = channel_id.to_channel(ctx).await?.guild().unwrap();
                let members = channel.members(ctx)?;
                
                let human_members: Vec<_> = members.iter().filter(|m| !m.user.bot).collect();

                if human_members.is_empty() {
                    delete_voice_channel(ctx, channel_id, data).await?;
                } else {
                    let user_id = old_state.user_id;
                    let owner_id = {
                        let store = data.voice_store.read().await;
                        *store.channels.get(&channel_id.get()).unwrap()
                    };

                    if user_id.get() == owner_id {
                        if let Some(new_owner) = human_members.first() {
                            transfer_ownership(ctx, channel_id, new_owner.user.id, data).await?;
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

async fn create_voice_channel(
    ctx: &serenity::Context,
    state: &serenity::VoiceState,
    data: &Data,
) -> Result<(), crate::Error> {
    let guild_id = state.guild_id.unwrap();
    let user = state.member.as_ref().unwrap().user.clone();
    
    let channel_name = format!("🔉〢{}", user.name);

    let permissions = vec![
        serenity::PermissionOverwrite {
            allow: serenity::Permissions::CONNECT | serenity::Permissions::SPEAK | serenity::Permissions::USE_VAD,
            deny: serenity::Permissions::empty(),
            kind: serenity::PermissionOverwriteType::Role(serenity::RoleId::new(guild_id.get())),
        },
        serenity::PermissionOverwrite {
            allow: serenity::Permissions::MANAGE_CHANNELS | serenity::Permissions::MUTE_MEMBERS | serenity::Permissions::DEAFEN_MEMBERS | serenity::Permissions::MOVE_MEMBERS,
            deny: serenity::Permissions::empty(),
            kind: serenity::PermissionOverwriteType::Member(user.id),
        },
        serenity::PermissionOverwrite {
            allow: serenity::Permissions::all(),
            deny: serenity::Permissions::empty(),
            kind: serenity::PermissionOverwriteType::Member(ctx.cache.current_user().id),
        },
    ];

    let builder = serenity::CreateChannel::new(channel_name)
        .kind(serenity::ChannelType::Voice)
        .category(serenity::ChannelId::new(data.config.categories.voice_category_id))
        .permissions(permissions);

    let channel = guild_id.create_channel(ctx, builder).await?;

    guild_id.edit_member(ctx, user.id, serenity::EditMember::new().voice_channel(channel.id)).await?;

    {
        let mut store = data.voice_store.write().await;
        store.channels.insert(channel.id.get(), user.id.get());
        store.save();
    }

    let log_channel = serenity::ChannelId::new(data.config.channels.voice_log_channel_id);
    let embed = serenity::CreateEmbed::new()
        .title("Vocal Créé")
        .description(format!("**Propriétaire :** <@{}>\n**Salon :** <#{}>", user.id, channel.id))
        .color(0x2ecc71)
        .timestamp(serenity::Timestamp::now());
    
    log_channel.send_message(ctx, serenity::CreateMessage::new().embed(embed)).await?;

    Ok(())
}

async fn delete_voice_channel(
    ctx: &serenity::Context,
    channel_id: serenity::ChannelId,
    data: &Data,
) -> Result<(), crate::Error> {
    let owner_id = {
        let mut store = data.voice_store.write().await;
        let owner = store.channels.remove(&channel_id.get());
        store.save();
        owner
    };

    channel_id.delete(ctx).await?;

    if let Some(uid) = owner_id {
        let log_channel = serenity::ChannelId::new(data.config.channels.voice_log_channel_id);
        let embed = serenity::CreateEmbed::new()
            .title("Vocal Supprimé")
            .description(format!("**Ancien Propriétaire :** <@{}>\n**Salon ID :** {}", uid, channel_id))
            .color(0xe74c3c)
            .timestamp(serenity::Timestamp::now());
        
        log_channel.send_message(ctx, serenity::CreateMessage::new().embed(embed)).await?;
    }

    Ok(())
}

async fn transfer_ownership(
    ctx: &serenity::Context,
    channel_id: serenity::ChannelId,
    new_owner_id: serenity::UserId,
    data: &Data,
) -> Result<(), crate::Error> {
    
    let permissions = serenity::PermissionOverwrite {
        allow: serenity::Permissions::MANAGE_CHANNELS | serenity::Permissions::MUTE_MEMBERS | serenity::Permissions::DEAFEN_MEMBERS | serenity::Permissions::MOVE_MEMBERS,
        deny: serenity::Permissions::empty(),
        kind: serenity::PermissionOverwriteType::Member(new_owner_id),
    };

    channel_id.create_permission(ctx, permissions).await?;
    
    {
        let mut store = data.voice_store.write().await;
        store.channels.insert(channel_id.get(), new_owner_id.get());
        store.save();
    }
    
    let log_channel = serenity::ChannelId::new(data.config.channels.voice_log_channel_id);
    let embed = serenity::CreateEmbed::new()
        .title("Transfert de propriété")
        .description(format!("**Nouveau Propriétaire :** <@{}>\n**Salon :** <#{}>", new_owner_id, channel_id))
        .color(0xf1c40f)
        .timestamp(serenity::Timestamp::now());
    
    log_channel.send_message(ctx, serenity::CreateMessage::new().embed(embed)).await?;

    Ok(())
}
