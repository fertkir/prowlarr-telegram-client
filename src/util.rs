use std::net::{IpAddr, Ipv4Addr};

pub fn parse_ip(env_var: &str) -> IpAddr {
    IpAddr::V4(std::env::var(env_var)
        .unwrap_or_else(|_| "0.0.0.0".to_string())
        .parse::<Ipv4Addr>()
        .unwrap_or_else(|_| panic!("Cannot parse the {env_var} env variable")))
}
