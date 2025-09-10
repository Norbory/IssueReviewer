use serde::Deserialize;
use reqwest::{Client};
use std::error::Error;

#[derive(Debug, Deserialize)]
pub struct Issue {
    pub id: i32,
    pub title: String,
    pub content: IssueContent,
}

#[derive(Debug, Deserialize)]
pub struct IssueContent {
    pub raw: String,
}

#[derive(Deserialize, Debug)]
pub struct Content {
    pub _raw: String,
}

pub async fn fetch_issues(client: &Client, token: &str, repo: &str, user: &str) -> Result<Vec<Issue>, Box<dyn Error>> {
    let url = format!("https://api.bitbucket.org/2.0/repositories/{}/issues", repo);
    let resp = client.get(&url)
        .basic_auth(user, Some(token))
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;
    
    if resp["values"].is_null() {
        return Ok(vec![]); // Devuelve lista vacía si no hay issues
    }

    let issues: Vec<Issue> = serde_json::from_value(resp["values"].clone())?;
    Ok(issues)
}

pub async fn create_branch(
    client: &Client,
    token: &str,
    repo: &str,
    user: &str,
    branch_name: &str,
    from_commit: &str
) -> Result<(), Box<dyn std::error::Error>> {
    let url = format!("https://api.bitbucket.org/2.0/repositories/{}/refs/branches", repo);
    let body = serde_json::json!({
        "name": branch_name,
        "target": {
            "hash": from_commit
        }
    });

    let resp = client.post(&url)
        .basic_auth(user, Some(token))
        .json(&body)
        .send()
        .await?;

    if resp.status().is_success() {
        Ok(())
    } else {
        Err(format!("Failed to create branch: {}", resp.text().await?).into())
    }
}

pub async fn get_latest_commit(
    client: &reqwest::Client,
    token: &str,
    repo: &str,
    user: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let url = format!("https://api.bitbucket.org/2.0/repositories/{}/commits", repo);

    let resp = client
        .get(&url)
        .basic_auth(user, Some(token))
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    let commit_hash = resp["values"][0]["hash"]
        .as_str()
        .ok_or("No se pudo obtener el hash del commit")?
        .to_string();

    Ok(commit_hash)
}

/// Sube (commitea) un archivo usando el endpoint POST /src en una rama existente
pub async fn commit_file(
    client: &Client,
    token: &str,
    repo: &str,
    user: &str,
    branch: &str,
    file_path: &str,
    content: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // La URL ya no necesita user separado si repo incluye workspace
    let url = format!("https://api.bitbucket.org/2.0/repositories/{}/src", repo);

    // Commit en la rama: si ya existe, solo agrega o actualiza el archivo
    let form = reqwest::multipart::Form::new()
        .text("branch", branch.to_string())
        .text("message", "Feat: Gemini_Solution".to_string())
        .text(file_path.to_string(), content.to_string());

    let resp = client
        .post(&url)
        .basic_auth(user, Some(token))
        .multipart(form)
        .send()
        .await?;

    if resp.status().is_success() {
        Ok(())
    } else {
        let status = resp.status();
        let body = resp.text().await?;
        Err(format!("Failed to commit file: {} - {}", status, body).into())
    }
}

pub async fn branch_exists(
    client: &Client,
    token: &str,
    repo: &str,
    user: &str,
    branch_name: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    let url = format!("https://api.bitbucket.org/2.0/repositories/{}/refs/branches/{}", repo, branch_name);

    let resp = client
        .get(&url)
        .basic_auth(user, Some(token))
        .send()
        .await?;

    if resp.status().is_success() {
        Ok(true) // La rama existe
    } else if resp.status().as_u16() == 404 {
        Ok(false) // La rama no existe
    } else {
        Err(format!("Error checking branch existence: {}", resp.text().await?).into())
    }
}