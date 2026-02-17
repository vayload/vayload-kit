use serde::Deserialize;

#[allow(unused)]
#[derive(Debug, Deserialize)]
pub struct UploadResponse {
    pub success: bool,
    pub message: String,
    pub id: String,
    pub version: String,
    pub checksum: String,
}

#[derive(Debug)]
pub struct DownloadMeta {
    pub id: String,
    pub version: String,
    pub checksum: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct JsonResponse<T> {
    pub data: T,
    #[allow(unused)]
    pub meta: Option<JsonResponseMeta>,
}

#[allow(unused)]
#[derive(Debug, Deserialize)]
pub struct JsonResponseMeta {
    pub request_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ErrorResponse {
    pub error: ApiError,
    #[allow(unused)]
    pub meta: Option<ApiErrorMeta>,
}

#[allow(unused)]
#[derive(Debug, Deserialize)]
pub struct ApiError {
    pub message: String,
    pub code: String,
    pub sub_code: Option<String>,
    pub details: Option<serde_json::Value>,
}

#[allow(unused)]
#[derive(Debug, Deserialize)]
pub struct ApiErrorMeta {
    pub request_id: String,
}
