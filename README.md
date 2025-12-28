
<img width="1500" height="400" alt="vector_bot banner" src="https://github.com/user-attachments/assets/3b693870-9294-4c8e-845f-748f5bd09dd4" />

# Vector Bot

A multifunction Discord bot developed in Rust using the Poise framework and Serenity library.

## Features

### Ticket System

- **Interactive Creation**:
  - Triggered by Direct Message (DM) to the bot.
  - Language selection (French/English).
  - Category selection (Partnership, Recruitment, Support, Other).
  - Automatic creation of a private channel on the server.
  - Automatically configured permissions (Staff + User + Bot).

- **Management**:
  - `/rep <message>`: Allows staff to reply to the user anonymously ("Support: Message").
  - `/close`: Closes the ticket, deletes the channel, sends a full transcript (.txt file) to logs, and notifies the user.
  - `/rename <name>`: Allows renaming the ticket channel.

- **Automation**:
  - Automatic reminder via DM after 24h of inactivity.
  - Automatic closure after 48h of inactivity.

- **Ticket Moderation**:
  - `/blticket @user <reason>`: Blacklists a user (prevents them from opening tickets).
  - `/unblticket @user`: Removes a user from the blacklist.
  - Automatic verification on every direct message.

### Voice System (Join to Create)

- **Automatic Creation**:
  - Joining a "Hub" channel (configured in `config.toml`) creates a temporary voice channel.
  - Channel name: `🔉〢Username`.
  - The creator becomes the owner.

- **Management**:
  - The owner has moderation permissions on their channel (Mute, Deafen, Move, Manage Channels).
  - Automatic deletion of the channel when empty.
  - Automatic ownership transfer if the owner leaves (but others remain).

- **Logs**:
  - Logs for creation, deletion, and ownership transfer in a dedicated channel.
  - Detection and logging if a channel is manually deleted.

### Utility Commands

- `/info`: Displays bot statistics (Uptime, Latency, RAM, CPU, Active tickets, Blacklists).
- `/profil [@user]`: Displays a user's profile (Creation date, Join date, Staff/Client Status, Blacklist Status).
- `/clear <number>`: Deletes a specific number of messages (max 99).
- `/hello`: Basic test command.

## Configuration

The bot is configured via the `config.toml` file:

```toml
[roles]
staff_role_id = 123456789...
client_role_id = 123456789...

[channels]
log_channel_id = 123456789...
voice_log_channel_id = 123456789...
jtc_channel_ids = [123456789..., 987654321...]

[categories]
partnership = 123456789...
recruitment = 123456789...
support = 123456789...
other = 123456789...
voice_category_id = 123456789...
```

## Database

The bot uses SQLite (`database.db`) to store:
- Active tickets.
- Ticket blacklist.
- Temporary voice channels.
- Ticket counters.

## Installation and Launch

1. Clone the repository.
2. Create a `.env` file with:
   ```
   DISCORD_TOKEN=your_token
   DISCORD_GUILD_ID=your_server_id
   ```
   > To find your server ID: Enable Developer Mode in Discord settings (Advanced > Developer Mode), then right-click your server icon > "Copy Server ID".
   
   > **IMPORTANT**: Never share your `.env` file and never commit it to Git (it is already ignored by `.gitignore`).

3. Configure `config.toml`.
4. Run with `cargo run`.

## Important Notes

- **Intents**: This bot currently uses non-privileged intents (`GatewayIntents::non_privileged()`). If you need to read message content (for old-style commands) or detect members joining, you will need to enable "Privileged Gateway Intents" on the [Discord Developer Portal](https://discord.com/developers/applications) and modify `src/main.rs`.

## Documentation

- [Poise Guide](https://github.com/serenity-rs/poise)
- [Serenity Documentation](https://docs.rs/serenity/latest/serenity/)
- [The Rust Book](https://doc.rust-lang.org/book/)

## Common issue on Windows: `linker 'link.exe' not found`

If you encounter the `linker 'link.exe' not found` error during compilation, it means you are missing C++ build tools.

**Solution:**

1. Download and install Visual Studio Build Tools (or Visual Studio Community).
2. During installation, check the box "Desktop development with C++".
3. Let the installation finish and restart your terminal (or PC).
