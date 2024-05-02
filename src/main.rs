use anyhow::Result;
use arb_lib::{
    deepl::{ApiOptions, DeeplApi, Lang, LanguageType},
    ArbFile, Intl, Invalidation, TranslationOptions,
};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap},
    path::{Path, PathBuf},
};

use csv::{Writer, WriterBuilder};

#[derive(Debug, Serialize, Deserialize)]
struct CsvRow {
    id: String,
    source: String,
    target: String,
    correction: String,
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Arb {
    #[clap(subcommand)]
    cmd: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Translate the template to a language.
    #[clap(alias = "tl")]
    Translate {
        /// API key.
        #[clap(short, long, hide_env_values = true, env = "DEEPL_API_KEY")]
        api_key: String,

        /// Invalidate all keys.
        #[clap(short, long)]
        force: bool,

        /// Invalidate specific keys.
        #[clap(short, long)]
        invalidate: Vec<String>,

        /// Directory of human-translated overrides.
        #[clap(short, long)]
        overrides: Option<PathBuf>,

        /// Translate and write to disc.
        #[clap(long)]
        apply: bool,

        /// File name prefix.
        #[clap(short, long)]
        name_prefix: Option<String>,

        /// Target language.
        #[clap(short, long)]
        lang: Lang,

        /// Localization YAML file.
        file: PathBuf,
    },
    /// Update existing translations.
    #[clap(alias = "up")]
    Update {
        /// API key.
        #[clap(short, long, hide_env_values = true, env = "DEEPL_API_KEY")]
        api_key: String,

        /// Invalidate all keys.
        #[clap(short, long)]
        force: bool,

        /// Invalidate specific keys.
        #[clap(short, long)]
        invalidate: Vec<String>,

        /// Directory of human-translated overrides.
        #[clap(short, long)]
        overrides: Option<PathBuf>,

        /// Translate and write to disc.
        #[clap(long)]
        apply: bool,

        /// File name prefix.
        #[clap(short, long)]
        name_prefix: Option<String>,

        /// Localization YAML file.
        file: PathBuf,
    },

    /// Print account usage.
    Usage {
        /// API key.
        #[clap(short, long, hide_env_values = true, env = "DEEPL_API_KEY")]
        api_key: String,
    },
    /// List language application resource bundles.
    #[clap(alias = "ls")]
    List {
        /// File name prefix.
        #[clap(short, long)]
        name_prefix: Option<String>,

        /// Localization YAML file.
        file: PathBuf,
    },
    /// Print supported languages.
    Languages {
        /// API key.
        #[clap(short, long, hide_env_values = true, env = "DEEPL_API_KEY")]
        api_key: String,

        /// Language type (source or target).
        #[clap(short, long, default_value = "source")]
        language_type: LanguageType,
    },
    /// Diff template with language(s).
    Diff {
        /// File name prefix.
        #[clap(short, long)]
        name_prefix: Option<String>,

        /// Languages to compare to the template language.
        #[clap(short, long)]
        languages: Vec<Lang>,

        /// Localization YAML file.
        file: PathBuf,
    },
    /// Generate CSV comparison between template and a target language.
    Compare {
        /// File name prefix.
        #[clap(short, long)]
        name_prefix: Option<String>,

        /// Target language.
        #[clap(short, long)]
        lang: Lang,

        /// Output file for CSV document.
        #[clap(short, long)]
        output: Option<PathBuf>,

        /// Localization YAML file.
        file: PathBuf,
    },
}

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(false)
                .without_time(),
        )
        .init();

    let args = Arb::parse();
    match args.cmd {
        Command::Update {
            file,
            api_key,
            name_prefix,
            apply,
            force,
            invalidate,
            overrides,
        } => {
            let mut intl = new_intl(&file, name_prefix.clone())?;

            let overrides = if let Some(dir) = &overrides {
                Some(intl.load_overrides(dir)?)
            } else {
                None
            };

            let translations = intl.list_translated()?;
            for lang in translations.keys() {
                if lang == intl.template_language() {
                    continue;
                }
                translate_language(
                    &mut intl,
                    *lang,
                    api_key.clone(),
                    apply,
                    force,
                    invalidate.clone(),
                    overrides.clone(),
                )
                .await?;
            }

            if !apply {
                tracing::warn!("dry run, use --apply to translate");
            }
        }

        Command::Translate {
            lang,
            file,
            api_key,
            name_prefix,
            apply,
            force,
            invalidate,
            overrides,
        } => {
            let mut intl = new_intl(&file, name_prefix.clone())?;

            let overrides = if let Some(dir) = &overrides {
                Some(intl.load_overrides(dir)?)
            } else {
                None
            };

            translate_language(
                &mut intl, lang, api_key, apply, force, invalidate, overrides,
            )
            .await?;

            if !apply {
                tracing::warn!("dry run, use --apply to translate");
            }
        }
        Command::Usage { api_key } => {
            let options = ApiOptions::new(api_key);
            let api = DeeplApi::new(options);
            let usage = api.usage().await?;
            serde_json::to_writer_pretty(std::io::stdout(), &usage)?;
            println!();
        }
        Command::Languages {
            api_key,
            language_type,
        } => {
            let options = ApiOptions::new(api_key);
            let api = DeeplApi::new(options);
            let langs = api.languages(language_type).await?;
            serde_json::to_writer_pretty(std::io::stdout(), &langs)?;
            println!();
        }
        Command::Diff {
            name_prefix,
            file,
            languages,
        } => {
            let mut output = BTreeMap::new();
            let index = new_intl(file, name_prefix)?;
            let template = index.template_content()?;
            for lang in languages {
                let lang_file = index.load_or_default(lang)?;
                let diff = template.diff(&lang_file, index.cache().get_file(&lang));
                output.insert(lang, diff);
            }
            serde_json::to_writer_pretty(std::io::stdout(), &output)?;
            println!();
        }
        Command::List { file, name_prefix } => {
            let index = new_intl(file, name_prefix)?;
            let output = index.list_translated()?;
            serde_json::to_writer_pretty(std::io::stdout(), &output)?;
            println!();
        }
        Command::Compare {
            file,
            name_prefix,
            output,
            lang,
        } => {
            let index = new_intl(file, name_prefix)?;
            let template_lang = index.template_language();
            let template = index.template_content()?;
            let translated = index.list_translated()?;
            let mut rows: Vec<CsvRow> = Vec::new();
            for (language, path) in translated {
                if language != lang {
                    continue;
                }

                let contents = std::fs::read_to_string(path)?;
                let file: ArbFile = serde_json::from_str(&contents)?;

                for entry in template.entries() {
                    if entry.is_translatable() {
                        if let Some(target) = file.lookup(entry.key().as_ref()) {
                            rows.push(CsvRow {
                                id: entry.key().as_ref().to_string(),
                                source: entry
                                    .value()
                                    .as_str()
                                    .map(|s| s.to_string())
                                    .unwrap_or_default(),
                                target: target
                                    .value()
                                    .as_str()
                                    .map(|s| s.to_string())
                                    .unwrap_or_default(),
                                correction: String::new(),
                            });
                        }
                    }
                }
            }

            if let Some(path) = output {
                let wtr = WriterBuilder::new().has_headers(false).from_path(path)?;
                write_csv_rows(wtr, rows, *template_lang, lang)?;
            } else {
                let wtr = WriterBuilder::new()
                    .has_headers(false)
                    .from_writer(std::io::stdout());
                write_csv_rows(wtr, rows, *template_lang, lang)?;
            }
        }
    }
    Ok(())
}

