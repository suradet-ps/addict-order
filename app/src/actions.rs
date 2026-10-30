//! Shared UI actions used by the command bar, inspector, and dialogs.

use addict_order_models::{PatientSummary, VisitSummary};
use chrono::{Duration, Local};
use leptos::prelude::*;

use crate::api::{self, ApiError, ApiErrorKind};
use crate::state::{AppState, Banner, ConnectionHealth};

pub fn handle_error(state: &AppState, error: ApiError) {
    if matches!(
        error.kind,
        ApiErrorKind::Connection | ApiErrorKind::NotConfigured
    ) {
        state.health.set(ConnectionHealth::Disconnected);
    }
    state.banner.set(Some(Banner::error(error.message)));
}

pub fn date_range_last_30_days() -> (String, String) {
    let today = Local::now().date_naive();
    let from = today - Duration::days(30);
    (
        from.format("%Y-%m-%d").to_string(),
        today.format("%Y-%m-%d").to_string(),
    )
}

pub fn do_search(state: AppState) {
    let term = state.search_term.get_untracked().trim().to_string();
    if term.is_empty() {
        state
            .banner
            .set(Some(Banner::warn("กรุณากรอก HN, CID หรือชื่อผู้ป่วย")));
        return;
    }
    state.search_busy.set(true);
    state.search_results.set(Vec::new());
    leptos::task::spawn_local(async move {
        match api::search_patients(&term).await {
            Ok(results) => {
                if results.is_empty() {
                    state.banner.set(Some(Banner::info("ไม่พบผู้ป่วยจากคำค้นหานี้")));
                }
                state.search_results.set(results);
            }
            Err(error) => handle_error(&state, error),
        }
        state.search_busy.set(false);
    });
}

pub fn select_patient(state: AppState, summary: PatientSummary) {
    let hn = summary.hn.clone();
    state.patient_busy.set(true);
    state.search_results.set(Vec::new());
    state.search_term.set(String::new());
    leptos::task::spawn_local(async move {
        match api::fetch_patient(&hn).await {
            Ok(Some(patient)) => {
                state.select_patient(patient);
                let (from, to) = date_range_last_30_days();
                match api::fetch_visits(&hn, &from, &to).await {
                    Ok(visits) => {
                        if visits.is_empty() {
                            state
                                .banner
                                .set(Some(Banner::warn("ไม่พบ visit ในช่วง 30 วันที่ผ่านมา")));
                        }
                        state.visits.set(visits);
                    }
                    Err(error) => handle_error(&state, error),
                }
            }
            Ok(None) => state.banner.set(Some(Banner::error("ไม่พบข้อมูลผู้ป่วย"))),
            Err(error) => handle_error(&state, error),
        }
        state.patient_busy.set(false);
    });
}

pub fn select_visit(state: AppState, visit: VisitSummary) {
    state.select_visit(visit);
}

pub fn do_fetch_orders(state: AppState) {
    let Some(visit) = state.selected_visit.get_untracked() else {
        state
            .banner
            .set(Some(Banner::warn("กรุณาเลือก visit ก่อนดึงรายการยา")));
        return;
    };
    if state.catalog_is_empty() {
        state.banner.set(Some(Banner::warn(
            "ยังไม่ได้ตั้งค่าบัญชียาเสพติด กรุณาเพิ่ม icode ในหน้าตั้งค่า",
        )));
        return;
    }
    state.order_busy.set(true);
    let vn = visit.vn.clone();
    leptos::task::spawn_local(async move {
        match api::fetch_narcotic_orders(&vn).await {
            Ok(bundle) => {
                let count = bundle.order_count;
                state.apply_orders(bundle.lines, count, bundle.prescriber);
                if count == 0 {
                    state
                        .banner
                        .set(Some(Banner::info("ไม่พบรายการยาเสพติดที่ตั้งค่าไว้ใน visit นี้")));
                } else {
                    state
                        .banner
                        .set(Some(Banner::info(format!("พบรายการยา {} รายการ", count))));
                }
            }
            Err(error) => handle_error(&state, error),
        }
        state.order_busy.set(false);
    });
}

pub fn open_confirm(state: AppState) {
    if !state.can_confirm() {
        state.banner.set(Some(Banner::error(
            "ยังมีข้อมูลที่ไม่ครบหรือเกิน 5 รายการ กรุณาตรวจสอบรายการแจ้งเตือน",
        )));
        return;
    }
    state.confirm_open.set(true);
}

pub fn do_print(state: AppState) {
    if state.confirmed.get_untracked().is_none() {
        state.banner.set(Some(Banner::error("กรุณายืนยันข้อมูลก่อนพิมพ์")));
        return;
    }
    if let Some(window) = web_sys::window() {
        let _ = window.print();
        state.mark_printed();
    }
}
