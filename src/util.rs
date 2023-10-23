pub fn get_env(env: &'static str) -> String { // todo what's 'static?
    std::env::var(env).unwrap_or_else(|_| panic!("Cannot get the {env} env variable"))
}