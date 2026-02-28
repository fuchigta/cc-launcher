use cc_launcher_lib::config::{AppConfig, TerminalType};
use cc_launcher_lib::models::{
    PluginConfig, ScheduleConfig, ScheduleExpression, SubscriptionConfig,
};
use clap::{Parser, Subcommand};
use uuid::Uuid;

#[derive(Parser)]
#[command(name = "cc-launcher-cli", about = "CLI for cc-launcher")]
struct Cli {
    /// Output in JSON format
    #[arg(long, global = true)]
    json: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Manage schedules
    Schedule {
        #[command(subcommand)]
        action: ScheduleAction,
    },
    /// Manage plugins
    Plugin {
        #[command(subcommand)]
        action: PluginAction,
    },
    /// Manage subscriptions
    Subscription {
        #[command(subcommand)]
        action: SubscriptionAction,
    },
    /// Manage configuration
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
}

#[derive(Subcommand)]
enum ScheduleAction {
    /// List all schedules
    List,
    /// Add a new schedule
    Add {
        #[arg(long)]
        name: String,
        /// Cron expression (e.g. "0 9 * * *")
        #[arg(long)]
        cron: Option<String>,
        /// Interval in seconds
        #[arg(long)]
        interval: Option<u64>,
        /// Daily time in HH:MM format
        #[arg(long = "daily-at")]
        daily_at: Option<String>,
        /// Prompt text, or "-" to read from stdin
        #[arg(long)]
        prompt: String,
        #[arg(long)]
        dir: Option<String>,
        #[arg(long = "arg")]
        args: Vec<String>,
    },
    /// Delete a schedule by ID or name
    Delete { id: String },
    /// Enable a schedule
    Enable { id: String },
    /// Disable a schedule
    Disable { id: String },
}

#[derive(Subcommand)]
enum PluginAction {
    /// List all plugins
    List,
    /// Add a new plugin
    Add {
        #[arg(long)]
        name: String,
        #[arg(long)]
        executable: String,
        #[arg(long = "arg")]
        args: Vec<String>,
    },
    /// Delete a plugin by ID or name
    Delete { id: String },
    /// Enable a plugin
    Enable { id: String },
    /// Disable a plugin
    Disable { id: String },
}

#[derive(Subcommand)]
enum SubscriptionAction {
    /// List all subscriptions
    List,
    /// Add a new subscription
    Add {
        #[arg(long)]
        name: String,
        #[arg(long)]
        plugin: String,
        #[arg(long)]
        event: String,
        /// Prompt template text, or "-" to read from stdin
        #[arg(long)]
        template: String,
        #[arg(long)]
        dir: Option<String>,
        #[arg(long = "arg")]
        args: Vec<String>,
    },
    /// Delete a subscription by ID or name
    Delete { id: String },
    /// Enable a subscription
    Enable { id: String },
    /// Disable a subscription
    Disable { id: String },
}

#[derive(Subcommand)]
enum ConfigAction {
    /// Show current configuration
    Show,
    /// Set a configuration value
    Set {
        #[command(subcommand)]
        key: ConfigKey,
    },
}

#[derive(Subcommand)]
enum ConfigKey {
    /// Set global keyboard shortcut (e.g. "Ctrl+Shift+Space")
    Shortcut { value: String },
    /// Set execution timeout in seconds
    Timeout { secs: u64 },
    /// Set terminal type: Auto, Pwsh, PowerShell, Cmd, Wsl
    Terminal { value: String },
}

fn read_prompt(value: &str) -> Result<String, Box<dyn std::error::Error>> {
    if value == "-" {
        use std::io::Read;
        let mut buf = String::new();
        std::io::stdin().read_to_string(&mut buf)?;
        Ok(buf.trim_end_matches(['\r', '\n']).to_string())
    } else {
        Ok(value.to_string())
    }
}

trait Identifiable {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn type_name() -> &'static str;
}

impl Identifiable for ScheduleConfig {
    fn id(&self) -> &str {
        &self.id
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn type_name() -> &'static str {
        "Schedule"
    }
}

impl Identifiable for PluginConfig {
    fn id(&self) -> &str {
        &self.id
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn type_name() -> &'static str {
        "Plugin"
    }
}

impl Identifiable for SubscriptionConfig {
    fn id(&self) -> &str {
        &self.id
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn type_name() -> &'static str {
        "Subscription"
    }
}

