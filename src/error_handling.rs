use std::fmt;
use actix_web::{error, HttpRequest, HttpResponse, ResponseError};
use actix_web::error::QueryPayloadError;
use actix_web::http::StatusCode;
use serde::Serialize;
use serde_json::{to_string_pretty, Value};

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

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", to_string_pretty(self).unwrap())
    }
}

impl ResponseError for Error {
    // builds the actual response to send back when an error occurs
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(StatusCode::from_u16(self.status).unwrap()).json(self)
    }
}

pub fn query_error_handler(err: QueryPayloadError, _req: &HttpRequest) -> actix_web::Error {
    let resp = Error::new(
        400,
        "Invalid query parameters",
        "invalid_query",
        Some(&err.to_string()),
    );
    error::InternalError::from_response(err, resp.error_response()).into()
}

pub fn json_error_handler(err: error::JsonPayloadError, _req: &HttpRequest) -> error::Error {
    use actix_web::error::JsonPayloadError;

    let (status, error_type) = match &err {
        JsonPayloadError::ContentType        => (415, "unsupported_media_type"),
        JsonPayloadError::Deserialize(e) if e.is_data() => (422, "invalid_json_body"),
        JsonPayloadError::Deserialize(_)      => (400, "invalid_json_body"),
        JsonPayloadError::Overflow{ limit: _ }=> (413, "payload_too_large"),
        JsonPayloadError::Payload(_)          => (400, "invalid_payload"),
        _                                     => (400, "invalid_json"),
    };

    let resp = Error::new(
        status,
        "Invalid JSON body",
        error_type,
        Some(&err.to_string()),
    );

    error::InternalError::from_response(err, resp.error_response()).into()
}

pub async fn format_es_response(r: Result<elasticsearch::http::response::Response, elasticsearch::Error>) -> Result<Value, Error> {
    let r = r.map_err(|e| Error::new(500, "Failed to execute search", "query_failed", Some(&e.to_string())))?;
    let status_code = r.status_code();
    let value = r
        .json::<Value>()
        .await
        .map_err(|e| Error::new(500, "Failed to parse response body", "parse_failed", Some(&e.to_string())))?;

    if !status_code.is_success() {
        if let Some(error) = value.get("error") {
            let error_type = error.get("type").and_then(Value::as_str).unwrap_or("Unknown error type");
            let reason = error.get("reason").and_then(Value::as_str);
            let detailed_reason = error
                .get("root_cause")
                .and_then(Value::as_array)
                .and_then(|arr| arr.get(0))
                .and_then(|cause| cause.get("reason"))
                .and_then(Value::as_str);
            let detaileder_reason = error
                .get("failed_shards")
                .and_then(Value::as_array)
                .and_then(|arr| arr.get(0))
                .and_then(|cause| cause.get("reason"))
                .and_then(|cause| cause.get("caused_by"))
                .and_then(|cause| cause.get("reason"))
                .and_then(Value::as_str);
            let error_message = format!(
                "Failed to execute search: {}",
                detailed_reason.unwrap_or(reason.unwrap_or("Unknown reason"))
            );
            return Err(Error::new(
                500,
                &error_message,
                error_type,
                detaileder_reason.or(detailed_reason).or(Some("Unknown reason")),
            ));
        }
        return Err(Error::new(500, "Failed to execute search", "query_failed", Some("Unknown error")));
    }

    Ok(value)
}