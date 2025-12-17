use std::{fmt::Write, io::BufRead};
use clap::Parser;
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
    let full_config_path = std::env::home_dir().unwrap().join(NVIM_CONFIG_FILE_PATH);
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

fn write_theme_to_alacritty_config(
    theme: &mut models::Theme,
) -> Result<(), Box<dyn std::error::Error>> {
    let full_config_path = std::env::home_dir().unwrap().join(ALACRITTY_CONFIG_FILE_PATH);
    let buff = std::fs::read_to_string(&full_config_path)?;
    let mut config = toml::from_str::<models::alacritty::Config>(&buff)?;
    config.replace_colors_from_theme(theme.get_colors());
    let buff = toml::to_string_pretty(&config)?;
    std::fs::write(&full_config_path, &buff)?;
    Ok(())
}

/// themesymc
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
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    if args.list {
        for i in collection::LIST {
            println!("{}", i);
        }
        return Ok(());
    }

    let mut theme = {
        if args.rand {
            let theme = if args.dark_rand {
                collection::rand_dark()
            } else if args.light_rand {
                collection::rand_light()
            } else {
                collection::rand()
            };
            println!("{}", theme.name.clone().unwrap_or_default());
            theme
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

    Ok(())
}
