use resolve::record::Txt;

pub fn query_txt(domain: &str) -> Result<Vec<String>, &'static str> {
    let config = {
        #[cfg(windows)]
        {
            windows::default_dns_config().map_err(|_| "Error loading DNS config")?
        }
        #[cfg(not(windows))]
        {
            resolve::DnsConfig::load_default().map_err(|e| {
                panic!("[debug] Error loading DNS config: {}", e);
                "Error loading DNS config"
            })?
        }
    };

    let resolver = resolve::DnsResolver::new(config).map_err(|_| "Error creating DNS resolver")?;

    let results = resolver
        .resolve_record::<Txt>(domain)
        .map_err(|_| "Error resolving DNS record")?;

    Ok(results
        .into_iter()
        .filter_map(|txt| String::from_utf8(txt.data).ok())
        .collect::<Vec<String>>())
}

#[cfg(windows)]
mod windows {
    use std::io;

    use resolve::DnsConfig;

    pub(super) fn default_dns_config() -> io::Result<DnsConfig> {
        use windows::Win32::Foundation::ERROR_BUFFER_OVERFLOW;
        use windows::Win32::NetworkManagement::IpHelper::GetNetworkParams;

        // First call to get the required buffer size
        let mut buf_len: u32 = 0;
        let result = unsafe { GetNetworkParams(None, &mut buf_len) };

        if result != ERROR_BUFFER_OVERFLOW {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("GetNetworkParams failed"),
            ));
        }

        // Allocate buffer and call again
        let mut buffer = vec![0u8; buf_len as usize];
        let fixed_info = buffer.as_mut_ptr()
            as *mut windows::Win32::NetworkManagement::IpHelper::FIXED_INFO_W2KSP1;

        let result = unsafe { GetNetworkParams(Some(fixed_info), &mut buf_len) };

        if result.0 != 0 {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("GetNetworkParams failed"),
            ));
        }

        // Extract DNS servers
        let mut nameservers: Vec<std::net::SocketAddr> = Vec::new();
        unsafe {
            let fixed = &*fixed_info;

            // Add primary DNS server
            let dns_addr = std::ffi::CStr::from_ptr(
                fixed.DnsServerList.IpAddress.String.as_ptr() as *const i8
            )
            .to_string_lossy();

            if !dns_addr.is_empty() && dns_addr != "0.0.0.0" {
                if let Ok(addr) = dns_addr.parse::<std::net::SocketAddr>() {
                    nameservers.push(addr);
                }
            }

            // Add additional DNS servers from linked list
            let mut current = fixed.DnsServerList.Next;
            while !current.is_null() {
                let dns_addr =
                    std::ffi::CStr::from_ptr((*current).IpAddress.String.as_ptr() as *const i8)
                        .to_string_lossy();

                if !dns_addr.is_empty() && dns_addr != "0.0.0.0" {
                    if let Ok(addr) = dns_addr.parse::<std::net::SocketAddr>() {
                        nameservers.push(addr);
                    }
                }
                current = (*current).Next;
            }
        }

        if nameservers.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "No DNS servers found",
            ));
        }

        Ok(DnsConfig::with_name_servers(nameservers))
    }
}

mod tests {

    #[test]
    fn test_query_txt() {
        let domain = "pronouns.kinda.red";
        let results = crate::dns::query_txt(domain).expect("Failed to query TXT records");
        assert!(results.contains(&"she/they".to_string()));
    }
}
