use local_ip_address::local_ip;
use anyhow::Result;

/// Get the local LAN IP address
pub fn get_lan_ip() -> Result<String> {
    let ip = local_ip()?;
    Ok(ip.to_string())
}

/// Generate a connection URL with the local IP and specified port
pub fn get_connection_url(port: u16) -> Result<String> {
    let ip = get_lan_ip()?;
    Ok(format!("http://{}:{}", ip, port))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_lan_ip() {
        let ip = get_lan_ip();
        assert!(ip.is_ok());
        let ip_str = ip.unwrap();
        assert!(!ip_str.is_empty());
    }

    #[test]
    fn test_get_connection_url() {
        let url = get_connection_url(38425);
        assert!(url.is_ok());
        let url_str = url.unwrap();
        assert!(url_str.contains("http://"));
        assert!(url_str.contains(":38425"));
    }
}
