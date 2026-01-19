#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PronounRecord {
    pub set: Option<PronounSet>,
    pub comment: Option<String>,
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