fn new_intl(path: impl AsRef<Path>, name_prefix: Option<String>) -> Result<Intl> {
    Ok(if let Some(name_prefix) = name_prefix {
        Intl::new_with_prefix(path, name_prefix)?
    } else {
        Intl::new(path)?
    })
}

async fn translate_language(
    intl: &mut Intl,
    lang: Lang,
    api_key: String,
    apply: bool,
    force: bool,
    invalidate: Vec<String>,
    overrides: Option<HashMap<Lang, ArbFile>>,
) -> Result<()> {
    let invalidation = if force {
        Some(Invalidation::All)
    } else if !invalidate.is_empty() {
        Some(Invalidation::Keys(invalidate))
    } else {
        None
    };

    let api = DeeplApi::new(ApiOptions::new(api_key));
    let options = TranslationOptions {
        target_lang: lang,
        dry_run: !apply,
        invalidation,
        overrides,
        disable_cache: false,
    };

    let result = intl.translate(&api, options).await?;

    if apply {
        let content = serde_json::to_string_pretty(&result.translated)?;
        let file_path = intl.file_path(lang)?;
        tracing::info!(path = %file_path.display(), "write file");
        std::fs::write(&file_path, &content)?;
    }
    Ok(())
}

fn write_csv_rows<W: std::io::Write>(
    mut wtr: Writer<W>,
    rows: Vec<CsvRow>,
    source: Lang,
    target: Lang,
) -> Result<()> {
    let source_header = format!("Source ({})", source);
    let target_header = format!("Target ({})", target);
    wtr.write_record(&["Identifier", &source_header, &target_header, "Correction"])?;
    for row in rows {
        wtr.serialize(&row)?;
    }
    wtr.flush()?;
    Ok(())
}
