use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PronounRecord {
    pub set: Option<PronounSet>,
    pub comment: Option<String>,
}

impl Display for PronounRecord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let has_set = match &self.set {
            Some(pronoun_set) => {
                write!(f, "{}", pronoun_set)?;
                true
            }
            None => false,
        };

        if let Some(comment) = &self.comment {
            if has_set {
                write!(f, " ")?;
            }

            write!(f, "# {}", comment)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PronounSet {
    Defined {
        definition: PronounDef,
        tags: Vec<PronounTag>,
    },
    /// Represented by * in the TXT record
    Any,
    /// Represented by ! in the TXT record
    None,
}

impl Display for PronounSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PronounSet::Defined { definition, tags } => {
                write!(f, "{}/{}", definition.subject, definition.object)?;
                if let Some(poss_det) = &definition.possessive_determiner {
                    write!(f, "/{}", poss_det)?;
                }
                if let Some(poss_pron) = &definition.possessive_pronoun {
                    write!(f, "/{}", poss_pron)?;
                }
                if let Some(reflexive) = &definition.reflexive {
                    write!(f, "/{}", reflexive)?;
                }
                if !tags.is_empty() {
                    // tags are started and separated by `; `
                    for tag in tags.iter() {
                        write!(f, "; {}", tag)?;
                    }
                }
                Ok(())
            }
            PronounSet::Any => write!(f, "*"),
            PronounSet::None => write!(f, "!"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PronounDef {
    pub subject: String,
    pub object: String,
    pub possessive_determiner: Option<String>,
    pub possessive_pronoun: Option<String>,
    pub reflexive: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum PronounTag {
    Preferred,
    Plural,
}

impl Display for PronounTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PronounTag::Preferred => write!(f, "preferred"),
            PronounTag::Plural => write!(f, "plural"),
        }
    }
}

impl PronounRecord {
    pub fn new(set: Option<PronounSet>, comment: Option<String>) -> Self {
        PronounRecord { set, comment }
    }
}

impl PronounSet {
    pub fn new_defined(
        subject: String,
        object: String,
        possessive_adjective: Option<String>,
        possessive_pronoun: Option<String>,
        reflexive: Option<String>,
        tags: Vec<PronounTag>,
    ) -> Self {
        PronounSet::Defined {
            definition: PronounDef {
                subject,
                object,
                possessive_determiner: possessive_adjective,
                possessive_pronoun,
                reflexive,
            },
            tags,
        }
    }
}

impl PronounTag {
    pub fn from_string(string: String) -> Option<Self> {
        match string.as_str() {
            "preferred" => Some(PronounTag::Preferred),
            "plural" => Some(PronounTag::Plural),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    // test Display implementations
    use super::*;

    #[test]
    fn test_basic() {
        let record = PronounRecord::new(
            Some(PronounSet::new_defined(
                "he".to_string(),
                "him".to_string(),
                None,
                None,
                None,
                vec![],
            )),
            None,
        );
        let display = format!("{}", record);
        assert_eq!(display, "he/him");
    }

    #[test]
    fn test_comment_only() {
        let record = PronounRecord::new(None, Some("No pronouns".to_string()));
        let display = format!("{}", record);
        assert_eq!(display, "# No pronouns");
    }

    #[test]
    fn test_any_and_none() {
        let any_record = PronounRecord::new(Some(PronounSet::Any), None);
        let none_record = PronounRecord::new(Some(PronounSet::None), None);
        let display_any = format!("{}", any_record);
        let display_none = format!("{}", none_record);
        assert_eq!(display_any, "*");
        assert_eq!(display_none, "!");
    }

    #[test]
    fn test_pronoun_record_display() {
        let record = PronounRecord::new(
            Some(PronounSet::new_defined(
                "they".to_string(),
                "them".to_string(),
                Some("their".to_string()),
                Some("theirs".to_string()),
                Some("themself".to_string()),
                vec![PronounTag::Preferred, PronounTag::Plural],
            )),
            Some("These are my pronouns".to_string()),
        );
        let display = format!("{}", record);
        assert_eq!(
            display,
            "they/them/their/theirs/themself; preferred; plural # These are my pronouns"
        );
    }
}
