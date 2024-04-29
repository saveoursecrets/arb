use arb_lib::{
    deepl::{ApiOptions, DeeplApi, Lang, LanguageType},
    translate, TranslationOptions,
};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Arb {
    #[clap(subcommand)]
    cmd: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Translate to a language.
    #[clap(alias = "tl")]
    Translate {
        /// Target language.
        #[clap(short, long, hide_env_values = true, env = "DEEPL_API_KEY")]
        api_key: String,

        /// Use DeepL API pro endpoint.
        #[clap(short, long)]
        pro: bool,

        /// Dry run.
        #[clap(short, long)]
        dry_run: bool,

        /// Write language file to disc.
        #[clap(short, long)]
        write: bool,

        /// File name prefix.
        #[clap(short, long, default_value = "app")]
        name_prefix: String,

        /// Target language.
        #[clap(short, long)]
        lang: Lang,

        /// Localization YAML file.
        file: PathBuf,
    },
    /// Print account usage.
    Usage {
        /// Target language.
        #[clap(short, long, hide_env_values = true, env = "DEEPL_API_KEY")]
        api_key: String,

        /// Use DeepL API pro endpoint.
        #[clap(short, long)]
        pro: bool,
    },
    /// Print supported languages.
    Languages {
        /// Target language.
        #[clap(short, long, hide_env_values = true, env = "DEEPL_API_KEY")]
        api_key: String,

        /// Use DeepL API pro endpoint.
        #[clap(short, long)]
        pro: bool,
        /// Language type (source or target).
        #[clap(short, long, default_value = "source")]
        language_type: LanguageType,
    },
}

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let args = Arb::parse();
    match args.cmd {
        Command::Translate {
            lang,
            file,
            api_key,
            pro,
            write,
            name_prefix,
            dry_run,
        } => {
            let options = if pro {
                ApiOptions::new_pro(api_key)
            } else {
                ApiOptions::new_free(api_key)
            };
            let api = DeeplApi::new(options);
            let options = TranslationOptions {
                index_file: file,
                target_lang: lang,
                dry_run,
                name_prefix,
            };
            let result = translate(api, options).await?;

            if write && !dry_run {
                let content = serde_json::to_string_pretty(&result.translated)?;
                let file_path = result.index.file_path(lang)?;
                tracing::info!(path = %file_path.display(), "write file");
                std::fs::write(&file_path, &content)?;
            } else {
                serde_json::to_writer_pretty(std::io::stdout(), &result.translated)?;
            }
        }
        Command::Usage { api_key, pro } => {
            let options = if pro {
                ApiOptions::new_pro(api_key)
            } else {
                ApiOptions::new_free(api_key)
            };
            let api = DeeplApi::new(options);
            let usage = api.usage().await?;
            serde_json::to_writer_pretty(std::io::stdout(), &usage)?;
            println!();
        }
        Command::Languages {
            api_key,
            pro,
            language_type,
        } => {
            let options = if pro {
                ApiOptions::new_pro(api_key)
            } else {
                ApiOptions::new_free(api_key)
            };
            let api = DeeplApi::new(options);
            let langs = api.languages(language_type).await?;
            serde_json::to_writer_pretty(std::io::stdout(), &langs)?;
            println!();
        }
    }
    Ok(())
}
