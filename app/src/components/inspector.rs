//! Right-side review inspector: patient, visit, prescriber, validation.

use addict_order_models::{
    ValidationSeverity, format_thai_date, format_thai_date_slash, format_time_hm,
};
use leptos::prelude::*;

use crate::components::icons::{IconCheck, IconUser, IconWarning};
use crate::state::AppState;

#[component]
pub fn Inspector(state: AppState) -> impl IntoView {
    let patient_state = state.clone();
    let visit_state = state.clone();
    let prescriber_state = state.clone();
    let check_state = state.clone();
    let confirm_state = state.clone();

    view! {
        <aside class="inspector">
            <section class="inspector-card">
                <header class="inspector-card__head">
                    <h2>"ผู้ป่วย"</h2>
                    <button
                        class="button-ghost inspector-card__action"
                        type="button"
                        on:click=move |_| patient_state.address_edit_open.set(true)
                        disabled=move || patient_state.selected_patient.get().is_none()
                    >
                        "แก้ไข"
                    </button>
                </header>
                {move || {
                    patient_state.selected_patient.get().map(|patient| {
                        let address = format!(
                            "{} หมู่ {} {} {} {} {} {}",
                            patient.address.house_no,
                            patient.address.moo,
                            patient.address.soi,
                            patient.address.road,
                            patient.address.subdistrict,
                            patient.address.district,
                            patient.address.province,
                        );
                        view! {
                            <div class="inspector-patient">
                                <p class="inspector-patient__name">{patient.full_name_th.clone()}</p>
                                <p class="inspector-patient__meta">
                                    <span class="mono">{patient.hn.clone()}</span>
                                    <span class="mono">{patient.cid.clone().unwrap_or_else(|| "ไม่มี CID".to_string())}</span>
                                </p>
                                <p class="inspector-patient__address">{address}</p>
                                <p class="inspector-patient__phone">{patient.phone.clone()}</p>
                            </div>
                        }
                    })
                }}
            </section>

            <section class="inspector-card">
                <header class="inspector-card__head">
                    <h2>"Visit"</h2>
                </header>
                {move || {
                    if let Some(visit) = visit_state.selected_visit.get() {
                        view! {
                            <div class="inspector-visit">
                                <p class="inspector-visit__date">
                                    {format_thai_date(visit.visit_date)}
                                    {visit.visit_time.map(|time| format!(" · {}", format_time_hm(time))).unwrap_or_default()}
                                </p>
                                <p>{visit.department_name.clone()}</p>
                                <p class="muted">{visit.doctor_name.clone()}</p>
                            </div>
                        }.into_any()
                    } else {
                        view! { <p class="muted">"ยังไม่เลือก visit"</p> }.into_any()
                    }
                }}
            </section>

            <section class="inspector-card">
                <header class="inspector-card__head">
                    <h2>"ผู้ออกใบสั่งจ่าย"</h2>
                    <button
                        class="button-ghost inspector-card__action"
                        type="button"
                        on:click=move |_| prescriber_state.prescriber_edit_open.set(true)
                    >
                        "เลือกแพทย์"
                    </button>
                </header>
                {move || {
                    let prescriber = prescriber_state.prescriber.get();
                    let input_state = prescriber_state.clone();
                    view! {
                        <p class="inspector-prescriber__name">
                            {if prescriber.name.trim().is_empty() { "ยังไม่เลือกแพทย์".to_string() } else { prescriber.name.clone() }}
                        </p>
                        <label class="field field--compact">
                            <span class="field__label">"เลขใบประกอบวิชาชีพ"</span>
                            <input
                                class="text-input"
                                type="text"
                                prop:value=move || input_state.prescriber.get().license_no
                                on:input=move |ev| {
                                    let mut next = input_state.prescriber.get();
                                    next.license_no = event_target_value(&ev);
                                    input_state.set_prescriber(next);
                                }
                            />
                        </label>
                    }
                }}
            </section>

            <section class="inspector-card">
                <header class="inspector-card__head">
                    <h2>"ตรวจสอบ"</h2>
                    <IconCheck />
                </header>
                {move || {
                    let report = check_state.validation();
                    if report.issues.is_empty() {
                        view! {
                            <p class="check-ok"><IconCheck /> "ข้อมูลครบถ้วน"</p>
                        }.into_any()
                    } else {
                        view! {
                            <ul class="check-list">
                                {report.issues.into_iter().map(|issue| {
                                    let class = match issue.severity {
                                        ValidationSeverity::Error => "check-item check-item--error",
                                        ValidationSeverity::Warning => "check-item check-item--warn",
                                    };
                                    let icon = match issue.severity {
                                        ValidationSeverity::Error => view! { <IconWarning /> }.into_any(),
                                        ValidationSeverity::Warning => view! { <IconWarning /> }.into_any(),
                                    };
                                    view! {
                                        <li class=class>
                                            {icon}
                                            <span>{issue.message}</span>
                                        </li>
                                    }
                                }).collect_view()}
                            </ul>
                        }.into_any()
                    }
                }}
            </section>

            <section class="inspector-card inspector-card--confirm">
                <header class="inspector-card__head">
                    <h2>"สถานะเอกสาร"</h2>
                </header>
                {move || {
                    if let Some(confirmed) = confirm_state.confirmed.get() {
                        view! {
                            <p class="check-ok">
                                <IconCheck />
                                "ยืนยันเมื่อ "
                                {format_thai_date_slash(confirmed.confirmed_at.date())}
                                " "
                                {format_time_hm(confirmed.confirmed_at.time())}
                            </p>
                        }.into_any()
                    } else {
                        view! { <p class="muted">"ยังไม่ยืนยันข้อมูล"</p> }.into_any()
                    }
                }}
                <p class="inspector-card__hint">
                    "ปุ่มพิมพ์จะเปิดหลังยืนยันข้อมูล และจะถูกปิดเมื่อมีการแก้ไข"
                </p>
            </section>

            <section class="inspector-card inspector-card--tiny">
                <IconUser />
                <span>"ข้อมูลผู้ป่วยใช้เพื่อสร้างเอกสารเท่านั้น ไม่บันทึกลงดิสก์"</span>
            </section>
        </aside>
    }
}
