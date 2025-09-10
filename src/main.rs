mod bitbucket;
mod ai;
mod utils;

use utils::{get_bitbucket_token, get_bitbucket_user, get_repo_slug, get_workspace};
use bitbucket::{fetch_issues, create_branch, get_latest_commit, commit_file, branch_exists};
use ai::{generate_branch_name, generate_fix_code};

use reqwest::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token: String = get_bitbucket_token();
    let user: String = get_bitbucket_user();
    let repo_slug: String = get_repo_slug();
    let workspace: String = get_workspace();
    let repo = format!("{}/{}", workspace, repo_slug);
    let client = Client::new();

    // Obtenemos el último commit de la rama base (main)
    let base_commit = get_latest_commit(&client, &token, &repo, &user).await?;
    println!("Último commit en la rama base: {}", base_commit);

    // Obtenemos issues del repo
    match fetch_issues(&client, &token, &repo, &user).await {
        Ok(issues) => {
            if issues.is_empty() {
                println!("No hay issues en el repositorio");
            } else {
                for issue in issues {
                    // Nombre de rama por IA
                    match generate_branch_name(&issue.id.to_string(), &issue.title).await {
                        Ok(branch) => {
                            // Si la rama no existe, crearla
                            if branch_exists(&client, &token, &repo, &user, &branch).await? {
                                println!("La rama '{}' ya existe, se harán commits en ella", branch);
                            } else {
                                println!("Creando rama '{}' para issue {}", branch, issue.id);
                                if let Err(err) = create_branch(&client, &token, &repo, &user, &branch, &base_commit).await {
                                    println!("Error creando rama para issue {}: {}", issue.id, err);
                                    continue;
                                }
                            }

                            // --- GENERAR CÓDIGO con IA ---
                            let language = std::env::var("DEFAULT_LANG")
                                .unwrap_or_else(|_| "javascript".to_string());

                            match generate_fix_code(&issue.id.to_string(), &issue.title, &issue.content.raw, &language).await {
                                Ok((file_path, code)) => {
                                    println!("Código generado. Archivo: {}", file_path);

                                    // --- HACER COMMIT en la rama ---
                                    match commit_file(&client, &token, &repo, &user, &branch, &file_path, &code).await {
                                        Ok(_) => println!("✅ Commit realizado correctamente en '{}' (rama {})", file_path, branch),
                                        Err(e) => println!("❌ Error al commitear el archivo {}: {}", file_path, e),
                                    }
                                }
                                Err(e) => {
                                    println!("❌ Error generando código para issue {}: {}", issue.id, e);
                                }
                            }
                        }
                        Err(err) => println!("❌ Error generando nombre de rama para issue {}: {}", issue.id, err),
                    }
                }
            }
        }
        Err(err) => println!("❌ Error al obtener issues: {}", err),
    }
    Ok(())
}