fn find_by_id_or_name<T: Identifiable>(items: &[T], id_or_name: &str) -> Result<String, String> {
    let matches: Vec<_> = items
        .iter()
        .filter(|item| {
            item.id() == id_or_name
                || item.name() == id_or_name
                || item.id().starts_with(id_or_name)
        })
        .collect();
    match matches.as_slice() {
        [] => Err(format!("{} not found: {id_or_name}", T::type_name())),
        [item] => Ok(item.id().to_string()),
        _ => Err(format!(
            "Ambiguous ID prefix '{}': matches {}",
            id_or_name,
            matches
                .iter()
                .map(|item| format!("{} ({})", short_id(item.id()), item.name()))
                .collect::<Vec<_>>()
                .join(", ")
        )),
    }
}

fn main() {
    let cli = Cli::parse();
    if let Err(e) = run(cli) {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    match cli.command {
        Commands::Schedule { action } => handle_schedule(action, cli.json),
        Commands::Plugin { action } => handle_plugin(action, cli.json),
        Commands::Subscription { action } => handle_subscription(action, cli.json),
        Commands::Config { action } => handle_config_cmd(action, cli.json),
    }
}

// ---- Schedule ----

fn handle_schedule(action: ScheduleAction, json: bool) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        ScheduleAction::List => {
            let config = AppConfig::load();
            if json {
                println!("{}", serde_json::to_string_pretty(&config.schedules)?);
            } else {
                print_schedules(&config.schedules);
            }
        }
        ScheduleAction::Add {
            name,
            cron,
            interval,
            daily_at,
            prompt,
            dir,
            args,
        } => {
            let expression = match (cron, interval, daily_at) {
                (Some(expr), None, None) => ScheduleExpression::Cron { expression: expr },
                (None, Some(secs), None) => ScheduleExpression::Interval { seconds: secs },
                (None, None, Some(time)) => ScheduleExpression::DailyAt { time },
                _ => {
                    return Err(
                        "Exactly one of --cron, --interval, or --daily-at must be specified".into(),
                    );
                }
            };
            let id = Uuid::new_v4().to_string();
            let schedule = ScheduleConfig {
                id: id.clone(),
                name,
                expression,
                prompt: read_prompt(&prompt)?,
                working_dir: dir,
                claude_args: args,
                enabled: true,
            };
            let mut config = AppConfig::load();
            config.schedules.push(schedule);
            config.save()?;
            if json {
                println!("{{\"id\":\"{id}\"}}");
            } else {
                println!("Added schedule: {id}");
            }
        }
        ScheduleAction::Delete { id } => {
            let mut config = AppConfig::load();
            let resolved = find_by_id_or_name(&config.schedules, &id)?;
            config.schedules.retain(|s| s.id != resolved);
            config.save()?;
            if json {
                println!("{{\"deleted\":\"{resolved}\"}}");
            } else {
                println!("Deleted schedule: {resolved}");
            }
        }
        ScheduleAction::Enable { id } => toggle_schedule(&id, true, json)?,
        ScheduleAction::Disable { id } => toggle_schedule(&id, false, json)?,
    }
    Ok(())
}

fn toggle_schedule(id: &str, enabled: bool, json: bool) -> Result<(), Box<dyn std::error::Error>> {
    let mut config = AppConfig::load();
    let resolved = find_by_id_or_name(&config.schedules, id)?;
    config
        .schedules
        .iter_mut()
        .find(|s| s.id == resolved)
        .unwrap()
        .enabled = enabled;
    config.save()?;
    let state = if enabled { "Enabled" } else { "Disabled" };
    if json {
        println!("{{\"id\":\"{resolved}\",\"enabled\":{enabled}}}");
    } else {
        println!("{state} schedule: {resolved}");
    }
    Ok(())
}

fn print_schedules(schedules: &[ScheduleConfig]) {
    if schedules.is_empty() {
        println!("No schedules configured.");
        return;
    }
    let rows: Vec<Vec<String>> = schedules
        .iter()
        .map(|s| {
            vec![
                short_id(&s.id),
                truncate(&s.name, 20),
                format_expression(&s.expression),
                truncate(&s.prompt, 30),
                yes_no(s.enabled),
            ]
        })
        .collect();
    print_table(&["ID", "NAME", "SCHEDULE", "PROMPT", "ENABLED"], &rows);
}

