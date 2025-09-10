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

pub fn get_repo_slug() -> String {
    dotenv().ok();
    env::var("REPO_SLUG").expect("El repositorio REPO_SLUG no está definido en .env")
}

pub fn get_workspace() -> String {
    dotenv().ok();
    env::var("WORKSPACE").expect("El workspace WORKSPACE no está definido en .env")
}