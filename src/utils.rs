use std::env;
use dotenv::dotenv;

pub fn get_bitbucket_token() -> String {
    dotenv().ok();
    env::var("BITBUCKET_TOKEN").expect("El token BITBUCKET_TOKEN no está definido en .env")
}

pub fn get_bitbucket_user() -> String {
    dotenv().ok();
    env::var("BITBUCKET_USER").expect("El usuario BITBUCKET_USER no está definido en .env")
}