fn format_expression(expr: &ScheduleExpression) -> String {
    match expr {
        ScheduleExpression::Cron { expression } => format!("cron:{expression}"),
        ScheduleExpression::Interval { seconds } => format!("every {seconds}s"),
        ScheduleExpression::DailyAt { time } => format!("daily@{time}"),
    }
}

// ---- Plugin ----

fn handle_plugin(action: PluginAction, json: bool) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        PluginAction::List => {
            let config = AppConfig::load();
            if json {
                println!("{}", serde_json::to_string_pretty(&config.plugins)?);
            } else {
                print_plugins(&config.plugins);
            }
        }
        PluginAction::Add {
            name,
            executable,
            args,
        } => {
            let id = Uuid::new_v4().to_string();
            let plugin = PluginConfig {
                id: id.clone(),
                name,
                executable,
                args,
                enabled: true,
            };
            let mut config = AppConfig::load();
            config.plugins.push(plugin);
            config.save()?;
            if json {
                println!("{{\"id\":\"{id}\"}}");
            } else {
                println!("Added plugin: {id}");
            }
        }
        PluginAction::Delete { id } => {
            let mut config = AppConfig::load();
            let resolved = find_by_id_or_name(&config.plugins, &id)?;
            config.plugins.retain(|p| p.id != resolved);
            config.save()?;
            if json {
                println!("{{\"deleted\":\"{resolved}\"}}");
            } else {
                println!("Deleted plugin: {resolved}");
            }
        }
        PluginAction::Enable { id } => toggle_plugin(&id, true, json)?,
        PluginAction::Disable { id } => toggle_plugin(&id, false, json)?,
    }
    Ok(())
}

fn toggle_plugin(id: &str, enabled: bool, json: bool) -> Result<(), Box<dyn std::error::Error>> {
    let mut config = AppConfig::load();
    let resolved = find_by_id_or_name(&config.plugins, id)?;
    config
        .plugins
        .iter_mut()
        .find(|p| p.id == resolved)
        .unwrap()
        .enabled = enabled;
    config.save()?;
    let state = if enabled { "Enabled" } else { "Disabled" };
    if json {
        println!("{{\"id\":\"{resolved}\",\"enabled\":{enabled}}}");
    } else {
        println!("{state} plugin: {resolved}");
    }
    Ok(())
}

fn print_plugins(plugins: &[PluginConfig]) {
    if plugins.is_empty() {
        println!("No plugins configured.");
        return;
    }
    let rows: Vec<Vec<String>> = plugins
        .iter()
        .map(|p| {
            vec![
                short_id(&p.id),
                truncate(&p.name, 20),
                truncate(&p.executable, 40),
                yes_no(p.enabled),
            ]
        })
        .collect();
    print_table(&["ID", "NAME", "EXECUTABLE", "ENABLED"], &rows);
}

// ---- Subscription ----

fn handle_subscription(
    action: SubscriptionAction,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        SubscriptionAction::List => {
            let config = AppConfig::load();
            if json {
                println!("{}", serde_json::to_string_pretty(&config.subscriptions)?);
            } else {
                print_subscriptions(&config.subscriptions);
            }
        }
        SubscriptionAction::Add {
            name,
            plugin,
            event,
            template,
            dir,
            args,
        } => {
            let id = Uuid::new_v4().to_string();
            let sub = SubscriptionConfig {
                id: id.clone(),
                name,
                plugin_name: plugin,
                event_type: event,
                prompt_template: read_prompt(&template)?,
                working_dir: dir,
                claude_args: args,
                enabled: true,
            };
            let mut config = AppConfig::load();
            config.subscriptions.push(sub);
            config.save()?;
            if json {
                println!("{{\"id\":\"{id}\"}}");
            } else {
                println!("Added subscription: {id}");
            }
        }
        SubscriptionAction::Delete { id } => {
            let mut config = AppConfig::load();
            let resolved = find_by_id_or_name(&config.subscriptions, &id)?;
            config.subscriptions.retain(|s| s.id != resolved);
            config.save()?;
            if json {
                println!("{{\"deleted\":\"{resolved}\"}}");
            } else {
                println!("Deleted subscription: {resolved}");
            }
        }
        SubscriptionAction::Enable { id } => toggle_subscription(&id, true, json)?,
        SubscriptionAction::Disable { id } => toggle_subscription(&id, false, json)?,
    }
    Ok(())
}

