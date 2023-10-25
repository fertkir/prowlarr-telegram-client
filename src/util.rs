pub fn get_env(env: &str) -> String {
    std::env::var(env).unwrap_or_else(|_| panic!("Cannot get the {env} env variable"))
}