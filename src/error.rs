use r2d2;

#[derive(Debug)]

pub enum Error {
    ReqwestError(reqwest::Error),
    JsonError(serde_json::Error),
    DatabaseError(diesel::result::Error),
    PoolError(r2d2::Error),
    MiscError(String), // Just a temp
    HttpSigError(http_signature_normalization::PrepareVerifyError)
}

impl From<&str> for Error {
    fn from(err: &str) -> Error {
        Error::MiscError(err.to_owned())
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Error {
        Error::ReqwestError(err)
    }
}

impl From<r2d2::Error> for Error {
    fn from(err: r2d2::Error) -> Error {
        Error::PoolError(err)
    }
}

impl From<diesel::result::Error> for Error {
    fn from(err: diesel::result::Error) -> Error {
        Error::DatabaseError(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Error {
        Error::JsonError(err)
    }
}
impl From<http_signature_normalization::PrepareVerifyError> for Error {
    fn from(err: http_signature_normalization::PrepareVerifyError) -> Error {
        Error::HttpSigError(err)
    }
}
