use artifactsmmo_api_wrapper::ArtifactApi;
use std::sync::Arc;

// NOTE: WIP
pub struct Client {
    account_name: String,
    api: Arc<ArtifactApi>,
    // characters: Vec<Arc<BaseCharacter>>>,
}

impl Client {
    pub fn new(url: String, account_name: String, token: String) -> Self {
        let api = Arc::new(ArtifactApi::new(url, token));
        // let characters = api
        //     .account
        //     .characters(&account_name)
        //     .unwrap()
        //     .data
        //     .into_iter()
        //     .enumerate()
        //     .map(|(id, data)| {
        //         BaseCharacter::new(
        //             id,
        //             Arc::new(RwLock::new(Arc::new(data))),
        //             bank.clone(),
        //             api.clone(),
        //         )
        //     })
        //     .collect::<Vec<Character>>();
        Self {
            account_name,
            api,
            // characters: vec![],
        }
    }
}
