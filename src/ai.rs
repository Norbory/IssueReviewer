use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
}

#[derive(Serialize)]
struct GeminiContent {
    parts: Vec<GeminiPart>,
}

#[derive(Serialize)]
struct GeminiPart {
    text: String,
}

#[derive(Deserialize, Debug)]
pub struct GeminiResponse {
    #[serde(default)]
    pub candidates: Vec<GeminiCandidate>,
    #[serde(default)]
    pub error: Option<GeminiError>,
}

#[derive(Deserialize, Debug)]
pub struct GeminiCandidate {
    pub content: GeminiContentResponse,
}

#[derive(Deserialize, Debug)]
pub struct GeminiContentResponse {
    pub parts: Vec<GeminiPartResponse>,
}

#[derive(Deserialize, Debug)]
pub struct GeminiPartResponse {
    pub text: String,
}

#[derive(Deserialize, Debug)]
pub struct GeminiError {
    pub message: String,
}


pub async fn generate_branch_name(issue_id: &str, description: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let api_key = std::env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY no está configurada");
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash:generateContent?key={}",
        api_key
    );

    // Prompt más claro y estructurado
    let prompt = format!(
        "Crea un nombre de rama para un issue con ID {} y descripción '{}'. 
        El nombre de la rama debe seguir este formato:
        feature/<issue_id>-<descripcion_corta>
        - <issue_id> es el número del issue
        - <descripcion_corta> debe estar en minúsculas, sin acentos ni caracteres especiales, y con guiones en lugar de espacios.
        Solo responde con el nombre de la rama, nada más.",
        issue_id, description
    );

    let request_body = GeminiRequest {
        contents: vec![GeminiContent {
            parts: vec![GeminiPart { text: prompt }],
        }],
    };

    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .json(&request_body)
        .send()
        .await?
        .json::<GeminiResponse>()
        .await?;

    // Verifica si la API devolvió un error
    if let Some(err) = &response.error {
        return Err(format!("Gemini API error: {}", err.message).into());
    }

    // Verifica si hay candidatos
    if response.candidates.is_empty() {
        return Err("La respuesta de Gemini no tiene candidatos. Verifica tu API Key o tu request.".into());
    }

    // Si todo está bien, usa el primer candidato
    Ok(response.candidates[0].content.parts[0].text.trim().to_string())
}

pub async fn _fix_issue() {

}