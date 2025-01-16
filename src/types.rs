use atrium_api::types::string::{AtIdentifier, Nsid, RecordKey};
use std::str::FromStr;

#[derive(Debug)]
pub enum AtUriError {
    MissingAuthority,
    InvalidAuthority,
    InvalidNsid,
    InvalidRecordKey,
}

#[derive(Debug)]
pub struct AtUri {
    pub authority: AtIdentifier,
    pub collection: Option<Nsid>,
    pub rkey: Option<RecordKey>,
}

impl FromStr for AtUri {
    type Err = AtUriError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let components = s
            .strip_prefix("web+")
            .unwrap_or(s)
            .strip_prefix("at://")
            .unwrap_or(s)
            .splitn(3, '/')
            .collect::<Vec<&str>>();

        let authority = components
            .first()
            .ok_or(AtUriError::MissingAuthority)?
            .to_string()
            .parse::<AtIdentifier>()
            .map_err(|_| AtUriError::InvalidAuthority)?;

        let collection = components
            .get(1)
            .map(|s| s.to_string().parse::<Nsid>())
            .transpose()
            .map_err(|_| AtUriError::InvalidNsid)?;

        let rkey = components
            .get(2)
            .map(|s| s.to_string().parse::<RecordKey>())
            .transpose()
            .map_err(|_| AtUriError::InvalidRecordKey)?;

        Ok(AtUri {
            authority,
            collection,
            rkey,
        })
    }
}

impl std::fmt::Display for AtUri {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut uri = String::from(self.authority.as_ref());
        match self.collection.as_ref() {
            Some(s) => {
                uri.push('/');
                uri.push_str(s.as_str())
            }
            None => (),
        };
        match self.rkey.as_ref() {
            Some(s) => {
                uri.push('/');
                uri.push_str(s.as_str())
            }
            None => (),
        };
        write!(f, "at://{}", uri)
    }
}
