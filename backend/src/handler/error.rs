use std::fmt::Display;

use poem::http::StatusCode;

pub trait WrapRespErr<T, E> {
    fn wrap_resp_err<D>(self, status: StatusCode, msg: D) -> Result<T, (StatusCode, eyre::Report)>
    where
        D: Display + Send + Sync + 'static;
}

impl<T, E> WrapRespErr<T, E> for Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn wrap_resp_err<D>(self, status: StatusCode, msg: D) -> Result<T, (StatusCode, eyre::Report)>
    where
        D: Display + Send + Sync + 'static,
    {
        match self {
            Ok(t) => Ok(t),
            Err(e) => Err((status, eyre::Report::new(e).wrap_err(msg))),
        }
    }
}
