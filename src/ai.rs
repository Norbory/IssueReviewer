use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::error::Error;

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

fn slugify(s: &str) -> String {
    let lower = s.to_lowercase();
    let mut out = String::with_capacity(lower.len());
    let mut last_was_dash = false;
    for c in lower.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c);
            last_was_dash = false;
        } else if c.is_whitespace() || "-_/".contains(c) {
            if !last_was_dash {
                out.push('-');
                last_was_dash = true;
            }
        } else {
            // ignore other punctuation (or replace with dash)
            if !last_was_dash {
                out.push('-');
                last_was_dash = true;
            }
        }
    }
    // trim dashes
    out.trim_matches('-').to_string()
}

/// Genera código con Gemini para arreglar un issue.
/// Retorna (filename, content).
pub async fn generate_fix_code(
    issue_id: &str,
    title: &str,
    description: &str,
    language: &str, // e.g. "javascript", "typescript", "python"
) -> Result<(String, String), Box<dyn Error>> {
    let api_key = std::env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY no está configurada");
    let url = format!("https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash:generateContent?key={}", api_key);

    // Prompt estructurado pidiendo JSON válido
    let prompt = format!(
        r#"Actúa como un desarrollador senior. Tienes la información del issue:
            - ID: {id}
            - Título: {title}
            - Descripción: {description}

            Genera UN SOLO archivo en lenguaje {lang} que solucione este issue. Debe ser un archivo funcional y mínimo que implemente la corrección o componente necesario. Incluye comentarios explicativos en el código.

            **RESPONDE SÓLO** con un objeto JSON válido, sin explicaciones adicionales, con esta forma:
            {{"filename":"relative/path/to/file.ext","content":"<archivo completo como string>"}}

            Ejemplo:
            {{"filename":"src/fix-login.js","content":"// código aquí\nfunction ..."}}
        "#,
        id = issue_id,
        title = title,
        description = description,
        lang = language
    );

    let body = GeminiRequest {
        contents: vec![GeminiContent {
            parts: vec![GeminiPart { text: prompt }],
        }],
    };

    let client = reqwest::Client::new();
    let resp = client
        .post(&url)
        .json(&body)
        .send()
        .await?;

    let resp_json: GeminiResponse = resp.json().await?;

    if let Some(err) = resp_json.error {
        return Err(format!("Gemini API error: {}", err.message).into());
    }

    if resp_json.candidates.is_empty() {
        return Err("La respuesta de Gemini no tiene candidatos".into());
    }

    let text = resp_json.candidates[0].content.parts[0].text.trim().to_string();

    // Intentamos parsear JSON estructurado (filename + content)
    if let Ok(v) = serde_json::from_str::<Value>(&text) {
        if let (Some(fname), Some(content)) = (v.get("filename"), v.get("content")) {
            if let (Some(fname_s), Some(content_s)) = (fname.as_str(), content.as_str()) {
                return Ok((fname_s.to_string(), content_s.to_string()));
            }
        }
    }

    // Fallback: si no viene JSON, asumimos que el texto es solo el contenido y generamos filename.
    // Creamos filename usando slug del title.
    let ext = match language.to_lowercase().as_str() {
        "typescript" => "ts",
        "python" => "py",
        "rust" => "rs",
        _ => "js",
    };
    let fname = format!("src/{}-{}.{}", issue_id, slugify(title), ext);
    // Si el texto viene con triple backticks, limpiamos
    let cleaned = if text.starts_with("```") {
        // quitar delimitadores ```[lang]\n ... \n```
        let _parts: Vec<&str> = text.splitn(2, '\n').collect();
        let mut t = text.to_string();
        // eliminar primera y última ``` si existen
        if t.starts_with("```") {
            t = t.replacen("```", "", 1);
        }
        if t.ends_with("```") {
            t.truncate(t.len() - 3);
        }
        t.trim().to_string()
    } else {
        text
    };

    Ok((fname, cleaned))
}