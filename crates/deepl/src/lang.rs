use crate::Error;
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

/// Languages supported by the DeepL API.
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING-KEBAB-CASE")]
pub enum Lang {
    /// Arabic.
    Ar,
    /// Bulgarian.
    Bg,
    /// Czech.
    Cs,
    /// Danish.
    Da,
    /// German.
    De,
    /// Greek.
    El,
    /// English.
    En,
    /// English (British)
    EnGb,
    /// English (American)
    EnUs,
    /// Spanish
    Es,
    /// Estonian
    Et,
    /// Finnish
    Fi,
    /// French.
    Fr,
    /// Hungarian.
    Hu,
    /// Indonesian.
    Id,
    /// Italian.
    It,
    /// Japanese.
    Ja,
    /// Korean.
    Ko,
    /// Lithuanian.
    Lt,
    /// Latvian.
    Lv,
    /// Norwegian.
    Nb,
    /// Dutch.
    Nl,
    /// Polish.
    Pl,
    /// Portuguese (all varieties mixed).
    Pt,
    /// Portuguese (Brazilian).
    PtBr,
    /// Portuguese (all Portuguese varieties excluding Brazilian Portuguese).
    PtPt,
    /// Romanian.
    Ro,
    /// Russian.
    Ru,
    /// Slovak.
    Sk,
    /// Slovenian.
    Sl,
    /// Swedish.
    Sv,
    /// Turkish.
    Tr,
    /// Ukrainian.
    Uk,
    /// Chinese.
    Zh,
}

impl fmt::Display for Lang {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Ar => "AR",
                Self::Bg => "BG",
                Self::Cs => "CS",
                Self::Da => "DA",
                Self::De => "DE",
                Self::El => "EL",
                Self::En => "EN",
                Self::EnGb => "EN-GB",
                Self::EnUs => "EN-US",
                Self::Es => "ES",
                Self::Et => "ET",
                Self::Fi => "FI",
                Self::Fr => "FR",
                Self::Hu => "HU",
                Self::Id => "ID",
                Self::It => "IT",
                Self::Ja => "JA",
                Self::Ko => "KO",
                Self::Lt => "LT",
                Self::Lv => "LV",
                Self::Nb => "NB",
                Self::Nl => "NL",
                Self::Pl => "PL",
                Self::Pt => "PT",
                Self::PtBr => "PT-BR",
                Self::PtPt => "PT-PT",
                Self::Ro => "RO",
                Self::Ru => "RU",
                Self::Sk => "SK",
                Self::Sl => "SL",
                Self::Sv => "SV",
                Self::Tr => "TR",
                Self::Uk => "UK",
                Self::Zh => "ZH",
            }
        )
    }
}

impl FromStr for Lang {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "AR" | "ar" => Self::Ar,
            "BG" | "bg" => Self::Bg,
            "CS" | "cs" => Self::Cs,
            "DA" | "da" => Self::Da,
            "DE" | "de" => Self::De,
            "EL" | "el" => Self::El,
            "EN" | "en" => Self::En,
            "EN-GB" | "en-gb" => Self::EnGb,
            "EN-US" | "en-us" => Self::EnUs,
            "ES" | "es" => Self::Es,
            "ET" | "et" => Self::Et,
            "FI" | "fi" => Self::Fi,
            "FR" | "fr" => Self::Fr,
            "HU" | "hu" => Self::Hu,
            "ID" | "id" => Self::Id,
            "IT" | "it" => Self::It,
            "JA" | "ja" => Self::Ja,
            "KO" | "ko" => Self::Ko,
            "LT" | "lt" => Self::Lt,
            "LV" | "lv" => Self::Lv,
            "NB" | "nb" => Self::Nb,
            "NL" | "nl" => Self::Nl,
            "PL" | "pl" => Self::Pl,
            "PT" | "pt" => Self::Pt,
            "PT-BR" | "pt-br" => Self::PtBr,
            "PT-PT" | "pt-pt" => Self::PtPt,
            "RO" | "ro" => Self::Ro,
            "RU" | "ru" => Self::Ru,
            "SK" | "sk" => Self::Sk,
            "SL" | "sl" => Self::Sl,
            "SV" | "sv" => Self::Sv,
            "TR" | "tr" => Self::Tr,
            "UK" | "uk" => Self::Uk,
            "ZH" | "zh" => Self::Zh,
            _ => {
                return Err(Error::InvalidLang(s.to_string()));
            }
        })
    }
}
