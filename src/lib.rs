#[cfg(feature = "dns_resolve")]
mod dns;
mod parser;
pub mod pronouns;

#[cfg(feature = "dns_resolve")]
pub use dns::query_txt;

pub use parser::parse_record;

#[cfg(feature = "dns_resolve")]
pub fn resolve_pronouns(domain: &str) -> std::io::Result<Vec<pronouns::PronounRecord>> {
    let txt_records = query_txt(domain)?;

    let mut pronoun_records = Vec::new();

    for record in txt_records {
        match parse_record(&record) {
            Ok(pronoun_record) => pronoun_records.push(pronoun_record),
            Err(_) => continue, // Ignore parsing errors
        }
    }

    Ok(pronoun_records)
}
