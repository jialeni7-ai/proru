use base64::{Engine, engine::general_purpose};
use dotenvy::dotenv;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::env;

fn sign(secret: &str, prehash: &str) -> String {
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(prehash.as_bytes());
    general_purpose::STANDARD.encode(mac.finalize().into_bytes())
}

pub fn okx_open() -> anyhow::Result<()> {
    dotenv().ok();
    let api_key = env::var("OKX_API_KEY")?;
    let secret = env::var("OKX_SECRET_KEY")?;
    let passphrase = env::var("OKX_PASSPHRASE")?;
    let ts = chrono::Utc::now().timestamp().to_string();
    let prehash = format!("{ts}GET/users/self/verify");
    let sign = sign(&secret, &prehash);
    let login = serde_json::json!({
        "op":"login",
        "args":[{
            "apiKey":api_key,
            "passphrase":passphrase,
            "timestamp":ts,
            "sign":sign
        }]
    });

    ws.send(Message::Text(login.to_string().into())).await?;
    Ok(())
}
