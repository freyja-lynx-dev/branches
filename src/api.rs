use atrium_api::did_doc::DidDocument;
use atrium_api::types::string::{AtIdentifier, Did, Handle};
use reqwest::get;
use serde_json;
use std::error::Error;

/* notes
this function can have four(?) types of errors:
1. unrecognized DID method error
2. a reqwest error (couldn't find server, couldn't find did.json, etc)
3. a serde_json error (whatever did doc ingested was invalid json)
4. invalid DidDocument

in terms of how _common_ each of these errors would be, only the reqwest errors are likely:
1. non-blessed DID methods will be exceptionally uncommon, and we will add support for blessed ones as they come
2. plc.directory should only be serving valid DidDocuments
3. did:web users with invalid DidDocument have far bigger problems than this app not working

fortunately, basically all of these errors have the same negative outcome: we couldn't resolve a DidDocument.
regardless it is not ideal to have to deal with four separate sources of errors, across different types
in the future it would be best to map all of these errors to a common, descriptive error type and not have to return
a Box<dyn Error>

for consumers: you should be prepared to handle any of these errors, and be prepared to update code that uses this
function to handle a common, non-Boxed error type when it happens
*/
pub async fn get_did_doc_for(did: &Did) -> Result<DidDocument, Box<dyn Error>> {
    let uri = match did.method() {
        "did:plc" => format!("https://plc.directory/{}", did.as_str()),
        "did:web" => format!("https://{}/.well-known/did.json", did.as_str()),
        method => return Err(format!("unrecognized DID method: {}", method).into()),
    };
    Ok(serde_json::from_str(&get(uri).await?.text().await?)?)
}

/*
internally, this function uses `get_did_doc_for` to get the DID document, and as such shares its wart
of returning a Box<dyn Error> with four separate sources of errors, plus one extra:
1. unrecognized DID method error
2. a reqwest error (couldn't find server, couldn't find did.json, etc)
3. a serde_json error (whatever did doc ingested was invalid json)
4. invalid DidDocument
5. no PDS endpoint found

for more information, see `get_did_doc_for()`
*/
pub async fn get_pds_endpoint_for(did: &Did) -> Result<String, Box<dyn Error>> {
    let did_doc = get_did_doc_for(did).await?;
    Ok(did_doc.get_pds_endpoint().ok_or("no PDS endpoint found")?)
}

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
