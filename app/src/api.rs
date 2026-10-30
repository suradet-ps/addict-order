//! Frontend API layer: the only place the webview talks to the Tauri backend.

use addict_order_models::{
    AppSettings, Catalog, ConnectionSummary, DrugItem, HospitalProfile, HospitalSource,
    PatientDetail, PatientSummary, Prescriber, PrescriptionLine, SessionUser, VisitSummary,
};
use js_sys::{Object, Reflect};
use serde_wasm_bindgen::{from_value, to_value};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

use crate::state::ConnectionHealth;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["__TAURI_INTERNALS__"])]
    fn invoke(cmd: &str, args: JsValue) -> js_sys::Promise;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ApiErrorKind {
    NotConfigured,
    Connection,
    Guard,
    Query,
    Validation,
    Auth,
    Storage,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiError {
    pub kind: ApiErrorKind,
    pub message: String,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionInput {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub user: String,
    pub password: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionTestResult {
    pub latency_ms: u64,
}

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginInput {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderBundle {
    pub lines: Vec<PrescriptionLine>,
    pub prescriber: Option<Prescriber>,
    pub order_count: usize,
}

fn request_build_error() -> ApiError {
    ApiError {
        kind: ApiErrorKind::Query,
        message: "สร้างคำขอไม่สำเร็จ".to_string(),
    }
}

async fn call_empty<T>(cmd: &str) -> Result<T, ApiError>
where
    T: serde::de::DeserializeOwned,
{
    let args = to_value(&std::collections::HashMap::<String, String>::new())
        .map_err(|_| request_build_error())?;
    call_raw(cmd, args).await
}

async fn call_string_arg<T>(cmd: &str, arg_name: &str, value: &str) -> Result<T, ApiError>
where
    T: serde::de::DeserializeOwned,
{
    let mut args = std::collections::HashMap::new();
    args.insert(arg_name, value);
    call_raw(cmd, to_value(&args).map_err(|_| request_build_error())?).await
}

async fn call_struct_arg<T>(
    cmd: &str,
    arg_name: &str,
    arg: &impl serde::Serialize,
) -> Result<T, ApiError>
where
    T: serde::de::DeserializeOwned,
{
    let arg_value = to_value(arg).map_err(|_| request_build_error())?;
    let args = Object::new();
    Reflect::set(&args, &JsValue::from_str(arg_name), &arg_value)
        .map_err(|_| request_build_error())?;
    call_raw(cmd, args.into()).await
}

async fn call_raw<T>(cmd: &str, args: JsValue) -> Result<T, ApiError>
where
    T: serde::de::DeserializeOwned,
{
    let promise = invoke(cmd, args);
    let value = JsFuture::from(promise).await.map_err(|err| {
        from_value::<ApiError>(err).unwrap_or(ApiError {
            kind: ApiErrorKind::Query,
            message: "การเชื่อมต่อกับโปรแกรมหลักล้มเหลว".to_string(),
        })
    })?;
    from_value(value).map_err(|error| {
        #[cfg(debug_assertions)]
        web_sys::console::error_1(&JsValue::from_str(&format!("deserialize {cmd}: {error}")));
        ApiError {
            kind: ApiErrorKind::Query,
            message: "อ่านผลลัพธ์ไม่สำเร็จ".to_string(),
        }
    })
}

pub async fn connection_status() -> bool {
    call_empty::<bool>("connection_status")
        .await
        .unwrap_or(false)
}

pub async fn connection_summary() -> Result<Option<ConnectionSummary>, ApiError> {
    call_empty("connection_summary").await
}

pub async fn connection_health() -> Result<ConnectionHealth, ApiError> {
    call_empty("connection_health").await
}

pub async fn test_connection(input: &ConnectionInput) -> Result<u64, ApiError> {
    let result: ConnectionTestResult = call_struct_arg("test_connection", "input", input).await?;
    Ok(result.latency_ms)
}

pub async fn connect(input: &ConnectionInput) -> Result<u64, ApiError> {
    let result: ConnectionTestResult = call_struct_arg("connect", "input", input).await?;
    Ok(result.latency_ms)
}

pub async fn login(username: &str, password: &str) -> Result<SessionUser, ApiError> {
    call_struct_arg(
        "login",
        "input",
        &LoginInput {
            username: username.to_string(),
            password: password.to_string(),
        },
    )
    .await
}

pub async fn logout() -> Result<(), ApiError> {
    call_empty("logout").await
}

pub async fn current_session() -> Result<Option<SessionUser>, ApiError> {
    call_empty("current_session").await
}

pub async fn load_settings() -> Result<AppSettings, ApiError> {
    call_empty("load_settings").await
}

pub async fn save_hospital(hospital: &HospitalProfile) -> Result<(), ApiError> {
    call_struct_arg("save_hospital", "hospital", hospital).await
}

pub async fn save_catalog(catalog: &Catalog) -> Result<(), ApiError> {
    call_struct_arg("save_catalog", "catalog", catalog).await
}

pub async fn search_patients(term: &str) -> Result<Vec<PatientSummary>, ApiError> {
    call_string_arg("search_patients", "term", term).await
}

pub async fn fetch_patient(hn: &str) -> Result<Option<PatientDetail>, ApiError> {
    call_string_arg("fetch_patient", "hn", hn).await
}

pub async fn fetch_visits(hn: &str, from: &str, to: &str) -> Result<Vec<VisitSummary>, ApiError> {
    #[derive(serde::Serialize)]
    struct VisitQuery<'a> {
        hn: &'a str,
        from: &'a str,
        to: &'a str,
    }
    call_struct_arg("fetch_visits", "query", &VisitQuery { hn, from, to }).await
}

pub async fn fetch_narcotic_orders(vn: &str) -> Result<OrderBundle, ApiError> {
    call_string_arg("fetch_narcotic_orders", "vn", vn).await
}

pub async fn search_drugs(term: &str) -> Result<Vec<DrugItem>, ApiError> {
    call_string_arg("search_drugs", "term", term).await
}

pub async fn search_doctors(term: &str) -> Result<Vec<Prescriber>, ApiError> {
    call_string_arg("search_doctors", "term", term).await
}

pub async fn fetch_hospital_source() -> Result<Option<HospitalSource>, ApiError> {
    call_empty("fetch_hospital_source").await
}
