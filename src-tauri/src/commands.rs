//! Tauri command adapters. Validate, delegate, translate.
//!
//! No SQL and no business logic live here. HOSxP access goes through
//! `addict-order-hosxp-connector`; this file is the Thai error boundary.

use std::time::{Duration, Instant};

use addict_order_core::{
    FormError, HosxRepository, RepositoryError, build_lines, classify_patient_query,
    verify_password,
};
use addict_order_hosxp_connector::config::{HosxConfig, load_vault};
use addict_order_hosxp_connector::{ConnectorRepository, MySqlPool, connect as connect_hosxp};
use addict_order_models::{
    AppSettings, Catalog, ConnectionSummary, DrugItem, HospitalProfile, HospitalSource, OrderLine,
    PatientDetail, PatientSummary, Prescriber, PrescriberSource, PrescriptionLine, SessionUser,
    VisitSummary,
};
use chrono::NaiveDate;
use secrecy::{ExposeSecret, SecretBox};
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::error::CommandError;
use crate::settings;
use crate::state::{AppState, ConnectionHealth};

const HEALTH_POLL_INTERVAL: Duration = Duration::from_secs(30);
const SEARCH_TERM_MIN_CHARS: usize = 2;
const SEARCH_TERM_MAX_CHARS: usize = 100;
const VISIT_RANGE_MAX_DAYS: i64 = 366;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionInput {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub user: String,
    pub password: String,
}

impl ConnectionInput {
    fn validate(&self) -> Result<(), CommandError> {
        if self.host.trim().is_empty() {
            return Err(CommandError::validation("กรุณากรอกที่อยู่เซิร์ฟเวอร์"));
        }
        if self.port == 0 {
            return Err(CommandError::validation("พอร์ตต้องอยู่ระหว่าง 1-65535"));
        }
        if self.database.trim().is_empty() {
            return Err(CommandError::validation("กรุณากรอกชื่อฐานข้อมูล"));
        }
        if self.user.trim().is_empty() {
            return Err(CommandError::validation("กรุณากรอกชื่อผู้ใช้"));
        }
        Ok(())
    }

