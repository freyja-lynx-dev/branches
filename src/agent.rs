use crate::api::*;
use crate::types::AtUri;
use atrium_api::agent::{store::MemorySessionStore, AtpAgent};
use atrium_api::did_doc::DidDocument;
use atrium_api::types::string::{AtIdentifier, Did, Handle, Nsid, RecordKey};
use atrium_api::xrpc::Result as AtResult;
use atrium_api::*;
use atrium_xrpc_client::reqwest::ReqwestClient;
use relm4::prelude::*;

#[derive(Debug)]
pub enum AgentInput {
    GetDidDoc(AtIdentifier),
    GetPdsEndpointFor(AtIdentifier),
    GetURI(AtUri),
}
#[derive(Debug)]
pub enum AgentOutput {
    DidDoc(DidDocument),
    PdsEndpoint(String),
    Repo(com::atproto::repo::describe_repo::OutputData),
    Records(com::atproto::repo::list_records::OutputData),
    Record(com::atproto::repo::get_record::OutputData),
    Error(AgentError),
}
#[derive(Debug)]
pub enum AgentCommand {}

#[derive(Debug, Clone)]
pub enum AgentError {
    // strings represent context
    InvalidIdentifier(String),
    NoDidDoc(String),
    NoPdsEndpointFound(String),
    RecordNotFound(String),
    RecordsNotFound(String),
    RepoNotFound(String),
}

/*
idea: in the future, we could cache lookups by DID to prevent unnecessary lookups
*/
pub struct AtprotoAgent {
    agent: AtpAgent<MemorySessionStore, ReqwestClient>,
}

impl AtprotoAgent {
    async fn did_from_handle(
        &self,
        handle: &Handle,
    ) -> AtResult<Did, com::atproto::identity::resolve_handle::Error> {
        match self
            .agent
            .api
            .com
            .atproto
            .identity
            .resolve_handle(
                com::atproto::identity::resolve_handle::ParametersData {
                    handle: handle.to_owned(),
                }
                .into(),
            )
            .await
        {
            Ok(response) => Ok(response.data.did),
            Err(err) => Err(err),
        }
    }
    async fn get_record(
        &self,
        repo: AtIdentifier,
        collection: Nsid,
        rkey: RecordKey,
    ) -> AtResult<com::atproto::repo::get_record::Output, com::atproto::repo::get_record::Error>
    {
        self.agent
            .api
            .com
            .atproto
            .repo
            .get_record(
                com::atproto::repo::get_record::ParametersData {
                    repo,
                    collection,
                    rkey: String::from(rkey),
                    cid: None,
                }
                .into(),
            )
            .await
    }
    async fn list_records(
        &self,
        repo: AtIdentifier,
        collection: Nsid,
    ) -> AtResult<com::atproto::repo::list_records::Output, com::atproto::repo::list_records::Error>
    {
        self.agent
            .api
            .com
            .atproto
            .repo
            .list_records(
                com::atproto::repo::list_records::ParametersData {
                    repo,
                    collection,
                    limit: None,
                    cursor: None,
                    reverse: None,
                    rkey_end: None,
                    rkey_start: None,
                }
                .into(),
            )
            .await
    }
    async fn describe_repo(
        &self,
        repo: AtIdentifier,
    ) -> AtResult<com::atproto::repo::describe_repo::Output, com::atproto::repo::describe_repo::Error>
    {
        self.agent
            .api
            .com
            .atproto
            .repo
            .describe_repo(com::atproto::repo::describe_repo::ParametersData { repo }.into())
            .await
    }
    async fn set_pds_endpoint_for(&self, repo: &AtIdentifier) -> Result<(), AgentError> {
        if let Ok(did) = match repo {
            AtIdentifier::Did(did) => Ok(did.to_owned()),
            AtIdentifier::Handle(handle) => self.did_from_handle(handle).await,
        } {
            match get_pds_endpoint_for(&did).await {
                Ok(endpoint) => Ok(self.agent.configure_endpoint(endpoint)),
                Err(err) => Err(AgentError::NoPdsEndpointFound(err.to_string())),
            }
        } else {
            Err(AgentError::InvalidIdentifier(String::from(repo.to_owned())))
        }
    }
}

impl AsyncComponent for AtprotoAgent {
    type Init = ();
    type Input = AgentInput;
    type Output = AgentOutput;
    type CommandOutput = AgentCommand;
    type Widgets = ();
    type Root = ();

    fn init_root() -> Self::Root {}

    async fn init(
        init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let agent = AtpAgent::new(
            ReqwestClient::new("https://bsky.social"),
            MemorySessionStore::default(),
        );
        let model = Self { agent };

        AsyncComponentParts { model, widgets: () }
    }

    async fn update(
        &mut self,
        message: Self::Input,
        sender: AsyncComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            AgentInput::GetDidDoc(id) => {
                let identifier = id.clone();
                if let Ok(did) = match id {
                    AtIdentifier::Did(did) => Ok(did),
                    AtIdentifier::Handle(handle) => self.did_from_handle(&handle).await,
                } {
                    match get_did_doc_for(&did).await {
                        Ok(did_doc) => sender.output(AgentOutput::DidDoc(did_doc)),
                        Err(err) => {
                            sender.output(AgentOutput::Error(AgentError::NoDidDoc(err.to_string())))
                        }
                    };
                } else {
                    sender.output(AgentOutput::Error(AgentError::InvalidIdentifier(
                        String::from(identifier),
                    )));
                };
            }
            AgentInput::GetPdsEndpointFor(id) => {
                let identifier = id.clone();
                if let Ok(did) = match id {
                    AtIdentifier::Did(did) => Ok(did),
                    AtIdentifier::Handle(handle) => self.did_from_handle(&handle).await,
                } {
                    match get_pds_endpoint_for(&did).await {
                        Ok(endpoint) => sender.output(AgentOutput::PdsEndpoint(endpoint)),
                        Err(err) => sender.output(AgentOutput::Error(
                            AgentError::NoPdsEndpointFound(err.to_string()),
                        )),
                    };
                } else {
                    sender.output(AgentOutput::Error(AgentError::InvalidIdentifier(
                        String::from(identifier),
                    )));
                };
            }
            AgentInput::GetURI(uri) => {
                if let Err(err) = self.set_pds_endpoint_for(&uri.authority).await {
                    sender.output(AgentOutput::Error(err));
                } else {
                    match (uri.authority, uri.collection, uri.rkey) {
                        (repo, Some(collection), Some(rkey)) => {
                            match self.get_record(repo, collection, rkey).await {
                                Ok(record) => sender.output(AgentOutput::Record(record.data)),
                                Err(err) => sender.output(AgentOutput::Error(
                                    AgentError::RecordNotFound(err.to_string()),
                                )),
                            };
                        }
                        (repo, Some(collection), None) => {
                            match self.list_records(repo, collection).await {
                                Ok(records) => sender.output(AgentOutput::Records(records.data)),
                                Err(err) => sender.output(AgentOutput::Error(
                                    AgentError::RecordsNotFound(err.to_string()),
                                )),
                            };
                        }
                        (repo, None, _) => {
                            match self.describe_repo(repo).await {
                                Ok(repo) => sender.output(AgentOutput::Repo(repo.data)),
                                Err(err) => sender.output(AgentOutput::Error(
                                    AgentError::RepoNotFound(err.to_string()),
                                )),
                            };
                        }
                    }
                }
            }
        }
    }

    fn shutdown(&mut self, _widgets: &mut Self::Widgets, _output: relm4::Sender<Self::Output>) {}
}
