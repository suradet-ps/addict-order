//! Patient edit, prescriber selection, and print confirmation dialogs.

use addict_order_models::{
    CategoryFlags, PatientAddress, PatientDetail, Prescriber, PrescriberSource, ValidationSeverity,
    format_thai_date,
};
use leptos::prelude::*;

use crate::api;
use crate::components::icons::{IconSearch, IconX};
use crate::state::{AppState, Banner};

#[component]
pub fn Dialogs(state: AppState) -> impl IntoView {
    let patient_state = state.clone();
    let prescriber_state = state.clone();
    let confirm_state = state.clone();
    view! {
        {move || {
            patient_state.address_edit_open.get().then(|| {
                view! { <PatientEditForm state=patient_state.clone() /> }
            })
        }}
        {move || {
            prescriber_state.prescriber_edit_open.get().then(|| {
                view! { <PrescriberDialog state=prescriber_state.clone() /> }
            })
        }}
        {move || {
            confirm_state.confirm_open.get().then(|| {
                view! { <ConfirmDialog state=confirm_state.clone() /> }
            })
        }}
    }
}

#[component]
fn PatientEditForm(state: AppState) -> impl IntoView {
    let patient = state
        .selected_patient
        .get_untracked()
        .unwrap_or(PatientDetail {
            hn: String::new(),
            cid: None,
            full_name_th: String::new(),
            birth_date: None,
            sex: None,
            phone: String::new(),
            address: PatientAddress::default(),
            address_hint: String::new(),
        });
    let full_name = RwSignal::new(patient.full_name_th.clone());
    let cid = RwSignal::new(patient.cid.clone().unwrap_or_default());
    let phone = RwSignal::new(patient.phone.clone());
    let house = RwSignal::new(patient.address.house_no.clone());
    let moo = RwSignal::new(patient.address.moo.clone());
    let soi = RwSignal::new(patient.address.soi.clone());
    let road = RwSignal::new(patient.address.road.clone());
    let subdistrict = RwSignal::new(patient.address.subdistrict.clone());
    let district = RwSignal::new(patient.address.district.clone());
    let province = RwSignal::new(patient.address.province.clone());
    let postal = RwSignal::new(patient.address.postal_code.clone());
    let close = {
        let state = state.clone();
        move |_| state.address_edit_open.set(false)
    };
    let close_footer = {
        let state = state.clone();
        move |_| state.address_edit_open.set(false)
    };
    let save = {
        let state = state.clone();
        let base = patient.clone();
        move |_| {
            let cid_value = cid.get().trim().to_string();
            let updated = PatientDetail {
                full_name_th: full_name.get().trim().to_string(),
                cid: if cid_value.is_empty() {
                    None
                } else {
                    Some(cid_value)
                },
                phone: phone.get().trim().to_string(),
                address: PatientAddress {
                    house_no: house.get().trim().to_string(),
                    moo: moo.get().trim().to_string(),
                    soi: soi.get().trim().to_string(),
                    road: road.get().trim().to_string(),
                    subdistrict: subdistrict.get().trim().to_string(),
                    district: district.get().trim().to_string(),
                    province: province.get().trim().to_string(),
                    postal_code: postal.get().trim().to_string(),
                },
                ..base.clone()
            };
            state.set_patient(updated);
            state.address_edit_open.set(false);
            state
                .banner
                .set(Some(Banner::info("แก้ไขข้อมูลผู้ป่วยในเอกสารแล้ว")));
        }
    };

    view! {
        <div class="modal-backdrop">
            <section class="modal modal--wide" role="dialog" aria-modal="true" aria-labelledby="patient-edit-title">
                <header class="modal__head">
                    <h2 id="patient-edit-title">"แก้ไขข้อมูลผู้ป่วย"</h2>
                    <button class="modal__close" aria-label="ปิด" on:click=close>
                        <IconX />
                    </button>
                </header>
                <div class="modal__body">
                    <p class="modal__hint">
                        "ข้อมูลนี้ใช้สำหรับเอกสารเท่านั้น ไม่บันทึกลงดิสก์ การแก้ไขจะยกเลิกการยืนยันเดิม"
                    </p>
                    <div class="form-grid">
                        <label class="field form-grid__full">
                            <span class="field__label">"ชื่อ-สกุล"</span>
                            <input class="text-input" prop:value=move || full_name.get() on:input=move |ev| full_name.set(event_target_value(&ev)) />
                        </label>
                        <label class="field">
                            <span class="field__label">"CID / เลขบัตร"</span>
                            <input class="text-input mono" prop:value=move || cid.get() on:input=move |ev| cid.set(event_target_value(&ev)) />
                        </label>
                        <label class="field">
                            <span class="field__label">"โทรศัพท์"</span>
                            <input class="text-input mono" prop:value=move || phone.get() on:input=move |ev| phone.set(event_target_value(&ev)) />
                        </label>
                        <label class="field">
                            <span class="field__label">"เลขที่"</span>
                            <input class="text-input" prop:value=move || house.get() on:input=move |ev| house.set(event_target_value(&ev)) />
                        </label>
                        <label class="field">
                            <span class="field__label">"หมู่ที่"</span>
                            <input class="text-input" prop:value=move || moo.get() on:input=move |ev| moo.set(event_target_value(&ev)) />
                        </label>
                        <label class="field">
                            <span class="field__label">"ตรอก/ซอย"</span>
                            <input class="text-input" prop:value=move || soi.get() on:input=move |ev| soi.set(event_target_value(&ev)) />
                        </label>
                        <label class="field">
                            <span class="field__label">"ถนน"</span>
                            <input class="text-input" prop:value=move || road.get() on:input=move |ev| road.set(event_target_value(&ev)) />
                        </label>
                        <label class="field">
                            <span class="field__label">"ตำบล/แขวง"</span>
                            <input class="text-input" prop:value=move || subdistrict.get() on:input=move |ev| subdistrict.set(event_target_value(&ev)) />
                        </label>
                        <label class="field">
                            <span class="field__label">"อำเภอ/เขต"</span>
                            <input class="text-input" prop:value=move || district.get() on:input=move |ev| district.set(event_target_value(&ev)) />
                        </label>
                        <label class="field">
                            <span class="field__label">"จังหวัด"</span>
                            <input class="text-input" prop:value=move || province.get() on:input=move |ev| province.set(event_target_value(&ev)) />
                        </label>
                        <label class="field">
                            <span class="field__label">"รหัสไปรษณีย์"</span>
                            <input class="text-input mono" prop:value=move || postal.get() on:input=move |ev| postal.set(event_target_value(&ev)) />
                        </label>
                    </div>
                </div>
                <footer class="modal__foot">
                    <button class="button-secondary" type="button" on:click=close_footer>"ยกเลิก"</button>
                    <button class="button-accent" type="button" on:click=save>"บันทึกข้อมูลผู้ป่วย"</button>
                </footer>
            </section>
        </div>
    }
}

