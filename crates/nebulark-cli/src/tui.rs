use console::{style, Term};
use dialoguer::{theme::ColorfulTheme, Select, Input};
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

fn theme() -> ColorfulTheme {
    ColorfulTheme::default()
}

pub async fn run_menu(config_path: &str) -> anyhow::Result<()> {
    let term = Term::stdout();

    loop {
        term.clear_screen()?;
        print_banner();

        let connected = if crate::daemon::socket_path().exists() {
            matches!(
        crate::commands::status_check().await,
        Ok(true)
    )
        } else {
            false
        };
        let status_str = if connected {
            style("● Connected").green().bold().to_string()
        } else {
            style("○ Disconnected").dim().to_string()
        };
        println!("  Status: {}\n", status_str);

        let items: Vec<&str> = if connected {
            vec![
                "Disconnect",
                "Status / Stats",
                "─────────────",
                "Generate WARP conf",
                "Import profile",
                "List profiles",
                "─────────────",
                "Exit",
            ]
        } else {
            vec![
                "Connect",
                "─────────────",
                "Generate WARP conf",
                "Import profile",
                "List profiles",
                "─────────────",
                "Exit",
            ]
        };

        let selection = Select::with_theme(&theme())
            .with_prompt("Select action")
            .items(&items)
            .default(0)
            .interact_on_opt(&term)?;

        match selection {
            None => break,
            Some(idx) => {
                let item = items[idx];
                if item.starts_with('─') {
                    continue;
                }
                match item {
                    "Connect"        => menu_connect(config_path).await?,
                    "Disconnect"     => menu_disconnect().await?,
                    "Status / Stats" => menu_status().await?,
                    "Generate WARP conf" => menu_warp(config_path).await?,
                    "Import profile" => menu_import(config_path).await?,
                    "List profiles"  => menu_list(config_path)?,
                    "Exit"           => break,
                    _ => {}
                }
            }
        }
    }

    Ok(())
}

async fn menu_connect(config_path: &str) -> anyhow::Result<()> {
    use nebulark_core::profiles::ProfileManager;

    let mgr = ProfileManager::load(config_path)?;
    let profiles = mgr.profiles();

    if profiles.is_empty() {
        println!(
            "\n  {} No profiles found. Import a .conf file first.\n",
            style("!").yellow()
        );
        pause();
        return Ok(());
    }

    let names: Vec<&str> = profiles.iter().map(|p| p.name.as_str()).collect();

    let selection = Select::with_theme(&theme())
        .with_prompt("Select profile")
        .items(&names)
        .default(0)
        .interact_opt()?;

    if let Some(idx) = selection {
        let name = names[idx];
        println!();
        let pb = spinner(&format!("Connecting to {}...", style(name).cyan()));
        match crate::commands::connect(config_path, name).await {
            Ok(_) => {
                pb.finish_with_message(format!("{} Connected to {}", style("✓").green(), style(name).cyan()));
            }
            Err(e) => {
                pb.finish_with_message(format!("{} Failed: {e}", style("✗").red()));
            }
        }
        pause();
    }

    Ok(())
}

async fn menu_disconnect() -> anyhow::Result<()> {
    println!();
    let pb = spinner("Disconnecting...");
    match crate::commands::disconnect().await {
        Ok(_) => pb.finish_with_message(format!("{} Disconnected", style("✓").green())),
        Err(e) => pb.finish_with_message(format!("{} Failed: {e}", style("✗").red())),
    }
    pause();
    Ok(())
}

async fn menu_status() -> anyhow::Result<()> {
    println!();
    match crate::commands::status().await {
        Ok(_) => {}
        Err(e) => println!("  {} {e}", style("Error:").red()),
    }
    pause();
    Ok(())
}

async fn menu_warp(config_path: &str) -> anyhow::Result<()> {
    println!();
    println!("  {} Получение WARP конфига\n", style("◈").cyan());

    let methods = vec![
        "Встроенный генератор (Cloudflare API напрямую)",
        "Открыть сайт в браузере → импортировать скачанный файл",
    ];

    let choice = Select::with_theme(&theme())
        .with_prompt("Способ")
        .items(&methods)
        .default(0)
        .interact()?;

    match choice {
        0 => menu_warp_builtin(config_path).await?,
        1 => menu_warp_browser(config_path).await?,
        _ => {}
    }

    Ok(())
}

