use super::{
    AuthPrompt, HostKeyPrompt, InteractionId, InteractionRequest, KeyboardInteractivePrompt,
};

pub(super) fn host(changed: bool) -> InteractionRequest {
    InteractionRequest::HostKey(HostKeyPrompt {
        id: InteractionId::new(),
        host: "visual.example.test".into(),
        port: 2222,
        algorithm: "ssh-ed25519".into(),
        sha256: format!("SHA256:{}", "synthetic-fingerprint-".repeat(18)),
        changed,
    })
}

pub(super) fn auth_request(kind: &str) -> InteractionRequest {
    let prompt = AuthPrompt {
        id: InteractionId::new(),
        label: "Synthetic authentication prompt — use only the disposable visual fixture. "
            .repeat(8),
        echo: false,
    };
    match kind {
        "password" => InteractionRequest::Password(prompt),
        "passphrase" => InteractionRequest::PrivateKeyPassphrase(prompt),
        _ => InteractionRequest::KeyboardInteractive(KeyboardInteractivePrompt {
            id: InteractionId::new(),
            name: "Keyboard authentication".into(),
            instruction:
                "Synthetic keyboard challenge; echo and masked controls must remain reachable. "
                    .repeat(8),
            prompts: vec![
                AuthPrompt {
                    id: InteractionId::new(),
                    label: "Synthetic visible answer".into(),
                    echo: true,
                },
                prompt,
            ],
        }),
    }
}
