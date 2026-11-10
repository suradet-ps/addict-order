//! Command bar: patient search, visit selection, prescriber, document actions.

use addict_order_models::{format_thai_date, format_time_hm, mask_cid};
use leptos::prelude::*;

use crate::actions;
use crate::components::icons::{IconArrowRight, IconCheck, IconPrint, IconSearch, IconUser, IconX};
use crate::state::AppState;

#[component]
pub fn CommandBar(state: AppState) -> impl IntoView {
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
            <div class="command-bar__flow">
                <section class="flow-step flow-step--patient">
                    <header class="flow-step__head">
                        <span class=move || {
                            if state.selected_patient.get().is_some() {
                                "flow-step__num flow-step__num--done"
                            } else {
                                "flow-step__num"
                            }
                        }>
                            {move || {
                                if state.selected_patient.get().is_some() {
                                    view! { <IconCheck /> }.into_any()
                                } else {
                                    view! { "1" }.into_any()
                                }
                            }}
                        </span>
                        <h2 class="flow-step__title">"ผู้ป่วย"</h2>
                        {move || {
                            state
                                .selected_patient
                                .get()
                                .is_some()
                                .then(|| {
                                    let clear_state = state.clone();
                                    view! {
                                        <button
                                            class="flow-step__link"
                                            type="button"
                                            on:click=move |_| clear_state.clear_patient_work()
                                        >
                                            "เปลี่ยนผู้ป่วย"
                                        </button>
                                    }
                                })
                        }}
                    </header>

                    {move || {
                        if let Some(patient) = state.selected_patient.get() {
                            view! {
                                <>
                                    <div class="patient-pick">
                                        <span class="patient-pick__hn mono">
                                            {patient.hn.clone()}
                                        </span>
                                        <span class="patient-pick__name">
                                            {patient.full_name_th.clone()}
                                        </span>
                                    </div>
                                    <div class="flow-step__foot">
                                        <span class="flow-step__foot-label">"CID"</span>
                                        <span class="flow-step__foot-value mono">
                                            {patient
                                                .cid
                                                .as_deref()
                                                .map(mask_cid)
                                                .unwrap_or_else(|| "ไม่มี CID".to_string())}
                                        </span>
                                    </div>
                                </>
                            }
                                .into_any()
                        } else {
                            let submit_state = state.clone();
                            let on_search = move |ev: web_sys::SubmitEvent| {
                                ev.prevent_default();
                                actions::do_search(submit_state.clone());
                            };
                            view! {
                                <>
                                    <form class="search" on:submit=on_search>
                                        <span class="search__icon"><IconSearch /></span>
                                        <input
                                            class="search__input"
                                            type="text"
                                            placeholder="HN / CID / ชื่อ-สกุล"
                                            autocomplete="off"
                                            autofocus=true
                                            aria-label="ค้นหาผู้ป่วยด้วย HN, CID หรือชื่อ-สกุล"
                                            prop:value=move || state.search_term.get()
                                            on:input=move |ev| {
                                                state.search_term.set(event_target_value(&ev))
                                            }
                                        />
                                        <button
                                            class="button-blue search__submit"
                                            type="submit"
                                            disabled=move || state.search_busy.get()
                                        >
                                            {move || {
                                                if state.search_busy.get() {
                                                    "กำลังค้นหา..."
                                                } else {
                                                    "ค้นหา"
                                                }
                                            }}
                                        </button>
                                    </form>
                                    <p class="flow-step__hint">
                                        {move || {
                                            if state.patient_busy.get() {
                                                "กำลังโหลดข้อมูลผู้ป่วย..."
                                            } else {
                                                "HN 7 หลัก · CID 13 หลัก · ชื่อหรือนามสกุล"
                                            }
                                        }}
                                    </p>
                                    {move || {
                                        let results = state.search_results.get();
                                        let count = results.len();
                                        (!results.is_empty())
                                            .then(|| {
                                                let close_state = state.clone();
                                                view! {
                                                    <div
                                                        class="search-results"
                                                        role="listbox"
                                                        aria-label="ผลการค้นหาผู้ป่วย"
                                                    >
                                                        <div class="search-results__head">
                                                            <span>
                                                                "ผลการค้นหา "
                                                                <span class="search-results__count mono">
                                                                    {count}
                                                                </span>
                                                                " รายการ"
                                                            </span>
                                                            <button
                                                                class="search-results__close"
                                                                type="button"
                                                                aria-label="ปิดผลการค้นหา"
                                                                on:click=move |_| {
                                                                    close_state.search_results.set(Vec::new())
                                                                }
                                                            >
                                                                <IconX />
                                                            </button>
                                                        </div>
                                                        {results
                                                            .into_iter()
                                                            .map(|patient| {
                                                                let summary = patient.clone();
                                                                let select_state = state.clone();
                                                                let cid = patient
                                                                    .cid
                                                                    .as_deref()
                                                                    .map(mask_cid)
                                                                    .unwrap_or_else(|| "ไม่มี CID".to_string());
                                                                view! {
                                                                    <button
                                                                        class="search-result"
                                                                        type="button"
                                                                        role="option"
                                                                        on:click=move |_| {
                                                                            actions::select_patient(
                                                                                select_state.clone(),
                                                                                summary.clone(),
                                                                            )
                                                                        }
                                                                    >
                                                                        <span class="search-result__hn mono">
                                                                            {patient.hn.clone()}
                                                                        </span>
                                                                        <span class="search-result__name">
                                                                            {patient.full_name_th.clone()}
                                                                        </span>
                                                                        <span class="search-result__cid mono">
                                                                            {cid}
                                                                        </span>
                                                                        <span class="search-result__go">
                                                                            <IconArrowRight />
                                                                        </span>
                                                                    </button>
                                                                }
                                                            })
                                                            .collect_view()}
                                                    </div>
                                                }
                                            })
                                    }}
                                </>
                            }
                                .into_any()
                        }
                    }}
                </section>

                <section class="flow-step">
                    <header class="flow-step__head">
                        <span class=move || {
                            if state.selected_visit.get().is_some() {
                                "flow-step__num flow-step__num--done"
                            } else {
                                "flow-step__num"
                            }
                        }>
                            {move || {
                                if state.selected_visit.get().is_some() {
                                    view! { <IconCheck /> }.into_any()
                                } else {
                                    view! { "2" }.into_any()
                                }
                            }}
                        </span>
                        <h2 class="flow-step__title">"Visit"</h2>
                        <span class="flow-step__meta">
                            {move || {
                                if state.selected_patient.get().is_none() {
                                    String::new()
                                } else if state.visits.get().is_empty() {
                                    "ไม่พบ visit".to_string()
                                } else {
                                    format!("{} visit", state.visits.get().len())
                                }
                            }}
                        </span>
                    </header>

                    <select
                        class="select flow-step__select"
                        aria-label="เลือก visit ที่ต้องการออกใบสั่งจ่าย"
                        prop:value=move || {
                            state
                                .selected_visit
                                .get()
                                .map(|visit| visit.vn)
                                .unwrap_or_default()
                        }
                        on:change=on_visit_change
                        disabled=move || state.visits.get().is_empty()
                    >
                        <option value="">"-- เลือก visit --"</option>
                        {move || {
                            state
                                .visits
                                .get()
                                .into_iter()
                                .map(|visit| {
                                    let label = format!(
                                        "{} {}{}",
                                        format_thai_date(visit.visit_date),
                                        visit.visit_time.map(format_time_hm).unwrap_or_default(),
                                        if visit.department_name.is_empty() {
                                            String::new()
                                        } else {
                                            format!(" · {}", visit.department_name)
                                        },
                                    );
                                    view! { <option value=visit.vn.clone()>{label}</option> }
                                })
                                .collect_view()
                        }}
                    </select>

                    <div class="flow-step__foot">
                        <span class="flow-step__foot-label">"วันที่สั่งจ่าย"</span>
                        <span class=move || {
                            if state.selected_visit.get().is_some() {
                                "flow-step__foot-value"
                            } else if state.selected_patient.get().is_some()
                                && state.visits.get().is_empty()
                            {
                                "flow-step__foot-value flow-step__foot-value--warn"
                            } else {
                                "flow-step__foot-value flow-step__foot-value--muted"
                            }
                        }>
                            {move || {
                                if let Some(visit) = state.selected_visit.get() {
                                    format_thai_date(visit.visit_date)
                                } else if state.selected_patient.get().is_some()
                                    && state.visits.get().is_empty()
                                {
                                    "ไม่พบ visit ใน 30 วัน".to_string()
                                } else {
                                    "ยังไม่เลือก visit".to_string()
                                }
                            }}
                        </span>
                    </div>
                </section>

                <section class="flow-step">
                    <header class="flow-step__head">
                        <span class=move || {
                            if !state.prescriber.get().name.trim().is_empty() {
                                "flow-step__num flow-step__num--done"
                            } else {
                                "flow-step__num"
                            }
                        }>
                            {move || {
                                if !state.prescriber.get().name.trim().is_empty() {
                                    view! { <IconCheck /> }.into_any()
                                } else {
                                    view! { "3" }.into_any()
                                }
                            }}
                        </span>
                        <h2 class="flow-step__title">"ผู้ออกใบสั่งจ่าย"</h2>
                        <span class="flow-step__meta">
                            {move || {
                                if state.prescriber.get().name.trim().is_empty() {
                                    String::new()
                                } else if state.prescriber_user_set.get() {
                                    "เลือกเอง".to_string()
                                } else {
                                    "จาก HOSxP".to_string()
                                }
                            }}
                        </span>
                    </header>

                    <button
                        class="prescriber-pick"
                        type="button"
                        on:click=move |_| state.prescriber_edit_open.set(true)
                        disabled=move || state.selected_patient.get().is_none()
                    >
                        <span class="prescriber-pick__icon"><IconUser /></span>
                        <span class="prescriber-pick__body">
                            <span class="prescriber-pick__name">
                                {move || {
                                    let name = state.prescriber.get().name;
                                    if !name.trim().is_empty() {
                                        name
                                    } else if state.selected_patient.get().is_none() {
                                        "รอเลือกผู้ป่วย".to_string()
                                    } else {
                                        "เลือกผู้ออกใบสั่งจ่าย".to_string()
                                    }
                                }}
                            </span>
                        </span>
                        <span class="prescriber-pick__choose">"เลือก"</span>
                    </button>

                    <div class="flow-step__foot">
                        <span class="flow-step__foot-label">"เลขใบประกอบวิชาชีพ"</span>
                        <span class=move || {
                            let prescriber = state.prescriber.get();
                            if !prescriber.name.trim().is_empty()
                                && prescriber.license_no.trim().is_empty()
                            {
                                "flow-step__foot-value flow-step__foot-value--warn"
                            } else if prescriber.license_no.trim().is_empty() {
                                "flow-step__foot-value flow-step__foot-value--muted"
                            } else {
                                "flow-step__foot-value mono"
                            }
                        }>
                            {move || {
                                let prescriber = state.prescriber.get();
                                if !prescriber.name.trim().is_empty()
                                    && prescriber.license_no.trim().is_empty()
                                {
                                    "ยังไม่มีเลขใบประกอบวิชาชีพ".to_string()
                                } else if prescriber.license_no.trim().is_empty() {
                                    "—".to_string()
                                } else {
                                    prescriber.license_no
                                }
                            }}
                        </span>
                    </div>
                </section>
            </div>

            <section class="action-step">
                <div class="action-step__head">
                    <span class=move || {
                        if state.confirmed.get().is_some() {
                            "flow-step__num flow-step__num--done"
                        } else {
                            "flow-step__num"
                        }
                    }>
                        {move || {
                            if state.confirmed.get().is_some() {
                                view! { <IconCheck /> }.into_any()
                            } else {
                                view! { "4" }.into_any()
                            }
                        }}
                    </span>
                    <div class="action-step__titles">
                        <h2 class="flow-step__title">"ตรวจสอบและพิมพ์"</h2>
                        {move || {
                            let (tone, text) = if state.order_busy.get() {
                                ("busy", "กำลังดึงรายการยา...".to_string())
                            } else if state.confirmed.get().is_some() {
                                match state.printed_at.get() {
                                    Some(printed) => {
                                        (
                                            "ok",
                                            format!(
                                                "พิมพ์แล้วเมื่อ {}",
                                                format_time_hm(printed.time()),
                                            ),
                                        )
                                    }
                                    None => ("ok", "ยืนยันแล้ว · พร้อมพิมพ์".to_string()),
                                }
                            } else if state.form_overflow.get() {
                                (
                                    "error",
                                    "เนื้อหาเกิน 1 หน้า A4 · ย่อขนาดการใช้ก่อนยืนยัน"
                                        .to_string(),
                                )
                            } else if state.selected_visit.get().is_none() {
                                ("idle", "เลือกผู้ป่วยและ visit เพื่อเริ่ม".to_string())
                            } else if state.lines.get().is_empty() {
                                ("ready", "พร้อมดึงรายการยาสำหรับ visit นี้".to_string())
                            } else if state.lines.get().len() > 5 {
                                (
                                    "error",
                                    format!(
                                        "รายการยา {} / 5 · เกินจำนวนที่พิมพ์ได้",
                                        state.lines.get().len(),
                                    ),
                                )
                            } else {
                                (
                                    "ready",
                                    format!(
                                        "รายการยา {} / 5 · ตรวจสอบรายการแล้วกดยืนยัน",
                                        state.lines.get().len(),
                                    ),
                                )
                            };
                            view! {
                                <span class="action-step__status" data-tone=tone role="status">
                                    <span class="action-step__dot" aria-hidden="true"></span>
                                    {text}
                                </span>
                            }
                        }}
                    </div>
                </div>

                <div class="action-step__buttons">
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
                        disabled=move || !state.can_confirm() || state.form_overflow.get()
                    >
                        <IconCheck />
                        "ยืนยันข้อมูล"
                    </button>
                    <button
                        class="button-primary"
                        type="button"
                        on:click=move |_| actions::do_print(state.clone())
                        disabled=move || {
                            state.confirmed.get().is_none() || state.form_overflow.get()
                        }
                    >
                        <IconPrint />
                        "พิมพ์ใบสั่งจ่าย"
                    </button>
                </div>
            </section>
        </section>
    }
}
