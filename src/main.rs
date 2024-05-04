use anyhow::{anyhow, Result};
use arb_lib::{
    deepl::{ApiOptions, DeeplApi, Lang, LanguageType},
    ArbFile, ArbKey, Intl, Invalidation, TranslationOptions,
};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap},
    path::{Path, PathBuf},
};

use csv::{ReaderBuilder, Writer, WriterBuilder};

#[derive(Debug, Serialize, Deserialize)]
struct CsvRow {
    id: String,
    source: String,
    target: String,
    correction: String,
    comment: String,
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
        #[clap(long)]
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
        #[clap(long)]
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
    /// CSV comparison between template and a target language.
    Compare {
        /// File name prefix.
        #[clap(short, long)]
        name_prefix: Option<String>,

        /// Target language.
        #[clap(short, long)]
        lang: Lang,

        /// Directory of human-translated overrides.
        #[clap(long)]
        overrides: Option<PathBuf>,

        /// Output file for CSV document.
        #[clap(short, long)]
        output: Option<PathBuf>,

        /// Localization YAML file.
        file: PathBuf,
    },
    /// Import CSV corrections to an overrides JSON file.
    Import {
        /// File name prefix.
        #[clap(short, long)]
        name_prefix: Option<String>,

        /// Column delimiter character.
        #[clap(short, long, default_value = ",")]
        delimiter: char,

        /// Target language.
        #[clap(short, long)]
        lang: Lang,

        /// CSV comparison with corrections.
        #[clap(short, long)]
        input: PathBuf,

        /// Directory of human-translated overrides.
        #[clap(long)]
        overrides: Option<PathBuf>,

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

            let overrides = overrides.or(intl.overrides_dir().map(PathBuf::from));
            let overrides = if let Some(dir) = &overrides {
                Some(intl.load_overrides(dir, None)?)
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

            let overrides = overrides.or(intl.overrides_dir().map(PathBuf::from));
            let overrides = if let Some(dir) = &overrides {
                Some(intl.load_overrides(dir, None)?)
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
        }
        Command::Diff {
            name_prefix,
            file,
            languages,
        } => {
            let mut output = BTreeMap::new();
            let intl = new_intl(file, name_prefix)?;
            let template = intl.template_content()?;
            for lang in languages {
                let lang_file = intl.load_or_default(lang)?;
                let diff = template.diff(&lang_file, intl.cache().get_file(&lang));
                output.insert(lang, diff);
            }
            serde_json::to_writer_pretty(std::io::stdout(), &output)?;
        }
        Command::List { file, name_prefix } => {
            let intl = new_intl(file, name_prefix)?;
            let output = intl.list_translated()?;
            serde_json::to_writer_pretty(std::io::stdout(), &output)?;
        }
        Command::Compare {
            file,
            name_prefix,
            output,
            lang,
            overrides,
        } => {
            let intl = new_intl(file, name_prefix)?;

            let overrides = overrides.or(intl.overrides_dir().map(PathBuf::from));
            let overrides = if let Some(dir) = &overrides {
                Some(intl.load_overrides(dir, Some(vec![lang]))?)
            } else {
                None
            };
            let template_lang = intl.template_language();
            let template = intl.template_content()?;
            let translated = intl.list_translated()?;
            let mut rows: Vec<CsvRow> = Vec::new();
            for (language, path) in translated {
                if language != lang {
                    continue;
                }

                let contents = std::fs::read_to_string(path)?;
                let file: ArbFile = serde_json::from_str(&contents)?;

                for entry in template.entries() {
                    if entry.is_translatable() {
                        let correction = if let Some(overrides) = &overrides {
                            if let Some(file) = overrides.get(&lang) {
                                if let Some(entry) = file.lookup(entry.key().as_ref()) {
                                    entry
                                        .value()
                                        .as_str()
                                        .map(|s| s.to_string())
                                        .unwrap_or_default()
                                } else {
                                    String::new()
                                }
                            } else {
                                String::new()
                            }
                        } else {
                            String::new()
                        };

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
                                correction,
                                comment: String::new(),
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
        Command::Import {
            file,
            name_prefix,
            lang,
            delimiter,
            overrides,
            input,
        } => {
            let intl = new_intl(file, name_prefix)?;

            let overrides = overrides.or(intl.overrides_dir().map(PathBuf::from));
            let overrides = overrides.ok_or_else(|| {
                anyhow!("no overrides, either configure overrides-dir or set --overrides")
            })?;
            let mut overrides_map = intl.load_overrides(&overrides, Some(vec![lang]))?;
            let mut default = ArbFile::default();
            let overrides_file = overrides_map.get_mut(&lang).unwrap_or_else(|| &mut default);
            let mut rdr = ReaderBuilder::new()
                .delimiter(delimiter as u8)
                .from_path(input)?;
            for result in rdr.deserialize() {
                // Our headers don't match the field names!
                let record: (String, String, String, String, String) = result?;
                let record = CsvRow {
                    id: record.0,
                    source: record.1,
                    target: record.2,
                    correction: record.3,
                    comment: record.4,
                };
                if !record.correction.is_empty() {
                    overrides_file.insert_translation(&ArbKey::new(&record.id), record.correction);
                }
            }

            let output_name = intl.format_file_name(lang);
            let output_file = overrides.join(output_name);

            tracing::info!(path = %output_file.display(), "write file");
            serde_json::to_writer_pretty(std::fs::File::create(&output_file)?, &overrides_file)?;
        }
    }
    Ok(())
}

fn new_intl(path: impl AsRef<Path>, name_prefix: Option<String>) -> Result<Intl> {
    Ok(Intl::new_with_prefix(path, name_prefix)?)
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
    let correction_header = format!("Correction ({})", target);
    wtr.write_record(&[
        "Identifier",
        &source_header,
        &target_header,
        &correction_header,
        "Comment",
    ])?;
    for row in rows {
        wtr.serialize(&row)?;
    }
    wtr.flush()?;
    Ok(())
}
