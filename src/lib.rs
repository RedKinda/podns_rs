#[cfg(feature = "dns_resolve")]
mod dns;

#[cfg(feature = "dns_resolve")]
pub use dns::query_txt;
