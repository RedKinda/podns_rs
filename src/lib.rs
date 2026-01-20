#[cfg(feature = "dns_resolve")]
mod dns;
mod parser;
pub mod pronouns;

#[cfg(feature = "dns_resolve")]
pub use dns::query_txt;

pub use parser::{ParserError, parse_record};
pub use pronouns::{CommonPronounDef, PronounDef, PronounRecord, PronounSet, PronounTag};

#[cfg(feature = "dns_resolve")]
pub fn resolve_pronouns(domain: &str) -> std::io::Result<Vec<pronouns::PronounRecord>> {
    // make sure there is a `pronouns.` prefix on the domain
    let mut domain = domain;
    let domain_qualified;

    if !domain.starts_with("pronouns.") {
        domain_qualified = format!("pronouns.{}", domain);
        domain = &domain_qualified;
    }

    let txt_records = query_txt(domain)?;

    let mut pronoun_records = Vec::new();

    for record in txt_records {
        match parse_record(&record) {
            Ok(pronoun_record) => pronoun_records.push(pronoun_record),
            Err(e) => {
                eprintln!("Warning: Failed to parse record '{}': {:?}", record, e);
            }
        }
    }

    Ok(pronoun_records)
}

#[cfg(all(test, feature = "dns_resolve"))]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_pronouns() {
        let result = resolve_pronouns("kinda.red");
        assert!(result.is_ok());
        let result2 = resolve_pronouns("pronouns.kinda.red");
        assert!(result2.is_ok());
    }
}