#[component]
fn PrescriberDialog(state: AppState) -> impl IntoView {
    let term = RwSignal::new(state.prescriber.get().name);
    let results = RwSignal::new(Vec::<Prescriber>::new());
    let busy = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);
    let manual_name = RwSignal::new(state.prescriber.get().name);
    let manual_license = RwSignal::new(state.prescriber.get().license_no);
    let close = {
        let state = state.clone();
        move |_| state.prescriber_edit_open.set(false)
    };
    let close_footer = {
        let state = state.clone();
        move |_| state.prescriber_edit_open.set(false)
    };

    let search = {
        move |ev: web_sys::SubmitEvent| {
            ev.prevent_default();
            let term_value = term.get().trim().to_string();
            if term_value.is_empty() {
                error.set(Some("กรุณากรอกชื่อหรือรหัสแพทย์".to_string()));
                return;
            }
            busy.set(true);
            error.set(None);
            leptos::task::spawn_local(async move {
                match api::search_doctors(&term_value).await {
                    Ok(rows) => {
                        if rows.is_empty() {
                            error.set(Some("ไม่พบแพทย์จากคำค้นหานี้".to_string()));
                        }
                        results.set(rows);
                    }
                    Err(err) => error.set(Some(err.message)),
                }
                busy.set(false);
            });
        }
    };

    let use_manual = {
        let state = state.clone();
        move |_| {
            let name = manual_name.get().trim().to_string();
            if name.is_empty() {
                error.set(Some("กรุณากรอกชื่อแพทย์".to_string()));
                return;
            }
            state.set_prescriber(Prescriber {
                doctor_code: None,
                name,
                license_no: manual_license.get().trim().to_string(),
                source: Some(PrescriberSource::Manual),
            });
            state.prescriber_edit_open.set(false);
        }
    };

    view! {
        <div class="modal-backdrop">
            <section class="modal modal--wide" role="dialog" aria-modal="true" aria-labelledby="prescriber-title">
                <header class="modal__head">
                    <h2 id="prescriber-title">"เลือกผู้ออกใบสั่งจ่าย"</h2>
                    <button class="modal__close" aria-label="ปิด" on:click=close>
                        <IconX />
                    </button>
                </header>
                <div class="modal__body">
                    <form class="search" on:submit=search>
                        <span class="search__icon"><IconSearch /></span>
                        <input
                            class="search__input"
                            type="text"
                            placeholder="ค้นหาชื่อหรือรหัสแพทย์"
                            prop:value=move || term.get()
                            on:input=move |ev| term.set(event_target_value(&ev))
                        />
                        <button class="button-blue search__submit" type="submit" disabled=move || busy.get()>
                            {move || if busy.get() { "กำลังค้นหา..." } else { "ค้นหา" }}
                        </button>
                    </form>
                    {move || error.get().map(|message| view! { <p class="form-error">{message}</p> })}
                    <div class="doctor-results">
                        {move || {
                            results.get().into_iter().map(|doctor| {
                                let state = state.clone();
                                let selected = doctor.clone();
                                view! {
                                    <button
                                        class="doctor-result"
                                        type="button"
                                        on:click=move |_| {
                                            state.set_prescriber(Prescriber {
                                                ..selected.clone()
                                            });
                                            state.prescriber_edit_open.set(false);
                                        }
                                    >
                                        <span class="doctor-result__name">{doctor.name.clone()}</span>
                                        <span class="doctor-result__meta mono">
                                            {doctor.doctor_code.clone().unwrap_or_default()}
                                            " · "
                                            {doctor.license_no.clone()}
                                        </span>
                                    </button>
                                }
                            }).collect_view()
                        }}
                    </div>
                    <div class="manual-prescriber">
                        <h3>"หรือกรอกเอง"</h3>
                        <div class="form-grid">
                            <label class="field">
                                <span class="field__label">"ชื่อแพทย์"</span>
                                <input class="text-input" prop:value=move || manual_name.get() on:input=move |ev| manual_name.set(event_target_value(&ev)) />
                            </label>
                            <label class="field">
                                <span class="field__label">"เลขใบประกอบวิชาชีพ"</span>
                                <input class="text-input mono" prop:value=move || manual_license.get() on:input=move |ev| manual_license.set(event_target_value(&ev)) />
                            </label>
                        </div>
                        <button class="button-secondary" type="button" on:click=use_manual>
                            "ใช้ข้อมูลที่กรอก"
                        </button>
                    </div>
                </div>
                <footer class="modal__foot">
                    <button class="button-secondary" type="button" on:click=close_footer>"ปิด"</button>
                </footer>
            </section>
        </div>
    }
}

