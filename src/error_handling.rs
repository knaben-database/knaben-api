use std::fmt::{Display, Formatter, Result as FmtResult};
use actix_web::{error, HttpRequest, HttpResponse, ResponseError};
use actix_web::http::StatusCode;
use serde::Serialize;
use serde_json::{to_string_pretty};

#[derive(Debug, Serialize)]
pub struct Error {
    status: u16,
    error: ErrorDetails,
}

#[derive(Debug, Serialize)]
struct ErrorDetails {
    #[serde(rename = "type")]
    error_type: String,
    reason: String,
    caused_by: CausedBy,
}

#[derive(Debug, Serialize)]
struct CausedBy {
    #[serde(rename = "type")]
    error_type: String,
    reason: String,
}

impl Error {
    pub fn new(status: u16, reason: &str, error_type: &str, caused_by_reason: Option<&str>) -> Self {
        Error {
            status,
            error: ErrorDetails {
                error_type: "middleware_exception".to_owned(),
                reason: reason.to_owned(),
                caused_by: CausedBy {
                    error_type: error_type.to_owned(),
                    reason: caused_by_reason.unwrap_or(reason).to_owned(),
                },
            },
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", to_string_pretty(self).unwrap())
    }
}

impl ResponseError for Error {
    // builds the actual response to send back when an error occurs
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(StatusCode::from_u16(self.status).unwrap()).json(self)
    }
}

pub fn json_error_handler(err: error::JsonPayloadError, _req: &HttpRequest) -> error::Error {
    use actix_web::error::JsonPayloadError;

    let detail = err.to_string();
    let resp = match &err {
        JsonPayloadError::ContentType => HttpResponse::UnsupportedMediaType().body(detail),
        JsonPayloadError::Deserialize(json_err) if json_err.is_data() => {
            HttpResponse::UnprocessableEntity().body(detail)
        }
        _ => HttpResponse::BadRequest().body(detail),
    };
    error::InternalError::from_response(err, resp).into()
}