async fn menu_warp_browser(config_path: &str) -> anyhow::Result<()> {
    let url = "https://warp-generator.github.io/";

    println!(
        "\n  {} Открываю {} в браузере...",
        style("→").cyan(),
        style(url).underlined()
    );

    // xdg-open на Linux, start на Windows
    #[cfg(target_os = "linux")]
    let _ = std::process::Command::new("xdg-open").arg(url).spawn();
    #[cfg(target_os = "windows")]
    let _ = std::process::Command::new("cmd").args(["/c", "start", url]).spawn();

    println!("  {} Скачайте конфиг на сайте (кнопка AmneziaWG → AWG 2.0)", style("1.").dim());
    println!("  {} Укажите путь к скачанному файлу ниже\n", style("2.").dim());

    let path: String = Input::with_theme(&theme())
        .with_prompt("Путь к .conf файлу (Enter для отмены)")
        .allow_empty(true)
        .interact_text()?;

    if path.trim().is_empty() {
        return Ok(());
    }

    let path = path.trim();
    if !std::path::Path::new(path).exists() {
        println!("  {} Файл не найден: {path}", style("✗").red());
        pause();
        return Ok(());
    }

    let default_name = std::path::Path::new(path)
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let name: String = Input::with_theme(&theme())
        .with_prompt("Имя профиля")
        .default(default_name)
        .interact_text()?;

    match crate::commands::import(config_path, path, Some(&name)).await {
        Ok(_) => println!("\n  {} Профиль '{}' импортирован", style("✓").green(), style(&name).cyan()),
        Err(e) => println!("  {} {e}", style("✗").red()),
    }

    pause();
    Ok(())
}

async fn menu_warp_builtin(config_path: &str) -> anyhow::Result<()> {
    use crate::warp;

    // выбор пресета
    let preset_names: Vec<&str> = warp::PRESETS.iter().map(|p| p.name).collect();
    let preset_idx = Select::with_theme(&theme())
        .with_prompt("Вариант AWG 2.0")
        .items(&preset_names)
        .default(0)
        .interact()?;

    // выбор endpoint
    let ep_labels: Vec<&str> = warp::ENDPOINTS.iter().map(|(l, _)| *l).collect();
    let ep_idx = Select::with_theme(&theme())
        .with_prompt("Сервер")
        .items(&ep_labels)
        .default(0)
        .interact()?;

    // выбор DNS
    let dns_labels: Vec<&str> = warp::DNS_OPTIONS.iter().map(|(l, _)| *l).collect();
    let dns_idx = Select::with_theme(&theme())
        .with_prompt("DNS")
        .items(&dns_labels)
        .default(0)
        .interact()?;

    let mtu_str: String = Input::with_theme(&theme())
        .with_prompt("MTU")
        .default("1280".into())
        .interact_text()?;
    let mtu: u16 = mtu_str.parse().unwrap_or(1280);

    let ka_str: String = Input::with_theme(&theme())
        .with_prompt("PersistentKeepalive")
        .default("25".into())
        .interact_text()?;
    let ka: u16 = ka_str.parse().unwrap_or(25);

    println!();
    let pb = spinner("Регистрация в Cloudflare WARP API...");

    let ep = warp::ENDPOINTS[ep_idx].1;
    let ep_override = if ep_idx == 0 { None } else { Some(ep) };
    let dns = warp::DNS_OPTIONS[dns_idx].1;

    match warp::generate(&warp::PRESETS[preset_idx], ep_override, dns, mtu, ka).await {
        Ok(generated) => {
            pb.finish_with_message(format!("{} Конфиг получен", style("✓").green()));

            let tmp = std::env::temp_dir().join(format!("{}.conf", generated.profile_name));
            std::fs::write(&tmp, &generated.conf)?;

            match crate::commands::import(config_path, tmp.to_str().unwrap(), Some(&generated.profile_name)).await {
                Ok(_) => println!(
                    "\n  {} Профиль '{}' сохранён — запустите: nebulark connect {}",
                    style("✓").green(),
                    style(&generated.profile_name).cyan(),
                    generated.profile_name,
                ),
                Err(e) => println!("  {} {e}", style("✗").red()),
            }
            let _ = std::fs::remove_file(tmp);
        }
        Err(e) => {
            pb.finish_with_message(format!("{} Ошибка: {e}", style("✗").red()));
            println!("\n  Попробуй вариант 2 — через браузер");
        }
    }

    pause();
    Ok(())
}

