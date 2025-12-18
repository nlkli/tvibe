use clap::Parser;
use strsim::levenshtein;
use std::{collections::HashSet, fmt::Write, io::BufRead};
mod collection;
mod color;
mod models;
mod templ;

const NVIM_CONFIG_FILE_PATH: &str = ".config/nvim/init.lua";
const ALACRITTY_CONFIG_FILE_PATH: &str = ".config/alacritty/alacritty.toml";

fn write_theme_to_nvim_config(theme: &mut models::Theme) -> Result<(), Box<dyn std::error::Error>> {
    const START_BLOCK_MARK: &str = "-- ====THEMESYNCSTARTBLOCK====";
    const END_BLOCK_MARK: &str = "-- ====THEMESYNCENDBLOCK====";

    let content = templ::nvim(theme);
    let full_config_path = std::env::home_dir().expect("home_dir").join(NVIM_CONFIG_FILE_PATH);
    let file = std::fs::File::open(&full_config_path).unwrap();
    let reader = std::io::BufReader::new(file);

    let mut buff = String::new();
    let mut lines = reader.lines();
    let mut insert_done = false;

    while let Some(line) = lines.next() {
        let line = line?;
        writeln!(&mut buff, "{}", &line)?;
        if line == START_BLOCK_MARK {
            writeln!(&mut buff, "{}", &content)?;
            insert_done = true;
            break;
        }
    }
    if insert_done {
        let mut old_block_buff = String::new();
        let mut block_end = false;
        while let Some(line) = lines.next() {
            let line = line?;
            writeln!(&mut old_block_buff, "{}", &line)?;
            if line == END_BLOCK_MARK {
                block_end = true;
                break;
            }
        }
        if block_end {
            writeln!(&mut buff, "{}", &END_BLOCK_MARK)?;
        } else {
            writeln!(&mut buff, "{}", &old_block_buff)?;
        }
        while let Some(line) = lines.next() {
            let line = line?;
            writeln!(&mut buff, "{}", &line)?;
        }
    } else {
        writeln!(
            &mut buff,
            "\n{START_BLOCK_MARK}\n{}\n{END_BLOCK_MARK}",
            &content
        )?;
    }

    std::fs::write(&full_config_path, &buff)?;

    Ok(())
}

fn alacritty_full_config_path() -> std::path::PathBuf {
    std::env::home_dir()
        .expect("home_dir")
        .join(ALACRITTY_CONFIG_FILE_PATH)
}

fn read_alacritty_config(path: impl AsRef<std::path::Path>) -> Result<models::alacritty::Config, Box<dyn std::error::Error>> {
    let buff = std::fs::read_to_string(path)?;
    Ok(toml::from_str::<models::alacritty::Config>(&buff)?)
}

fn write_alacritty_config(path: impl AsRef<std::path::Path>, config: &models::alacritty::Config) -> Result<models::alacritty::Config, Box<dyn std::error::Error>> {
    let buff = toml::to_string_pretty(&config)?;
    std::fs::write(&path, &buff)?;
    Ok(toml::from_str::<models::alacritty::Config>(&buff)?)
}

fn write_theme_to_alacritty_config(
    theme: &mut models::Theme,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = alacritty_full_config_path();
    let mut config = read_alacritty_config(&path)?;
    config.replace_colors_from_theme(theme.get_colors());
    write_alacritty_config(path, &config)?;
    Ok(())
}

fn nerd_font_list() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let list = if cfg!(target_os = "macos") {
        let path = std::env::home_dir().expect("home_dir").join("Library/Fonts");
        std::fs::read_dir(path)?.into_iter()
            .map(|e| e.expect("read_dir").path())
            .filter(|e| e.is_file())
            .map(|f| f.file_name()
                .unwrap()
                .to_os_string()
                .into_string()
                .expect("utf8")
                .split_once("-")
                .map(|(p1, p2)| (p1.to_string(), p2.to_string())))
            .filter(Option::is_some)
            .map(|v| v.unwrap().0)
            .filter(|s| s.contains("NerdFont"))
            .collect::<HashSet<_>>()
            .into_iter()
            .map(|v| {
                v.replace("NerdFont", " Nerd Font ").replace("  ", " ").trim().to_string()
            })
            .collect()
    } else if cfg!(target_os = "linux") {
        println!("not implemented for linux");
        vec![]
    } else {
        vec![]
    };

    Ok(list)
}

fn change_alacritty_font(query: &str) -> Result<(), Box<dyn std::error::Error>> {
    let query = query.to_lowercase();
    if cfg!(target_os = "linux") {
        println!("not implemented for linux");
    } else if cfg!(target_os = "macos") {
        let fonts = nerd_font_list()?;
        if fonts.is_empty() {
            println!("not found nerd family fonts");
            return Ok(());
        }
        let font = fonts.iter()
            .min_by_key(|v| levenshtein(&v.to_lowercase(), &query)).ok_or("not match")?;
        let path = alacritty_full_config_path();
        let mut config = read_alacritty_config(&path)?;
        config.set_font_family(font);
        write_alacritty_config(path, &config)?;
    }
    Ok(())
}

/// theme changing utility
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Set random theme
    #[arg(short, long)]
    rand: bool,

    /// Set random light theme
    #[arg(short, long)]
    light_rand: bool,

    /// Set random dark theme
    #[arg(short, long)]
    dark_rand: bool,

    /// Search and apply theme
    #[arg(short, long)]
    query: Option<String>,

    /// List of available themes
    #[arg(long)]
    list: bool,

    /// Change alacritty nerd font family
    #[arg(short, long)]
    font: Option<String>,

    /// List of nerd font family
    #[arg(long)]
    font_list: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let mut exit = false;
    if args.list {
        for i in collection::LIST {
            println!("{}", i);
        }
        exit = true;
    }
    if args.font_list {
        for i in nerd_font_list()? {
            println!("{}", i);
        }
        exit = true;
    }
    if let Some(ref font) = args.font {
        change_alacritty_font(font)?;
        exit = true;
    }

    if exit {
        return Ok(());
    }

    let mut theme = {
        if args.rand {
            if args.dark_rand {
                collection::rand_dark()
            } else if args.light_rand {
                collection::rand_light()
            } else {
                collection::rand()
            }
        } else if args.dark_rand {
            collection::rand_dark()
        } else if args.light_rand {
            collection::rand_light()
        } else if !args.rand && args.query.is_some() {
            collection::search(&unsafe { args.query.unwrap_unchecked() })
        } else {
            collection::rand()
        }
    };

    theme.prepare()?;
    theme.validation()?;

    write_theme_to_nvim_config(&mut theme)?;
    write_theme_to_alacritty_config(&mut theme)?;

    println!("{}", theme.name.clone().unwrap_or_default());

    Ok(())
}
