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
                if let Some(poss_det) = &definition.possessive_determiner() {
                    write!(f, "/{}", poss_det)?;
                }
                if let Some(poss_pron) = &definition.possessive_pronoun() {
                    write!(f, "/{}", poss_pron)?;
                }
                if let Some(reflexive) = &definition.reflexive() {
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

impl PartialOrd for PronounSet {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/*
first should be preferred over non-preferred
second should be non-any and non-none
third should be any over none
fourth should be lexicographical order of subject, then object
*/
impl Ord for PronounSet {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (
                PronounSet::Defined {
                    tags: tags_a,
                    definition: def_a,
                },
                PronounSet::Defined {
                    tags: tags_b,
                    definition: def_b,
                },
            ) => {
                let a_preferred = tags_a.contains(&PronounTag::Preferred);
                let b_preferred = tags_b.contains(&PronounTag::Preferred);

                match (a_preferred, b_preferred) {
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                    _ => def_a
                        .subject
                        .cmp(&def_b.subject)
                        .then_with(|| def_a.object.cmp(&def_b.object)),
                }
            }
            (PronounSet::Defined { .. }, _) => std::cmp::Ordering::Less,
            (_, PronounSet::Defined { .. }) => std::cmp::Ordering::Greater,
            (PronounSet::Any, PronounSet::None) => std::cmp::Ordering::Less,
            (PronounSet::None, PronounSet::Any) => std::cmp::Ordering::Greater,
            _ => std::cmp::Ordering::Equal,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommonPronounDef {
    Masculine,
    Feminine,
    Neuter,
    TheyThem,
}

impl CommonPronounDef {
    pub fn subject(&self) -> &str {
        match self {
            CommonPronounDef::Masculine => "he",
            CommonPronounDef::Feminine => "she",
            CommonPronounDef::Neuter => "it",
            CommonPronounDef::TheyThem => "they",
        }
    }

    pub fn object(&self) -> &str {
        match self {
            CommonPronounDef::Masculine => "him",
            CommonPronounDef::Feminine => "her",
            CommonPronounDef::Neuter => "it",
            CommonPronounDef::TheyThem => "them",
        }
    }

    pub fn possessive_determiner(&self) -> &str {
        match self {
            CommonPronounDef::Masculine => "his",
            CommonPronounDef::Feminine => "her",
            CommonPronounDef::Neuter => "its",
            CommonPronounDef::TheyThem => "their",
        }
    }

    pub fn possessive_pronoun(&self) -> &str {
        match self {
            CommonPronounDef::Masculine => "his",
            CommonPronounDef::Feminine => "hers",
            CommonPronounDef::Neuter => "its",
            CommonPronounDef::TheyThem => "theirs",
        }
    }

    pub fn reflexive(&self) -> &str {
        match self {
            CommonPronounDef::Masculine => "himself",
            CommonPronounDef::Feminine => "herself",
            CommonPronounDef::Neuter => "itself",
            CommonPronounDef::TheyThem => "themself",
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

    common_def: Option<CommonPronounDef>,
}

impl PronounDef {
    pub fn new(
        subject: String,
        object: String,
        possessive_determiner: Option<String>,
        possessive_pronoun: Option<String>,
        reflexive: Option<String>,
    ) -> Self {
        let mut def = PronounDef {
            subject,
            object,
            possessive_determiner,
            possessive_pronoun,
            reflexive,
            common_def: None,
        };

        def.guess_common();

        def
    }

    pub fn subject(&self) -> &str {
        &self.subject
    }

    pub fn object(&self) -> &str {
        &self.object
    }

    pub fn possessive_determiner(&self) -> Option<&str> {
        // either defined, or from common_def
        self.possessive_determiner.as_deref().or_else(|| {
            self.common_def
                .as_ref()
                .map(|common| common.possessive_determiner())
        })
    }

    pub fn possessive_pronoun(&self) -> Option<&str> {
        self.possessive_pronoun.as_deref().or_else(|| {
            self.common_def
                .as_ref()
                .map(|common| common.possessive_pronoun())
        })
    }

    pub fn reflexive(&self) -> Option<&str> {
        self.reflexive
            .as_deref()
            .or_else(|| self.common_def.as_ref().map(|common| common.reflexive()))
    }

    pub fn common_def(&self) -> Option<&CommonPronounDef> {
        self.common_def.as_ref()
    }

    pub(crate) fn guess_common(&mut self) {
        // if subject+object match, and rest either match or are None, set common_def
        let common = match (self.subject.as_str(), self.object.as_str()) {
            ("he", "him") => Some(CommonPronounDef::Masculine),
            ("she", "her") => Some(CommonPronounDef::Feminine),
            ("it", "it") => Some(CommonPronounDef::Neuter),
            ("they", "them") => Some(CommonPronounDef::TheyThem),
            _ => None,
        };

        if let Some(common_def) = common {
            let poss_det_match = match &self.possessive_determiner {
                Some(pd) => pd == common_def.possessive_determiner(),
                None => true,
            };
            let poss_pron_match = match &self.possessive_pronoun {
                Some(pp) => pp == common_def.possessive_pronoun(),
                None => true,
            };
            let reflexive_match = match &self.reflexive {
                Some(r) => r == common_def.reflexive(),
                None => true,
            };

            if poss_det_match && poss_pron_match && reflexive_match {
                self.common_def = Some(common_def);
            }
        }
    }
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
        let def: PronounDef = PronounDef::new(
            subject,
            object,
            possessive_adjective,
            possessive_pronoun,
            reflexive,
        );

        PronounSet::Defined {
            definition: def,
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

    #[test]
    fn test_common_def_match() {
        let def = PronounDef::new("she".to_string(), "her".to_string(), None, None, None);

        assert_eq!(def.common_def(), Some(&CommonPronounDef::Feminine));
        assert_eq!(def.possessive_determiner(), Some("her"));
        assert_eq!(def.possessive_pronoun(), Some("hers"));
        assert_eq!(def.reflexive(), Some("herself"));
    }
}
