use atrium_api::did_doc::DidDocument;
use atrium_api::types::string::Did;
use reqwest::get;
use serde_json;
use std::error::Error;

pub async fn did_doc_from_plc_directory(did: &Did) -> Result<Option<String>, Box<dyn Error>> {
    let did_doc: DidDocument = serde_json::from_str(
        &get(format!("https://plc.directory/{}", did.as_str()))
            .await?
            .text()
            .await?,
    )?;
    Ok(did_doc.get_pds_endpoint())
}
pub async fn did_doc_from_web(did: &Did) -> Result<Option<String>, Box<dyn Error>> {
    let did_doc: DidDocument = serde_json::from_str(
        &get(format!("https://{}/.well-known/did.json", did.as_str()))
            .await?
            .text()
            .await?,
    )?;
    Ok(did_doc.get_pds_endpoint())
}