#[component]
fn ConfirmDialog(state: AppState) -> impl IntoView {
    let close = {
        let state = state.clone();
        move |_| state.confirm_open.set(false)
    };
    let close_footer = {
        let state = state.clone();
        move |_| state.confirm_open.set(false)
    };
    let confirm = {
        let state = state.clone();
        move |_| {
            if state.confirm() {
                state.confirm_open.set(false);
                state
                    .banner
                    .set(Some(Banner::info("ยืนยันข้อมูลแล้ว พร้อมพิมพ์ใบสั่งจ่าย")));
            }
        }
    };

    view! {
        <div class="modal-backdrop">
            <section class="modal" role="dialog" aria-modal="true" aria-labelledby="confirm-title">
                <header class="modal__head">
                    <h2 id="confirm-title">"ยืนยันข้อมูลใบสั่งจ่าย"</h2>
                    <button class="modal__close" aria-label="ปิด" on:click=close>
                        <IconX />
                    </button>
                </header>
                <div class="modal__body">
                    {move || {
                        let draft_state = state.clone();
                        draft_state.draft().map(|draft| {
                            let flags = CategoryFlags::from_lines(&draft.lines);
                            view! {
                                <dl class="confirm-summary">
                                    <div><dt>"ผู้ป่วย"</dt><dd>{draft.patient.full_name_th.clone()}</dd></div>
                                    <div><dt>"HN"</dt><dd class="mono">{draft.patient.hn.clone()}</dd></div>
                                    <div><dt>"วันที่สั่งจ่าย"</dt><dd>{format_thai_date(draft.prescription_date)}</dd></div>
                                    <div><dt>"ผู้ออกใบสั่งจ่าย"</dt><dd>{draft.prescriber.name.clone()}</dd></div>
                                    <div><dt>"เลขใบประกอบวิชาชีพ"</dt><dd class="mono">{draft.prescriber.license_no.clone()}</dd></div>
                                    <div><dt>"สถานพยาบาล"</dt><dd>{draft.hospital.name.clone()}</dd></div>
                                    <div><dt>"รายการยา"</dt><dd class="mono">{draft.lines.len()} / 5</dd></div>
                                    <div>
                                        <dt>"ประเภท"</dt>
                                        <dd>
                                            {if flags.narcotic_2 { "ย.ส. ๒ " } else { "" }}
                                            {if flags.psychotropic_2 { "ว.จ. ๒" } else { "" }}
                                        </dd>
                                    </div>
                                </dl>
                            }
                        })
                    }}
                    <div class="confirm-warnings">
                        {move || {
                            let check_state = state.clone();
                            let report = check_state.validation();
                            report.issues.into_iter().map(|issue| {
                                let class = match issue.severity {
                                    ValidationSeverity::Error => "check-item check-item--error",
                                    ValidationSeverity::Warning => "check-item check-item--warn",
                                };
                                view! { <p class=class>{issue.message}</p> }
                            }).collect_view()
                        }}
                    </div>
                    <p class="modal__hint">
                        "การยืนยันจะล็อกข้อมูลชั่วคราวสำหรับพิมพ์ หากแก้ไขข้อมูลใด ๆ หลังยืนยัน ต้องยืนยันใหม่"
                    </p>
                </div>
                <footer class="modal__foot">
                    <button class="button-secondary" type="button" on:click=close_footer>"ยกเลิก"</button>
                    <button
                        class="button-accent"
                        type="button"
                        on:click=confirm
                        disabled=move || {
                            let check_state = state.clone();
                            !check_state.can_confirm()
                        }
                    >
                        "ยืนยันและล็อกข้อมูล"
                    </button>
                </footer>
            </section>
        </div>
    }
}
