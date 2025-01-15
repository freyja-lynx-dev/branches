use crate::types::AtUri;
use atrium_api::agent::{store::MemorySessionStore, AtpAgent};
use atrium_api::did_doc::DidDocument;
use atrium_api::types::string::{AtIdentifier, Nsid, RecordKey};
use atrium_api::xrpc;
use atrium_api::*;
use atrium_xrpc_client::reqwest::ReqwestClient;
use http::Request;
use relm4::prelude::*;
use relm4::Worker;
use std::borrow::Borrow;
// use std::str::FromStr;

// pub enum AtUriError {
//     MissingIdentifier,
// }

// #[derive(Debug)]
// pub struct AtUri {
//     pub authority: String,
//     pub collection: Option<String>,
//     pub rkey: Option<String>,
// }

// impl FromStr for AtUri {
//     type Err = String;

//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         let components = s
//             .strip_prefix("web+")
//             .unwrap_or(s)
//             .strip_prefix("at://")
//             .unwrap_or(s)
//             .splitn(3, '/')
//             .collect::<Vec<&str>>();
//         /* alright so here's where i'm leaving off bc i'm sleepy
//         i want FromStr to validate the URI components, but only the handle/did is required
//         so we need to ensure that there is a valid identifier (and error if not), then see if
//         there is a valid nsid and rkey, putting them there if valid, and returning None if not

//         some questions:
//         - this should probably be in its own file and exported as a type
//         - do i just match nsid and rkey and transform the err into a None?
//         - i can't even think of the questions i'm so tired
//         - maybe i'll have better brain tomorrow
//         */
//         let authority: String = components
//             .first()
//             .ok_or(AtUriError::MissingIdentifier)?
//             .to_string();

//         let collection = components.get(1).map(|s| s.to_string());
//         let rkey = components.get(2).map(|s| s.to_string());

//         Ok(AtUri {
//             authority,
//             collection,
//             rkey,
//         })
//     }
// }

// impl std::fmt::Display for AtUri {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         let mut uri = self.authority.clone();
//         match self.collection.clone() {
//             Some(s) => {
//                 uri.push('/');
//                 uri.push_str(s.as_str())
//             }
//             None => (),
//         };
//         match self.rkey.clone() {
//             Some(s) => {
//                 uri.push('/');
//                 uri.push_str(s.as_str())
//             }
//             None => (),
//         };
//         write!(f, "at://{}", uri)
//     }
// }

pub struct AtprotoClient {
    atp_agent: AtpAgent<MemorySessionStore, ReqwestClient>,
}

impl AtprotoClient {
    async fn get_repo(
        &self,
        identifier: AtIdentifier,
        sender: ComponentSender<Self>,
    ) -> Result<(), AtprotoClientOutput> {
        let request = self
            .atp_agent
            .api
            .com
            .atproto
            .repo
            .describe_repo(
                com::atproto::repo::describe_repo::ParametersData { repo: identifier }.into(),
            )
            .await;
        match request {
            Ok(r) => sender.output(AtprotoClientOutput::Repo(r.data)),
            Err(e) => sender.output(AtprotoClientOutput::Error(e.to_string())),
        }
        // Result<Object<OutputData>, Error<Error>>
    }
}

#[derive(Debug)]
pub enum AtprotoClientInput {
    // auth will be implemented later
    // Authenticate,
    // Deauthenticate,
    Get(String),
}

#[derive(Debug)]
pub enum AtprotoClientOutput {
    Record(com::atproto::repo::get_record::OutputData),
    Repo(com::atproto::repo::describe_repo::OutputData),
    Error(String),
}

impl Worker for AtprotoClient {
    type Init = ();
    type Input = AtprotoClientInput;
    type Output = AtprotoClientOutput;

    fn init(_init: Self::Init, _sender: ComponentSender<Self>) -> Self {
        Self {
            atp_agent: AtpAgent::new(
                ReqwestClient::new("https://public.api.bsky.app"),
                MemorySessionStore::default(),
            ),
        }
    }

    fn update(&mut self, msg: AtprotoClientInput, sender: ComponentSender<Self>) {
        match msg {
            AtprotoClientInput::Get(text) => {
                if let Ok(uri) = text.parse::<AtUri>() {
                    println!("at uri: {:?}", uri);
                    // if we have a collection and rkey: com.atproto.repo.getRecord
                    // if we have a collection but no rkey: com.atproto.repo.listRecords
                    // if we just have an authority: com.atproto.repo.describeRepo
                } else {
                    sender.output(AtprotoClientOutput::Error(format!("invalid uri: {}", text)));
                }
                // if text.starts_with("did:plc") {
                //     println!("is did:plc : {}", text);
                //     let request =
                //         Request::get(String::from("https://plc.directory/") + text.as_str())
                //             .body(())
                //             .unwrap();
                // } else if text.starts_with("did:web") {
                //     println!("is did:web : {}", text);
                //     // let did_web = text.as_str().split(':').collect::<Vec<&str>>();
                //     let request = Request::get(format!(
                //         "https://{}/.well-known/did.json",
                //         text.as_str().split(':').collect::<Vec<&str>>()[2]
                //     ))
                //     .body(())
                //     .unwrap();
                // // in the future we might also have to handle web+at:// URIs
                // // for opening the URI from some other app
                // } else if text.starts_with("at://") {
                //     println!("is at:// uri: {}", text);
                // } else {
                //     println!("is presumably handle: {}", text);
                //     // send the messatge to the client
                // }
                // sender.output(AtprotoClientOutput::Repo("repo placeholder".to_string()));
            }
        }
    }
}
