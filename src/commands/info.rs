use crate::{Context, Error};
use poise::serenity_prelude as serenity;
use sysinfo::{System, get_current_pid};

#[poise::command(slash_command)]
pub async fn info(ctx: Context<'_>) -> Result<(), Error> {
    let data = ctx.data();
    let start_time = data.start_time;
    let uptime = std::time::Instant::now().duration_since(start_time);
    
    let days = uptime.as_secs() / 86400;
    let hours = (uptime.as_secs() % 86400) / 3600;
    let minutes = (uptime.as_secs() % 3600) / 60;
    let seconds = uptime.as_secs() % 60;
    let uptime_str = format!("{}j {}h {}m {}s", days, hours, minutes, seconds);

    let (bot_memory_usage, bot_cpu_usage) = {
        let mut sys = data.system_info.lock().unwrap();
        sys.refresh_all();
        
        let pid = get_current_pid().ok();
        if let Some(pid) = pid {
            if let Some(process) = sys.process(pid) {
                let memory = process.memory() / 1024 / 1024; 
                let cpu = process.cpu_usage();
                (memory, cpu)
            } else {
                (0, 0.0)
            }
        } else {
            (0, 0.0)
        }
    };

    let os_name = System::name().unwrap_or_else(|| "Inconnu".to_string());
    let os_version = System::os_version().unwrap_or_else(|| "Inconnue".to_string());

    let latency = ctx.ping().await;

    let command_count = ctx.framework().options().commands.len();

    ctx.send(poise::CreateReply::default().embed(serenity::CreateEmbed::new()
        .title("Informations & Statistiques")
        .color(0x3498db) 
        .field("Uptime", uptime_str, true)
        .field("Latence API", format!("{:.2} ms", latency.as_millis()), true)
        .field("Commandes", command_count.to_string(), true)
        .field("Système", format!("{} {}", os_name, os_version), true)
        .field("Mémoire (Bot)", format!("{} MB", bot_memory_usage), true)
        .field("CPU (Bot)", format!("{:.2}%", bot_cpu_usage), true)
        .field("Stack Technique", "Language: `Rust` \nFramework: `Poise`\nLib: `Serenity`", true)
        .footer(serenity::CreateEmbedFooter::new("VECTOR • Développé avec ❤️ en Rust"))
        .timestamp(serenity::Timestamp::now())
    )).await?;

    Ok(())
}
