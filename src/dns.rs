use resolve::{DnsConfig, record::Txt};

pub fn query_txt(domain: &str) -> Result<Vec<String>, &'static str> {
    let resolver = resolve::DnsResolver::new(
        DnsConfig::load_default().map_err(|_| "Error loading DNS config")?,
    )
    .map_err(|_| "Error creating DNS resolver")?;

    let results = resolver
        .resolve_record::<Txt>(domain)
        .map_err(|_| "Error resolving DNS record")?;

    Ok(results
        .into_iter()
        .filter_map(|txt| String::from_utf8(txt.data).ok())
        .collect::<Vec<String>>())
}

mod tests {

    #[test]
    fn test_query_txt() {
        let domain = "pronouns.kinda.red";
        let results = crate::dns::query_txt(domain).expect("Failed to query TXT records");
        assert!(results.contains(&"she/they".to_string()));
    }
}