    fn into_config(self) -> HosxConfig {
        HosxConfig::new(
            self.host.trim().to_string(),
            self.port,
            self.database.trim().to_string(),
            self.user.trim().to_string(),
            SecretBox::from(self.password),
        )
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionTestResult {
    pub latency_ms: u64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginInput {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderBundle {
    pub lines: Vec<PrescriptionLine>,
    pub prescriber: Option<Prescriber>,
    pub order_count: usize,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VisitQuery {
    pub hn: String,
    pub from: String,
    pub to: String,
}

#[tauri::command]
pub async fn connection_status(state: State<'_, AppState>) -> Result<bool, CommandError> {
    Ok(state.config_path().exists())
}

#[tauri::command]
pub async fn connection_health(
    state: State<'_, AppState>,
) -> Result<ConnectionHealth, CommandError> {
    Ok(*state.health.lock().await)
}

#[tauri::command]
pub async fn connection_summary(
    state: State<'_, AppState>,
) -> Result<Option<ConnectionSummary>, CommandError> {
    if !state.config_path().exists() {
        return Ok(None);
    }
    let config = load_config(&state).await?;
    Ok(Some(ConnectionSummary {
        host: config.host,
        port: config.port,
        database: config.database,
        user: config.user,
    }))
}

#[tauri::command]
pub async fn test_connection(input: ConnectionInput) -> Result<ConnectionTestResult, CommandError> {
    input.validate()?;
    let config = input.into_config();
    let started = Instant::now();
    let pool = connect_hosxp(&config).await.map_err(CommandError::from)?;
    ConnectorRepository::new(pool)
        .ping()
        .await
        .map_err(CommandError::from)?;
    Ok(ConnectionTestResult {
        latency_ms: started.elapsed().as_millis() as u64,
    })
}

#[tauri::command]
pub async fn connect(
    input: ConnectionInput,
    state: State<'_, AppState>,
) -> Result<ConnectionTestResult, CommandError> {
    input.validate()?;
    let config = input.into_config();
    let started = Instant::now();
    let pool = match connect_hosxp(&config).await {
        Ok(pool) => pool,
        Err(error) => {
            *state.health.lock().await = ConnectionHealth::Disconnected;
            return Err(error.into());
        }
    };
    if let Err(error) = ConnectorRepository::new(pool.clone()).ping().await {
        *state.pool.lock().await = None;
        *state.health.lock().await = ConnectionHealth::Disconnected;
        return Err(error.into());
    }
    let vault = load_vault().map_err(CommandError::from)?;
    addict_order_hosxp_connector::config::save_encrypted(&state.config_path(), &vault, &config)
        .await
        .map_err(CommandError::from)?;
    *state.pool.lock().await = Some(pool);
    *state.health.lock().await = ConnectionHealth::Connected;
    *state.session.lock().await = None;
    Ok(ConnectionTestResult {
        latency_ms: started.elapsed().as_millis() as u64,
    })
}

#[tauri::command]
pub async fn login(
    input: LoginInput,
    state: State<'_, AppState>,
) -> Result<SessionUser, CommandError> {
    let username = input.username.trim();
    if username.is_empty() {
        return Err(CommandError::validation("กรุณากรอกชื่อผู้ใช้"));
    }
    if username.chars().count() > 100 {
        return Err(CommandError::validation("ชื่อผู้ใช้ยาวเกินไป"));
    }
    let password = SecretBox::from(input.password);
    if password.expose_secret().is_empty() {
        return Err(CommandError::validation("กรุณากรอกรหัสผ่าน"));
    }
    if password.expose_secret().chars().count() > 200 {
        return Err(CommandError::validation("รหัสผ่านยาวเกินไป"));
    }

    *state.session.lock().await = None;
    let repository = acquire_repository(&state).await?;
    let account = repository.fetch_officer(username).await?;
    let Some(account) = account else {
        return Err(CommandError::auth("ชื่อผู้ใช้หรือรหัสผ่านไม่ถูกต้อง"));
    };
    if account.auth_type_id != 1 {
        return Err(CommandError::auth(
            "บัญชีนี้ใช้การยืนยันตัวตนแบบ LDAP ซึ่งยังไม่เปิดใช้งาน",
        ));
    }
    if !account.active.eq_ignore_ascii_case("Y") {
        return Err(CommandError::auth("บัญชีนี้ถูกปิดใช้งาน กรุณาติดต่อผู้ดูแลระบบ"));
    }
    if !verify_password(password.expose_secret(), &account.password_md5) {
        return Err(CommandError::auth("ชื่อผู้ใช้หรือรหัสผ่านไม่ถูกต้อง"));
    }

    let display_name = if account.display_name.trim().is_empty() {
        account.login_name.clone()
    } else {
        account.display_name
    };
    let user = SessionUser {
        officer_id: account.officer_id,
        login_name: account.login_name,
        display_name,
        doctor_code: account.doctor_code,
        role: "STAFF".to_string(),
    };
    *state.session.lock().await = Some(user.clone());
    Ok(user)
}

#[tauri::command]
pub async fn logout(state: State<'_, AppState>) -> Result<(), CommandError> {
    *state.session.lock().await = None;
    Ok(())
}

#[tauri::command]
pub async fn current_session(
    state: State<'_, AppState>,
) -> Result<Option<SessionUser>, CommandError> {
    Ok(state.session.lock().await.clone())
}

#[tauri::command]
pub async fn load_settings(state: State<'_, AppState>) -> Result<AppSettings, CommandError> {
    Ok(state.settings.lock().await.clone())
}

#[tauri::command]
pub async fn save_hospital(
    hospital: HospitalProfile,
    state: State<'_, AppState>,
) -> Result<(), CommandError> {
    require_session(&state).await?;
    if hospital.name.trim().is_empty() {
        return Err(CommandError::validation("กรุณากรอกชื่อสถานพยาบาล"));
    }
    let mut settings = state.settings.lock().await;
    settings.hospital = hospital;
    settings::save(&state.settings_path(), &settings)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn save_catalog(
    catalog: Catalog,
    state: State<'_, AppState>,
) -> Result<(), CommandError> {
    require_session(&state).await?;
    let errors = catalog.validation_errors();
    if !errors.is_empty() {
        return Err(CommandError::validation(errors.join("; ")));
    }
    let mut seen = std::collections::HashSet::new();
    for drug in &catalog.drugs {
        if !seen.insert(drug.icode.trim().to_string()) {
            return Err(CommandError::validation(format!(
                "icode {} ซ้ำในบัญชี",
                drug.icode
            )));
        }
    }
    let mut settings = state.settings.lock().await;
    settings.catalog = catalog;
    settings::save(&state.settings_path(), &settings)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn search_patients(
    term: String,
    state: State<'_, AppState>,
) -> Result<Vec<PatientSummary>, CommandError> {
    require_session(&state).await?;
    let term = term.trim();
    let chars = term.chars().count();
    if chars == 0 {
        return Err(CommandError::validation("กรุณากรอกคำค้นหา"));
    }
    if chars > SEARCH_TERM_MAX_CHARS {
        return Err(CommandError::validation("คำค้นหายาวเกินไป"));
    }
    let query = classify_patient_query(term);
    let repository = acquire_repository(&state).await?;
    match repository.search_patients(&query).await {
        Ok(rows) => Ok(rows),
        Err(error) => {
            note_query_outcome(&state, &error).await;
            Err(error.into())
        }
    }
}

#[tauri::command]
pub async fn fetch_patient(
    hn: String,
    state: State<'_, AppState>,
) -> Result<Option<PatientDetail>, CommandError> {
    require_session(&state).await?;
    let hn = hn.trim();
    if hn.is_empty() {
        return Err(CommandError::validation("ไม่พบ HN"));
    }
    let repository = acquire_repository(&state).await?;
    match repository.fetch_patient(hn).await {
        Ok(patient) => Ok(patient),
        Err(error) => {
            note_query_outcome(&state, &error).await;
            Err(error.into())
        }
    }
}

#[tauri::command]
pub async fn fetch_visits(
    query: VisitQuery,
    state: State<'_, AppState>,
) -> Result<Vec<VisitSummary>, CommandError> {
    require_session(&state).await?;
    let hn = query.hn.trim();
    if hn.is_empty() {
        return Err(CommandError::validation("ไม่พบ HN"));
    }
    let from = NaiveDate::parse_from_str(query.from.trim(), "%Y-%m-%d")
        .map_err(|_| CommandError::validation("วันที่เริ่มต้นไม่ถูกต้อง"))?;
    let to = NaiveDate::parse_from_str(query.to.trim(), "%Y-%m-%d")
        .map_err(|_| CommandError::validation("วันที่สิ้นสุดไม่ถูกต้อง"))?;
    if from > to {
        return Err(CommandError::validation("ช่วงวันที่ไม่ถูกต้อง"));
    }
    if (to - from).num_days() > VISIT_RANGE_MAX_DAYS {
        return Err(CommandError::validation("ช่วงวันที่กว้างเกินไป"));
    }
    let repository = acquire_repository(&state).await?;
    match repository.fetch_visits(hn, from, to).await {
        Ok(visits) => Ok(visits),
        Err(error) => {
            note_query_outcome(&state, &error).await;
            Err(error.into())
        }
    }
}

#[tauri::command]
pub async fn fetch_narcotic_orders(
    vn: String,
    state: State<'_, AppState>,
) -> Result<OrderBundle, CommandError> {
    require_session(&state).await?;
    let vn = vn.trim();
    if vn.is_empty() {
        return Err(CommandError::validation("ไม่พบ visit"));
    }
    let settings = state.settings.lock().await.clone();
    let icodes = settings.catalog.icodes();
    if icodes.is_empty() {
        return Err(CommandError::validation(
            "ยังไม่ได้ตั้งค่าบัญชียาเสพติด กรุณาเพิ่ม icode ในหน้าตั้งค่า",
        ));
    }
    let repository = acquire_repository(&state).await?;
    let orders = match repository.fetch_narcotic_orders(vn, &icodes).await {
        Ok(orders) => orders,
        Err(error) => {
            note_query_outcome(&state, &error).await;
            return Err(error.into());
        }
    };
    let prescriber = prescriber_from_orders(&orders);
    let lines = build_lines(&orders, &settings.catalog).map_err(map_form_error)?;
    Ok(OrderBundle {
        lines,
        prescriber,
        order_count: orders.len(),
    })
}

#[tauri::command]
pub async fn search_drugs(
    term: String,
    state: State<'_, AppState>,
) -> Result<Vec<DrugItem>, CommandError> {
    require_session(&state).await?;
    let term = term.trim();
    let chars = term.chars().count();
    if chars < SEARCH_TERM_MIN_CHARS {
        return Err(CommandError::validation("พิมพ์อย่างน้อย 2 ตัวอักษรเพื่อค้นหายา"));
    }
    if chars > SEARCH_TERM_MAX_CHARS {
        return Err(CommandError::validation("คำค้นหายาวเกินไป"));
    }
    let repository = acquire_repository(&state).await?;
    match repository.search_drugs(term).await {
        Ok(rows) => Ok(rows),
        Err(error) => {
            note_query_outcome(&state, &error).await;
            Err(error.into())
        }
    }
}

#[tauri::command]
pub async fn search_doctors(
    term: String,
    state: State<'_, AppState>,
) -> Result<Vec<Prescriber>, CommandError> {
    require_session(&state).await?;
    let term = term.trim();
    if term.is_empty() {
        return Err(CommandError::validation("กรุณากรอกชื่อหรือรหัสแพทย์"));
    }
    if term.chars().count() > SEARCH_TERM_MAX_CHARS {
        return Err(CommandError::validation("คำค้นหายาวเกินไป"));
    }
    let repository = acquire_repository(&state).await?;
    match repository.search_doctors(term).await {
        Ok(rows) => Ok(rows),
        Err(error) => {
            note_query_outcome(&state, &error).await;
            Err(error.into())
        }
    }
}

#[tauri::command]
pub async fn fetch_hospital_source(
    state: State<'_, AppState>,
) -> Result<Option<HospitalSource>, CommandError> {
    require_session(&state).await?;
    let repository = acquire_repository(&state).await?;
    match repository.fetch_hospital_source().await {
        Ok(source) => Ok(source),
        Err(error) => {
            note_query_outcome(&state, &error).await;
            Err(error.into())
        }
    }
}

fn prescriber_from_orders(orders: &[OrderLine]) -> Option<Prescriber> {
    orders.iter().find_map(|order| {
        if order.doctor_code.is_none() && order.doctor_name.trim().is_empty() {
            return None;
        }
        Some(Prescriber {
            doctor_code: order.doctor_code.clone(),
            name: order.doctor_name.clone(),
            license_no: order.doctor_license_no.clone().unwrap_or_default(),
            source: Some(PrescriberSource::Hosxp),
        })
    })
}

fn map_form_error(error: FormError) -> CommandError {
    match error {
        FormError::MissingCatalogEntry(icode) => {
            CommandError::validation(format!("ไม่พบ icode {} ในบัญชียา", icode))
        }
        FormError::MissingLawCategory(icode) => {
            CommandError::validation(format!("icode {} ยังไม่ระบุประเภททางกฎหมาย", icode))
        }
        FormError::InvalidLine => CommandError::validation("รายการยาบางรายการไม่มีข้อมูลพอสำหรับพิมพ์"),
    }
}

async fn load_config(state: &AppState) -> Result<HosxConfig, CommandError> {
    if !state.config_path().exists() {
        return Err(CommandError::not_configured());
    }
    let vault = load_vault().map_err(CommandError::from)?;
    addict_order_hosxp_connector::config::load(&state.config_path(), &vault)
        .await
        .map_err(CommandError::from)?
        .ok_or_else(CommandError::not_configured)
}

async fn ensure_pool(state: &AppState, config: &HosxConfig) -> Result<MySqlPool, CommandError> {
    let mut guard = state.pool.lock().await;
    if let Some(pool) = guard.as_ref() {
        return Ok(pool.clone());
    }
    let pool = connect_hosxp(config).await.map_err(CommandError::from)?;
    *guard = Some(pool.clone());
    Ok(pool)
}

async fn acquire_repository(state: &AppState) -> Result<ConnectorRepository, CommandError> {
    let config = load_config(state).await?;
    let pool = ensure_pool(state, &config).await?;
    Ok(ConnectorRepository::new(pool))
}

async fn require_session(state: &AppState) -> Result<SessionUser, CommandError> {
    state
        .session
        .lock()
        .await
        .clone()
        .ok_or_else(CommandError::unauthorized)
}

async fn note_query_outcome(state: &AppState, error: &RepositoryError) {
    if matches!(
        error,
        RepositoryError::Connection | RepositoryError::NotConfigured
    ) {
        *state.health.lock().await = ConnectionHealth::Disconnected;
    }
}

pub async fn run_health_monitor(state: AppState) {
    loop {
        let health = probe_health(&state).await;
        *state.health.lock().await = health;
        tokio::time::sleep(HEALTH_POLL_INTERVAL).await;
    }
}

async fn probe_health(state: &AppState) -> ConnectionHealth {
    if !state.config_path().exists() {
        return ConnectionHealth::Unconfigured;
    }
    let Ok(vault) = load_vault() else {
        return ConnectionHealth::Disconnected;
    };
    let Ok(Some(config)) =
        addict_order_hosxp_connector::config::load(&state.config_path(), &vault).await
    else {
        return ConnectionHealth::Disconnected;
    };
    match ensure_pool(state, &config).await {
        Ok(pool) => match ConnectorRepository::new(pool).ping().await {
            Ok(()) => ConnectionHealth::Connected,
            Err(_) => {
                *state.pool.lock().await = None;
                ConnectionHealth::Disconnected
            }
        },
        Err(_) => ConnectionHealth::Disconnected,
    }
}