async fn menu_import(config_path: &str) -> anyhow::Result<()> {
    println!();
    let path: String = Input::with_theme(&theme())
        .with_prompt("Path to .conf file")
        .interact_text()?;

    let path = path.trim().to_string();

    if let Some(parent) = std::path::Path::new(config_path).parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let content = if std::path::Path::new(&path).exists() {
        std::fs::read_to_string(&path)
            .or_else(|_| {
                read_with_sudo(&path)
            })?
    } else {
        match read_with_sudo(&path) {
            Ok(c) => c,
            Err(_) => {
                println!("  {} File not found: {path}", style("✗").red());
                pause();
                return Ok(());
            }
        }
    };

    let tmp = std::env::temp_dir().join("nebulark_import_tmp.conf");
    std::fs::write(&tmp, &content)?;

    let default_name = std::path::Path::new(&path)
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let name: String = Input::with_theme(&theme())
        .with_prompt("Profile name")
        .default(default_name)
        .interact_text()?;

    match crate::commands::import(config_path, tmp.to_str().unwrap(), Some(&name)).await {
        Ok(_) => println!("\n  {} Profile '{}' imported", style("✓").green(), style(&name).cyan()),
        Err(e) => println!("  {} {e}", style("✗").red()),
    }

    let _ = std::fs::remove_file(tmp);
    pause();
    Ok(())
}

fn read_with_sudo(path: &str) -> anyhow::Result<String> {
    let out = std::process::Command::new("sudo")
        .args(["cat", path])
        .output()?;
    if out.status.success() {
        Ok(String::from_utf8(out.stdout)?)
    } else {
        anyhow::bail!("sudo cat failed: {}", String::from_utf8_lossy(&out.stderr))
    }
}

fn menu_list(config_path: &str) -> anyhow::Result<()> {
    use nebulark_core::profiles::ProfileManager;
    println!();
    let mgr = ProfileManager::load(config_path)?;
    let profiles = mgr.profiles();
    if profiles.is_empty() {
        println!("  No profiles yet.");
    } else {
        println!("  {}", style("Profiles:").bold());
        for p in profiles {
            println!("    {} {}", style("•").cyan(), p.name);
        }
    }
    pause();
    Ok(())
}

fn print_banner() {
    println!(
        "{}",
        style(r#"
  ███╗   ██╗███████╗██████╗ ██╗   ██╗██╗      █████╗ ██████╗ ██╗  ██╗
  ████╗  ██║██╔════╝██╔══██╗██║   ██║██║     ██╔══██╗██╔══██╗██║ ██╔╝
  ██╔██╗ ██║█████╗  ██████╔╝██║   ██║██║     ███████║██████╔╝█████╔╝
  ██║╚██╗██║██╔══╝  ██╔══██╗██║   ██║██║     ██╔══██║██╔══██╗██╔═██╗
  ██║ ╚████║███████╗██████╔╝╚██████╔╝███████╗██║  ██║██║  ██║██║  ██╗
  ╚═╝  ╚═══╝╚══════╝╚═════╝  ╚═════╝ ╚══════╝╚═╝  ╚═╝╚═╝  ╚═╝╚═╝  ╚═╝
"#).cyan().bold()
    );
    println!("  {} AmneziaWG 2.0 client  {}\n",
        style("//").dim(),
        style("github.com/IsNotAcceptable/Nebulark").dim()
    );
}

fn spinner(msg: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    pb.set_message(msg.to_string());
    pb.enable_steady_tick(Duration::from_millis(80));
    pb
}

fn pause() {
    use std::io::{self, Write};
    print!("\n  Press Enter to continue...");
    io::stdout().flush().unwrap();
    let mut s = String::new();
    io::stdin().read_line(&mut s).ok();
}