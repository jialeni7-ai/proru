use serde::Deserialize;

#[derive(Deserialize)]
struct OkxInstResp {
    data: Vec<OkxInstData>,
}

#[derive(Deserialize)]
struct OkxInstData {
    #[serde(rename="ctVal")]
    ct_val: String,
}

pub async fn get_okx_ct_mult(inst_id: &str) -> anyhow::Result<f64> {
    let url = format!(
        "https://www.okx.com/api/v5/public/instruments?instType=SWAP&instId={}",
        inst_id
    );
        let resp: OkxInstResp = reqwest::get(&url).await?.json().await?;

    let ct_mult = resp.data
        .get(0)
        .ok_or(anyhow::anyhow!("no instrument"))?
        .ct_val
        .parse::<f64>()?;

    Ok(ct_mult)
}
