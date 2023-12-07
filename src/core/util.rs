use std::net::{IpAddr, Ipv4Addr};

pub fn parse_ip(env_var: &str) -> IpAddr {
    IpAddr::V4(std::env::var(env_var)
        .unwrap_or_else(|_| "0.0.0.0".to_string())
        .parse::<Ipv4Addr>()
        .unwrap_or_else(|_| panic!("Cannot parse the {env_var} env variable")))
}

#[cfg(test)]
mod tests {
    use crate::util::parse_ip;

    #[test]
    fn parse_ip_from_env_var() {
        temp_env::with_var("LOCALHOST", Some("127.0.0.1"), || {
            assert_eq!(parse_ip("LOCALHOST").to_string(), "127.0.0.1")
        });
    }

    #[test]
    fn default_value() {
        assert_eq!(parse_ip("DEFAULT_IP").to_string(), "0.0.0.0")
    }

    #[test]
    #[should_panic(expected = "Cannot parse the INCORRECT_IP env variable")]
    fn incorrect_ip_address() {
        temp_env::with_var("INCORRECT_IP", Some("aaa"), || {
            parse_ip("INCORRECT_IP");
        });
    }
}
