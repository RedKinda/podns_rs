#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PronounRecord {
    pub set: Option<PronounSet>,
    pub tags: Vec<PronounTag>,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PronounSet {
    Defined(PronounDef),
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
    pub fn new(set: Option<PronounSet>, tags: Vec<PronounTag>, comment: Option<String>) -> Self {
        PronounRecord { set, tags, comment }
    }
}

impl PronounSet {
    pub fn new_defined(
        subject: String,
        object: String,
        possessive_adjective: Option<String>,
        possessive_pronoun: Option<String>,
        reflexive: Option<String>,
    ) -> Self {
        PronounSet::Defined(PronounDef {
            subject,
            object,
            possessive_determiner: possessive_adjective,
            possessive_pronoun,
            reflexive,
        })
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
