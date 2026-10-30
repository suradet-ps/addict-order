//! Shared reactive state for the Addict Order frontend.

use addict_order_models::{
    AppSettings, ConfirmedPrescription, ConnectionSummary, HospitalProfile, PatientDetail,
    PatientSummary, Prescriber, PrescriberSource, PrescriptionDraft, PrescriptionLine, SessionUser,
    ValidationIssue, ValidationReport, VisitSummary, validate_draft,
};
use chrono::{Local, NaiveDateTime};
use leptos::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ConnectionHealth {
    Unconfigured,
    Connected,
    Disconnected,
}

impl ConnectionHealth {
    pub fn label_th(self) -> &'static str {
        match self {
            Self::Unconfigured => "ยังไม่ตั้งค่าการเชื่อมต่อ",
            Self::Connected => "เชื่อมต่อ HOSxP แล้ว",
            Self::Disconnected => "การเชื่อมต่อขัดข้อง",
        }
    }

    pub fn dot_class(self) -> &'static str {
        match self {
            Self::Unconfigured => "idle",
            Self::Connected => "ok",
            Self::Disconnected => "down",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsTab {
    Connection,
    Catalog,
    Hospital,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BannerKind {
    Error,
    Warn,
    Info,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Banner {
    pub kind: BannerKind,
    pub message: String,
}

impl Banner {
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            kind: BannerKind::Error,
            message: message.into(),
        }
    }

    pub fn warn(message: impl Into<String>) -> Self {
        Self {
            kind: BannerKind::Warn,
            message: message.into(),
        }
    }

    pub fn info(message: impl Into<String>) -> Self {
        Self {
            kind: BannerKind::Info,
            message: message.into(),
        }
    }
}

#[derive(Clone, Copy)]
pub struct AppState {
    pub settings: RwSignal<Option<AppSettings>>,
    pub configured: RwSignal<bool>,
    pub connection_summary: RwSignal<Option<ConnectionSummary>>,
    pub session: RwSignal<Option<SessionUser>>,
    pub auth_checked: RwSignal<bool>,
    pub health: RwSignal<ConnectionHealth>,
    pub settings_open: RwSignal<bool>,
    pub settings_tab: RwSignal<SettingsTab>,

    pub search_term: RwSignal<String>,
    pub search_busy: RwSignal<bool>,
    pub search_results: RwSignal<Vec<PatientSummary>>,
    pub patient_busy: RwSignal<bool>,
    pub selected_patient: RwSignal<Option<PatientDetail>>,

    pub visits: RwSignal<Vec<VisitSummary>>,
    pub selected_visit: RwSignal<Option<VisitSummary>>,

    pub order_busy: RwSignal<bool>,
    pub lines: RwSignal<Vec<PrescriptionLine>>,
    pub order_count: RwSignal<usize>,
    pub prescriber: RwSignal<Prescriber>,
    pub hospital: RwSignal<HospitalProfile>,

    pub address_edit_open: RwSignal<bool>,
    pub prescriber_edit_open: RwSignal<bool>,
    pub confirm_open: RwSignal<bool>,
    pub confirmed: RwSignal<Option<ConfirmedPrescription>>,
    pub printed_at: RwSignal<Option<NaiveDateTime>>,

    pub banner: RwSignal<Option<Banner>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        Self {
            settings: RwSignal::new(None),
            configured: RwSignal::new(false),
            connection_summary: RwSignal::new(None),
            session: RwSignal::new(None),
            auth_checked: RwSignal::new(false),
            health: RwSignal::new(ConnectionHealth::Unconfigured),
            settings_open: RwSignal::new(false),
            settings_tab: RwSignal::new(SettingsTab::Connection),
            search_term: RwSignal::new(String::new()),
            search_busy: RwSignal::new(false),
            search_results: RwSignal::new(Vec::new()),
            patient_busy: RwSignal::new(false),
            selected_patient: RwSignal::new(None),
            visits: RwSignal::new(Vec::new()),
            selected_visit: RwSignal::new(None),
            order_busy: RwSignal::new(false),
            lines: RwSignal::new(Vec::new()),
            order_count: RwSignal::new(0),
            prescriber: RwSignal::new(Prescriber::default()),
            hospital: RwSignal::new(HospitalProfile::default()),
            address_edit_open: RwSignal::new(false),
            prescriber_edit_open: RwSignal::new(false),
            confirm_open: RwSignal::new(false),
            confirmed: RwSignal::new(None),
            printed_at: RwSignal::new(None),
            banner: RwSignal::new(None),
        }
    }

    pub fn apply_settings(&self, settings: AppSettings) {
        if !settings.hospital.name.trim().is_empty() {
            self.hospital.set(settings.hospital.clone());
        }
        self.settings.set(Some(settings));
    }

    pub fn catalog_is_empty(&self) -> bool {
        self.settings
            .get()
            .map(|settings| settings.catalog.is_empty())
            .unwrap_or(true)
    }

    pub fn clear_session(&self) {
        self.session.set(None);
        self.clear_patient_work();
    }

    pub fn clear_patient_work(&self) {
        self.selected_patient.set(None);
        self.visits.set(Vec::new());
        self.selected_visit.set(None);
        self.lines.set(Vec::new());
        self.order_count.set(0);
        self.prescriber.set(Prescriber::default());
        self.confirmed.set(None);
        self.printed_at.set(None);
    }

    pub fn select_patient(&self, patient: PatientDetail) {
        self.selected_patient.set(Some(patient));
        self.visits.set(Vec::new());
        self.selected_visit.set(None);
        self.lines.set(Vec::new());
        self.order_count.set(0);
        self.prescriber.set(Prescriber::default());
        self.confirmed.set(None);
        self.printed_at.set(None);
    }

    pub fn select_visit(&self, visit: VisitSummary) {
        self.prescriber.set(Prescriber {
            doctor_code: visit.doctor_code.clone(),
            name: visit.doctor_name.clone(),
            license_no: String::new(),
            source: Some(PrescriberSource::Hosxp),
        });
        self.selected_visit.set(Some(visit));
        self.lines.set(Vec::new());
        self.order_count.set(0);
        self.confirmed.set(None);
        self.printed_at.set(None);
    }

    pub fn apply_orders(
        &self,
        lines: Vec<PrescriptionLine>,
        order_count: usize,
        prescriber: Option<Prescriber>,
    ) {
        self.lines.set(lines);
        self.order_count.set(order_count);
        if let Some(prescriber) = prescriber {
            self.prescriber.set(prescriber);
        }
        self.confirmed.set(None);
        self.printed_at.set(None);
    }

    pub fn set_prescriber(&self, prescriber: Prescriber) {
        self.prescriber.set(prescriber);
        self.confirmed.set(None);
    }

    pub fn set_patient(&self, patient: PatientDetail) {
        self.selected_patient.set(Some(patient));
        self.confirmed.set(None);
    }

    pub fn set_hospital(&self, hospital: HospitalProfile) {
        self.hospital.set(hospital);
        self.confirmed.set(None);
    }

    pub fn draft(&self) -> Option<PrescriptionDraft> {
        let patient = self.selected_patient.get()?;
        let visit = self.selected_visit.get()?;
        let lines = self.lines.get();
        let hospital = self.hospital.get();
        let prescriber = self.prescriber.get();
        let prescription_date = visit.visit_date;
        Some(PrescriptionDraft {
            patient,
            visit,
            prescriber,
            hospital,
            prescription_date,
            category_flags: addict_order_models::CategoryFlags::from_lines(&lines),
            lines,
        })
    }

    pub fn validation(&self) -> ValidationReport {
        match self.draft() {
            Some(draft) => validate_draft(&draft),
            None => ValidationReport {
                issues: vec![ValidationIssue::error(
                    "draft",
                    "เลือกผู้ป่วยและ visit ก่อนสร้างใบสั่งจ่าย",
                )],
            },
        }
    }

    pub fn can_confirm(&self) -> bool {
        self.draft().is_some() && self.validation().is_valid()
    }

    pub fn confirm(&self) -> bool {
        let Some(draft) = self.draft() else {
            return false;
        };
        if !self.validation().is_valid() {
            return false;
        }
        self.confirmed
            .set(Some(draft.snapshot(Local::now().naive_local())));
        self.printed_at.set(None);
        true
    }

    pub fn invalidate_confirmation(&self) {
        self.confirmed.set(None);
        self.printed_at.set(None);
    }

    pub fn mark_printed(&self) {
        self.printed_at.set(Some(Local::now().naive_local()));
    }
}
