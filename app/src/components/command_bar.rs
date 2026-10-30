//! Command bar: patient search, visit selection, prescriber, document actions.

use addict_order_models::{format_thai_date, format_time_hm, mask_cid};
use leptos::prelude::*;

use crate::actions;
use crate::components::icons::{IconCheck, IconPrint, IconSearch, IconUser};
use crate::state::AppState;

#[component]
pub fn CommandBar(state: AppState) -> impl IntoView {
    let on_search = {
        let state = state.clone();
        move |ev: web_sys::SubmitEvent| {
            ev.prevent_default();
            actions::do_search(state.clone());
        }
    };

    let on_visit_change = {
        let state = state.clone();
        move |ev| {
            let vn = event_target_value(&ev);
            if let Some(visit) = state.visits.get().into_iter().find(|visit| visit.vn == vn) {
                actions::select_visit(state.clone(), visit);
            }
        }
    };

    view! {
        <section class="command-bar">
            <div class="command-bar__search-row">
                <form class="search" on:submit=on_search>
                    <span class="search__icon"><IconSearch /></span>
                    <input
                        class="search__input"
                        type="text"
                        placeholder="ค้นหา HN 7 หลัก / CID 13 หลัก / ชื่อ-สกุล"
                        autocomplete="off"
                        prop:value=move || state.search_term.get()
                        on:input=move |ev| state.search_term.set(event_target_value(&ev))
                    />
                    <button
                        class="button-blue search__submit"
                        type="submit"
                        disabled=move || state.search_busy.get()
                    >
                        {move || if state.search_busy.get() { "กำลังค้นหา..." } else { "ค้นหา" }}
                    </button>
                </form>
                <span class="command-bar__hint">
                    "HN 7 หลัก · CID 13 หลัก · ชื่อหรือนามสกุล"
                </span>
            </div>

            {move || {
                let results = state.search_results.get();
                (!results.is_empty()).then(|| {
                    view! {
                        <div class="search-results" role="listbox">
                            <div class="search-results__head">
                                <span>"ผลการค้นหา"</span>
                                <span class="mono">{results.len()}</span>
                            </div>
                            {results
                                .into_iter()
                                .map(|patient| {
                                    let summary = patient.clone();
                                    let state = state.clone();
                                    view! {
                                        <button
                                            class="search-result"
                                            type="button"
                                            role="option"
                                            on:click=move |_| actions::select_patient(state.clone(), summary.clone())
                                        >
                                            <span class="search-result__hn mono">{patient.hn.clone()}</span>
                                            <span class="search-result__name">{patient.full_name_th.clone()}</span>
                                            <span class="search-result__cid mono">
                                                {patient.cid.as_deref().map(mask_cid).unwrap_or_default()}
                                            </span>
                                        </button>
                                    }
                                })
                                .collect_view()}
                        </div>
                    }
                })
            }}

            <div class="command-bar__context">
                {move || {
                    state.selected_patient.get().map(|patient| {
                        let clear_state = state.clone();
                        view! {
                            <div class="patient-chip">
                                <span class="patient-chip__label">"ผู้ป่วย"</span>
                                <span class="patient-chip__hn mono">{patient.hn.clone()}</span>
                                <span class="patient-chip__name">{patient.full_name_th.clone()}</span>
                                <span class="patient-chip__cid mono">
                                    {patient.cid.as_deref().map(mask_cid).unwrap_or_default()}
                                </span>
                                <button
                                    class="patient-chip__change"
                                    type="button"
                                    on:click=move |_| clear_state.clear_patient_work()
                                >
                                    "เปลี่ยนผู้ป่วย"
                                </button>
                            </div>
                        }
                    })
                }}

                <div class="visit-picker">
                    <span class="visit-picker__label">"Visit"</span>
                    <select
                        class="select visit-picker__select"
                        prop:value=move || {
                            state.selected_visit.get().map(|visit| visit.vn).unwrap_or_default()
                        }
                        on:change=on_visit_change
                        disabled=move || state.visits.get().is_empty()
                    >
                        <option value="">"-- เลือก visit --"</option>
                        {move || {
                            state.visits.get().into_iter().map(|visit| {
                                let label = format!(
                                    "{} {}{}",
                                    format_thai_date(visit.visit_date),
                                    visit.visit_time.map(format_time_hm).unwrap_or_default(),
                                    if visit.department_name.is_empty() {
                                        String::new()
                                    } else {
                                        format!(" · {}", visit.department_name)
                                    }
                                );
                                view! {
                                    <option value=visit.vn.clone()>{label}</option>
                                }
                            }).collect_view()
                        }}
                    </select>
                </div>

                <div class="visit-date">
                    <span class="visit-date__label">"วันที่สั่งจ่าย"</span>
                    <span class="visit-date__value mono">
                        {move || {
                            state
                                .selected_visit
                                .get()
                                .map(|visit| format_thai_date(visit.visit_date))
                                .unwrap_or_else(|| "ยังไม่เลือก visit".to_string())
                        }}
                    </span>
                </div>

                <button
                    class="button-secondary prescriber-chip"
                    type="button"
                    on:click=move |_| state.prescriber_edit_open.set(true)
                    disabled=move || state.selected_patient.get().is_none()
                >
                    <IconUser />
                    <span>
                        {move || {
                            let prescriber = state.prescriber.get();
                            if prescriber.name.trim().is_empty() {
                                "เลือกผู้ออกใบสั่งจ่าย".to_string()
                            } else {
                                prescriber.name
                            }
                        }}
                    </span>
                </button>
            </div>

            <div class="command-bar__actions">
                <span class="command-bar__status">
                    {move || {
                        if state.order_busy.get() {
                            "กำลังดึงรายการยา...".to_string()
                        } else if state.order_count.get() > 0 {
                            format!("รายการยา {} รายการ", state.lines.get().len())
                        } else if state.selected_visit.get().is_some() {
                            "พร้อมดึงรายการยา".to_string()
                        } else {
                            "เลือกผู้ป่วยและ visit เพื่อเริ่ม".to_string()
                        }
                    }}
                </span>
                <button
                    class="button-secondary"
                    type="button"
                    on:click=move |_| actions::do_fetch_orders(state.clone())
                    disabled=move || {
                        state.selected_visit.get().is_none()
                            || state.order_busy.get()
                            || state.catalog_is_empty()
                    }
                >
                    "ดึงรายการยา"
                </button>
                <button
                    class="button-accent"
                    type="button"
                    on:click=move |_| actions::open_confirm(state.clone())
                    disabled=move || !state.can_confirm()
                >
                    <IconCheck />
                    "ยืนยันข้อมูล"
                </button>
                <button
                    class="button-primary"
                    type="button"
                    on:click=move |_| actions::do_print(state.clone())
                    disabled=move || state.confirmed.get().is_none()
                >
                    <IconPrint />
                    "พิมพ์ใบสั่งจ่าย"
                </button>
            </div>
        </section>
    }
}
