use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use sha2::{Digest, Sha256};

pub struct PkceChallenge {
    pub code_verifier: String,
    pub code_challenge: String,
}

pub fn generate_pkce() -> PkceChallenge {
    let verifier_bytes = generate_random_bytes();
    let code_verifier = URL_SAFE_NO_PAD.encode(&verifier_bytes);

    let mut hasher = Sha256::new();
    hasher.update(code_verifier.as_bytes());
    let hash = hasher.finalize();
    let code_challenge = URL_SAFE_NO_PAD.encode(hash);

    PkceChallenge {
        code_verifier,
        code_challenge,
    }
}

fn generate_random_bytes() -> [u8; 32] {
    let mut bytes = [0u8; 32];
    getrandom::getrandom(&mut bytes)
        .expect("OS CSPRNG unavailable — cannot generate PKCE verifier safely");
    bytes
}
