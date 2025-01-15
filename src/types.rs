use std::str::FromStr;

#[derive(Debug)]
pub enum AtUriError {
    MissingIdentifier,
}

#[derive(Debug)]
pub struct AtUri {
    pub authority: String,
    pub collection: Option<String>,
    pub rkey: Option<String>,
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
        /* alright so here's where i'm leaving off bc i'm sleepy
        i want FromStr to validate the URI components, but only the handle/did is required
        so we need to ensure that there is a valid identifier (and error if not), then see if
        there is a valid nsid and rkey, putting them there if valid, and returning None if not

        some questions:
        - this should probably be in its own file and exported as a type
        - do i just match nsid and rkey and transform the err into a None?
        - i can't even think of the questions i'm so tired
        - maybe i'll have better brain tomorrow
        */
        let authority: String = components
            .first()
            .ok_or(AtUriError::MissingIdentifier)?
            .to_string();

        let collection = components.get(1).map(|s| s.to_string());
        let rkey = components.get(2).map(|s| s.to_string());

        Ok(AtUri {
            authority,
            collection,
            rkey,
        })
    }
}

impl std::fmt::Display for AtUri {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut uri = self.authority.clone();
        match self.collection.clone() {
            Some(s) => {
                uri.push('/');
                uri.push_str(s.as_str())
            }
            None => (),
        };
        match self.rkey.clone() {
            Some(s) => {
                uri.push('/');
                uri.push_str(s.as_str())
            }
            None => (),
        };
        write!(f, "at://{}", uri)
    }
}
