use crate::Error;
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

/// Languages supported by the DeepL API.
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING-KEBAB-CASE")]
pub enum Lang {
    Ar,
    Bg,
    Cs,
    Da,
    De,
    El,
    En,
    EnGb,
    EnUs,
    Es,
    Et,
    Fi,
    Fr,
    Hu,
    Id,
    It,
    Ja,
    Ko,
    Lt,
    Lv,
    Nb,
    Nl,
    Pl,
    Pt,
    PtBr,
    PtPt,
    Ro,
    Ru,
    Sk,
    Sl,
    Sv,
    Tr,
    Uk,
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
            "AR" => Self::Ar,
            "BG" => Self::Bg,
            "CS" => Self::Cs,
            "DA" => Self::Da,
            "DE" => Self::De,
            "EL" => Self::El,
            "EN" => Self::En,
            "EN-GB" => Self::EnGb,
            "EN-US" => Self::EnUs,
            "ES" => Self::Es,
            "ET" => Self::Et,
            "FI" => Self::Fi,
            "FR" => Self::Fr,
            "HU" => Self::Hu,
            "ID" => Self::Id,
            "IT" => Self::It,
            "JA" => Self::Ja,
            "KO" => Self::Ko,
            "LT" => Self::Lt,
            "LV" => Self::Lv,
            "NB" => Self::Nb,
            "NL" => Self::Nl,
            "PL" => Self::Pl,
            "PT" => Self::Pt,
            "PT-BR" => Self::PtBr,
            "PT-PT" => Self::PtPt,
            "RO" => Self::Ro,
            "RU" => Self::Ru,
            "SK" => Self::Sk,
            "SL" => Self::Sl,
            "SV" => Self::Sv,
            "TR" => Self::Tr,
            "UK" => Self::Uk,
            "ZH" => Self::Zh,
            _ => {
                return Err(Error::InvalidLang(s.to_string()));
            }
        })
    }
}
