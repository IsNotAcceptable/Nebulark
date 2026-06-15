use rand::RngCore;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use x25519_dalek::{PublicKey, StaticSecret};
use base64::{engine::general_purpose::STANDARD, Engine};

const CF_API: &str = "https://api.cloudflareclient.com/v0a2158";
const CF_TOKEN: &str = "6RezDMNHXRBkUDQfPiDi68WzQHfEdBWM";

pub struct AwgPreset {
    pub name: &'static str,
    pub jc: u16,
    pub jmin: u16,
    pub jmax: u16,
    pub s1: u16,
    pub s2: u16,
    pub s3: u16,
    pub s4: u16,
    pub h1: u32,
    pub h2: u32,
    pub h3: u32,
    pub h4: u32,
}

pub const PRESETS: &[AwgPreset] = &[
    AwgPreset {
        name: "AWG 2.0 v1",
        jc: 4, jmin: 40, jmax: 70,
        s1: 0, s2: 0, s3: 0, s4: 0,
        h1: 1, h2: 2, h3: 3, h4: 4,
    },
    AwgPreset {
        name: "AWG 2.0 v2",
        jc: 5, jmin: 50, jmax: 100,
        s1: 10, s2: 20, s3: 0, s4: 0,
        h1: 0xdeadbeef, h2: 0xcafebabe, h3: 0x12345678, h4: 0x87654321,
    },
    AwgPreset {
        name: "AWG 2.0 v3",
        jc: 8, jmin: 30, jmax: 60,
        s1: 5, s2: 10, s3: 15, s4: 20,
        h1: 0x11111111, h2: 0x22222222, h3: 0x33333333, h4: 0x44444444,
    },
];

pub const ENDPOINTS: &[(&str, &str)] = &[
    ("Standart (auto)",      "engage.cloudflareclient.com:2408"),
    ("188.114.96.0:500",        "188.114.96.0:500"),
    ("188.114.96.0:1701",       "188.114.96.0:1701"),
    ("188.114.96.0:4500",       "188.114.96.0:4500"),
    ("162.159.192.1:2408",      "162.159.192.1:2408"),
    ("162.159.192.1:500",       "162.159.192.1:500"),
    ("162.159.192.1:1701",      "162.159.192.1:1701"),
    ("[2606:4700:d0::a29f:c001]:2408", "[2606:4700:d0::a29f:c001]:2408"),
];

pub const DNS_OPTIONS: &[(&str, &str)] = &[
    ("1.1.1.1, 1.0.0.1 (Cloudflare)",  "1.1.1.1,1.0.0.1"),
    ("8.8.8.8, 8.8.4.4 (Google)",      "8.8.8.8,8.8.4.4"),
    ("dns.malw.link",                   "dns.malw.link"),
    ("dns.comss.one",                   "dns.comss.one"),
];

#[derive(Serialize)]
struct RegisterRequest {
    key: String,
    install_id: String,
    fcm_token: String,
    tos: String,
    model: String,
    serial_number: String,
    locale: String,
}

#[derive(Deserialize)]
struct RegisterResponse {
    result: RegisterResult,
}

#[derive(Deserialize)]
struct RegisterResult {
    id: String,
    token: String,
    config: WarpConfig,
}

#[derive(Deserialize)]
struct WarpConfig {
    peers: Vec<WarpPeer>,
    interface: WarpInterface,
}

#[derive(Deserialize)]
struct WarpPeer {
    public_key: String,
    endpoint: WarpEndpoint,
}

#[derive(Deserialize)]
struct WarpEndpoint {
    host: String,
}

#[derive(Deserialize)]
struct WarpInterface {
    addresses: WarpAddresses,
}

#[derive(Deserialize)]
struct WarpAddresses {
    v4: String,
    v6: String,
}

pub struct GeneratedConfig {
    pub conf: String,
    pub profile_name: String,
}

pub async fn generate(
    preset: &AwgPreset,
    endpoint_override: Option<&str>,
    dns: &str,
    mtu: u16,
    keepalive: u16,
) -> anyhow::Result<GeneratedConfig> {
    let mut rng = rand::thread_rng();
    let mut secret_bytes = [0u8; 32];
    rng.fill_bytes(&mut secret_bytes);
    let secret = StaticSecret::from(secret_bytes);
    let public = PublicKey::from(&secret);

    let private_b64 = STANDARD.encode(secret.as_bytes());
    let public_b64 = STANDARD.encode(public.as_bytes());

    let mut id_bytes = [0u8; 16];
    rng.fill_bytes(&mut id_bytes);
    let install_id = hex::encode(id_bytes);

    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()?;

    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();

    let req = RegisterRequest {
        key: public_b64,
        install_id: install_id.clone(),
        fcm_token: install_id.clone(),
        tos: now,
        model: "PC".into(),
        serial_number: install_id.clone(),
        locale: "en_US".into(),
    };

    let resp = client
        .post(format!("{CF_API}/reg"))
        .header("Authorization", format!("Bearer {CF_TOKEN}"))
        .header("Content-Type", "application/json")
        .header("User-Agent", "okhttp/3.12.1")
        .json(&req)
        .send()
        .await?
        .error_for_status()?
        .json::<RegisterResponse>()
        .await?;

    let result = resp.result;
    let peer_pubkey = &result.config.peers[0].public_key;
    let cf_endpoint = &result.config.peers[0].endpoint.host;
    let addr_v4 = &result.config.interface.addresses.v4;
    let addr_v6 = &result.config.interface.addresses.v6;

    let endpoint = endpoint_override.unwrap_or(cf_endpoint);

    let conf = format!(
        "[Interface]\n\
        PrivateKey = {private_b64}\n\
        Address = {addr_v4}/32, {addr_v6}/128\n\
        DNS = {dns}\n\
        MTU = {mtu}\n\
        Jc = {jc}\n\
        Jmin = {jmin}\n\
        Jmax = {jmax}\n\
        S1 = {s1}\n\
        S2 = {s2}\n\
        S3 = {s3}\n\
        S4 = {s4}\n\
        H1 = {h1}\n\
        H2 = {h2}\n\
        H3 = {h3}\n\
        H4 = {h4}\n\
        \n\
        [Peer]\n\
        PublicKey = {peer_pubkey}\n\
        AllowedIPs = 0.0.0.0/0, ::/0\n\
        Endpoint = {endpoint}\n\
        PersistentKeepalive = {keepalive}\n",
        jc = preset.jc, jmin = preset.jmin, jmax = preset.jmax,
        s1 = preset.s1, s2 = preset.s2, s3 = preset.s3, s4 = preset.s4,
        h1 = preset.h1, h2 = preset.h2, h3 = preset.h3, h4 = preset.h4,
    );

    let profile_name = format!("WARP-{}", &result.result.id[..8]);

    Ok(GeneratedConfig { conf, profile_name })
}