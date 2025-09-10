mod bitbucket;
mod ai;
mod utils;

use utils::{get_bitbucket_token, get_bitbucket_user};
use bitbucket::{fetch_issues, create_branch, get_latest_commit};
use ai::{generate_branch_name};

use reqwest::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token: String = get_bitbucket_token();
    let user: String = get_bitbucket_user();
    let repo = "map_py_dev/cotizadorhogar-frontend";
    let client = Client::new();

    // Obtenemos el último commit de la rama base (main)
    let base_commit = get_latest_commit(&client, &token, repo, &user).await?;
        println!("Último commit en la rama base: {}", base_commit);

    match fetch_issues(&client, &token, repo, &user).await {
        Ok(issues) => {
            if issues.is_empty() {
                println!("No hay issues en el repositorio");
            } else {
                for issue in issues {
                    println!("Issue: {:?}", issue);

                    // Generamos el nombre de la rama con IA
                    match generate_branch_name(&issue.id.to_string(), &issue.title).await {
                        Ok(branch) => {
                            // Ahora usamos el base_commit en create_branch
                            if let Err(err) = create_branch(&client, &token, repo, &user, &branch, &base_commit).await {
                                println!("Error creando rama para issue {}: {}", issue.id, err);
                            } else {
                                println!("Rama '{}' creada para el issue {}", branch, issue.id);
                            }
                        },
                        Err(err) => println!("Error generando rama para issue {}: {}", issue.id, err),
                    }
                }
            }
        }
        Err(err) => println!("Error: {}", err),
    }

    Ok(())
}