fn toggle_subscription(
    id: &str,
    enabled: bool,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut config = AppConfig::load();
    let resolved = find_by_id_or_name(&config.subscriptions, id)?;
    config
        .subscriptions
        .iter_mut()
        .find(|s| s.id == resolved)
        .unwrap()
        .enabled = enabled;
    config.save()?;
    let state = if enabled { "Enabled" } else { "Disabled" };
    if json {
        println!("{{\"id\":\"{resolved}\",\"enabled\":{enabled}}}");
    } else {
        println!("{state} subscription: {resolved}");
    }
    Ok(())
}

fn print_subscriptions(subs: &[SubscriptionConfig]) {
    if subs.is_empty() {
        println!("No subscriptions configured.");
        return;
    }
    let rows: Vec<Vec<String>> = subs
        .iter()
        .map(|s| {
            vec![
                short_id(&s.id),
                truncate(&s.name, 20),
                truncate(&s.plugin_name, 20),
                truncate(&s.event_type, 20),
                yes_no(s.enabled),
            ]
        })
        .collect();
    print_table(&["ID", "NAME", "PLUGIN", "EVENT", "ENABLED"], &rows);
}

// ---- Config ----

fn handle_config_cmd(action: ConfigAction, json: bool) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        ConfigAction::Show => {
            let config = AppConfig::load();
            if json {
                println!("{}", serde_json::to_string_pretty(&config)?);
            } else {
                print_config(&config);
            }
        }
        ConfigAction::Set { key } => {
            let mut config = AppConfig::load();
            match key {
                ConfigKey::Shortcut { value } => config.shortcut = value,
                ConfigKey::Timeout { secs } => config.timeout_secs = secs,
                ConfigKey::Terminal { value } => config.terminal = parse_terminal_type(&value)?,
            }
            config.save()?;
            if json {
                println!("{{\"ok\":true}}");
            } else {
                println!("Configuration saved.");
            }
        }
    }
    Ok(())
}

fn parse_terminal_type(s: &str) -> Result<TerminalType, Box<dyn std::error::Error>> {
    match s.to_lowercase().as_str() {
        "auto" => Ok(TerminalType::Auto),
        "pwsh" => Ok(TerminalType::Pwsh),
        "powershell" => Ok(TerminalType::PowerShell),
        "cmd" => Ok(TerminalType::Cmd),
        "wsl" => Ok(TerminalType::Wsl),
        _ => Err(format!(
            "Unknown terminal type: {s}. Valid values: Auto, Pwsh, PowerShell, Cmd, Wsl"
        )
        .into()),
    }
}

fn print_config(config: &AppConfig) {
    println!("shortcut:      {}", config.shortcut);
    println!("terminal:      {:?}", config.terminal);
    println!("timeout:       {}s", config.timeout_secs);
    println!(
        "last_dir:      {}",
        config.last_directory.as_deref().unwrap_or("-")
    );
    println!("schedules:     {}", config.schedules.len());
    println!("plugins:       {}", config.plugins.len());
    println!("subscriptions: {}", config.subscriptions.len());
}

// ---- Table formatting ----

fn print_table(headers: &[&str], rows: &[Vec<String>]) {
    let mut widths: Vec<usize> = headers.iter().map(|h| h.len()).collect();
    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            if i < widths.len() {
                widths[i] = widths[i].max(cell.len());
            }
        }
    }
    let header: Vec<String> = headers
        .iter()
        .zip(&widths)
        .map(|(h, w)| format!("{h:<width$}", width = w))
        .collect();
    println!("{}", header.join("  "));
    let sep: Vec<String> = widths.iter().map(|w| "-".repeat(*w)).collect();
    println!("{}", sep.join("  "));
    for row in rows {
        let cells: Vec<String> = row
            .iter()
            .zip(&widths)
            .map(|(c, w)| format!("{c:<width$}", width = w))
            .collect();
        println!("{}", cells.join("  "));
    }
}

fn short_id(id: &str) -> String {
    id.chars().take(8).collect()
}

fn truncate(s: &str, max: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= max {
        s.to_string()
    } else {
        let cut = max.saturating_sub(3);
        format!("{}...", chars[..cut].iter().collect::<String>())
    }
}

fn yes_no(b: bool) -> String {
    if b { "yes" } else { "no" }.to_string()
}
