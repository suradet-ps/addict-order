//! Settings modal: connection, narcotic catalog, hospital profile.

use addict_order_models::{Catalog, ConnectionSummary, DrugItem, LawCategory, NarcoticDrugConfig};
use leptos::prelude::*;

use crate::api;
use crate::components::icons::{IconCheck, IconSearch, IconSettings, IconX};
use crate::state::{AppState, Banner, SettingsTab};

#[component]
pub fn SettingsModal(state: AppState) -> impl IntoView {
    let connection_state = state.clone();
    let catalog_state = state.clone();
    let hospital_state = state.clone();
    let close = {
        let state = state.clone();
        move |_| state.settings_open.set(false)
    };
    let close_footer = {
        let state = state.clone();
        move |_| state.settings_open.set(false)
    };

    view! {
        {move || {
            state.settings_open.get().then(|| {
                let signed_in = state.session.get().is_some();
                let active_tab = if signed_in { state.settings_tab.get() } else { SettingsTab::Connection };
                let nav_connection = state.clone();
                let nav_catalog = state.clone();
                let nav_hospital = state.clone();
                view! {
                    <div class="modal-backdrop">
                        <section class="modal modal--settings" role="dialog" aria-modal="true" aria-labelledby="settings-title">
                            <header class="modal__head">
                                <div class="modal__title-wrap">
                                    <IconSettings />
                                    <h2 id="settings-title">"ตั้งค่า"</h2>
                                </div>
                                <button class="modal__close" aria-label="ปิด" on:click=close>
                                    <IconX />
                                </button>
                            </header>
                            <div class="settings-layout">
                                <nav class="settings-nav" aria-label="ตั้งค่า">
                                    <button
                                        class=if active_tab == SettingsTab::Connection { "settings-nav__item settings-nav__item--active" } else { "settings-nav__item" }
                                        type="button"
                                        on:click=move |_| nav_connection.settings_tab.set(SettingsTab::Connection)
                                    >
                                        "การเชื่อมต่อ"
                                    </button>
                                    {signed_in.then(|| view! {
                                        <button
                                            class=if active_tab == SettingsTab::Catalog { "settings-nav__item settings-nav__item--active" } else { "settings-nav__item" }
                                            type="button"
                                            on:click=move |_| nav_catalog.settings_tab.set(SettingsTab::Catalog)
                                        >
                                            "บัญชียา"
                                        </button>
                                        <button
                                            class=if active_tab == SettingsTab::Hospital { "settings-nav__item settings-nav__item--active" } else { "settings-nav__item" }
                                            type="button"
                                            on:click=move |_| nav_hospital.settings_tab.set(SettingsTab::Hospital)
                                        >
                                            "หัวใบสั่งจ่าย"
                                        </button>
                                    })}
                                </nav>
                                <div class="settings-content">
                                    {match active_tab {
                                        SettingsTab::Connection => view! { <ConnectionTab state=connection_state.clone() /> }.into_any(),
                                        SettingsTab::Catalog => view! { <CatalogTab state=catalog_state.clone() /> }.into_any(),
                                        SettingsTab::Hospital => view! { <HospitalTab state=hospital_state.clone() /> }.into_any(),
                                    }}
                                </div>
                            </div>
                            <footer class="modal__foot">
                                <span class="modal__foot-note">"การตั้งค่าไม่มีข้อมูลผู้ป่วย และไม่มีรหัสผ่านฐานข้อมูลในไฟล์ตั้งค่า"</span>
                                <button class="button-secondary" type="button" on:click=close_footer>"ปิด"</button>
                            </footer>
                        </section>
                    </div>
                }
            })
        }}
    }
}

#[component]
fn ConnectionTab(state: AppState) -> impl IntoView {
    let host = RwSignal::new(String::new());
    let port = RwSignal::new("3306".to_string());
    let database = RwSignal::new("hos".to_string());
    let user = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let busy = RwSignal::new(false);
    let result = RwSignal::new(None::<String>);

    Effect::new(move |_| {
        if let Some(summary) = state.connection_summary.get() {
            host.set(summary.host);
            port.set(summary.port.to_string());
            database.set(summary.database);
            user.set(summary.user);
        }
    });

    let test = {
        let state = state.clone();
        move |_| {
            let input = read_connection_input(host, port, database, user, password);
            busy.set(true);
            result.set(None);
            let state = state.clone();
            leptos::task::spawn_local(async move {
                match api::test_connection(&input).await {
                    Ok(ms) => result.set(Some(format!("เชื่อมต่อสำเร็จ ({} ms)", ms))),
                    Err(error) => {
                        state.banner.set(Some(Banner::error(error.message)));
                    }
                }
                busy.set(false);
            });
        }
    };

    let connect = {
        let state = state.clone();
        move |_| {
            let input = read_connection_input(host, port, database, user, password);
            busy.set(true);
            result.set(None);
            let state = state.clone();
            leptos::task::spawn_local(async move {
                match api::connect(&input).await {
                    Ok(ms) => {
                        state.configured.set(true);
                        state.connection_summary.set(Some(ConnectionSummary {
                            host: input.host.clone(),
                            port: input.port,
                            database: input.database.clone(),
                            user: input.user.clone(),
                        }));
                        state.health.set(crate::state::ConnectionHealth::Connected);
                        state.clear_session();
                        state.settings_open.set(false);
                        state.banner.set(Some(Banner::info(format!(
                            "บันทึกการเชื่อมต่อแล้ว ({} ms) กรุณาเข้าสู่ระบบใหม่",
                            ms
                        ))));
                    }
                    Err(error) => {
                        state.banner.set(Some(Banner::error(error.message)));
                    }
                }
                busy.set(false);
            });
        }
    };

    view! {
        <section class="settings-panel">
            <header class="settings-panel__head">
                <h3>"เชื่อมต่อ HOSxP"</h3>
                <p>"ใช้บัญชีอ่านอย่างเดียว (SELECT) และฐานข้อมูล hos"</p>
            </header>
            <div class="form-grid">
                <label class="field form-grid__wide">
                    <span class="field__label">"Host"</span>
                    <input class="text-input mono" prop:value=move || host.get() on:input=move |ev| host.set(event_target_value(&ev)) />
                </label>
                <label class="field">
                    <span class="field__label">"Port"</span>
                    <input class="text-input mono" type="number" prop:value=move || port.get() on:input=move |ev| port.set(event_target_value(&ev)) />
                </label>
                <label class="field">
                    <span class="field__label">"Database"</span>
                    <input class="text-input mono" prop:value=move || database.get() on:input=move |ev| database.set(event_target_value(&ev)) />
                </label>
                <label class="field">
                    <span class="field__label">"User (SELECT only)"</span>
                    <input class="text-input mono" prop:value=move || user.get() on:input=move |ev| user.set(event_target_value(&ev)) />
                </label>
                <label class="field">
                    <span class="field__label">"Password"</span>
                    <input class="text-input" type="password" prop:value=move || password.get() on:input=move |ev| password.set(event_target_value(&ev)) />
                </label>
            </div>
            {move || result.get().map(|message| view! { <p class="check-ok"><IconCheck /> {message}</p> })}
            <div class="settings-actions">
                <button class="button-secondary" type="button" on:click=test disabled=move || busy.get()>
                    {move || if busy.get() { "กำลังทดสอบ..." } else { "ทดสอบการเชื่อมต่อ" }}
                </button>
                <button class="button-primary" type="button" on:click=connect disabled=move || busy.get()>
                    "บันทึกการเชื่อมต่อ"
                </button>
            </div>
            <p class="settings-hint">
                "รหัสผ่านถูกเข้ารหัสด้วย encryptman และ master key อยู่ใน OS keychain ไม่ถูกเก็บในไฟล์ settings"
            </p>
        </section>
    }
}

#[component]
fn CatalogTab(state: AppState) -> impl IntoView {
    let catalog = RwSignal::new(Catalog::default());
    let loaded = RwSignal::new(false);
    Effect::new(move |_| {
        if !loaded.get()
            && let Some(settings) = state.settings.get()
        {
            catalog.set(settings.catalog);
            loaded.set(true);
        }
    });

    let search_term = RwSignal::new(String::new());
    let search_results = RwSignal::new(Vec::<DrugItem>::new());
    let search_busy = RwSignal::new(false);

    let do_search = {
        let state = state.clone();
        move |_| {
            let term = search_term.get().trim().to_string();
            if term.chars().count() < 2 {
                state
                    .banner
                    .set(Some(Banner::warn("พิมพ์อย่างน้อย 2 ตัวอักษรเพื่อค้นหายา")));
                return;
            }
            search_busy.set(true);
            search_results.set(Vec::new());
            let state = state.clone();
            leptos::task::spawn_local(async move {
                match api::search_drugs(&term).await {
                    Ok(rows) => search_results.set(rows),
                    Err(error) => state.banner.set(Some(Banner::error(error.message))),
                }
                search_busy.set(false);
            });
        }
    };

    let save = {
        let state = state.clone();
        move |_| {
            let value = catalog.get();
            let state = state.clone();
            leptos::task::spawn_local(async move {
                match api::save_catalog(&value).await {
                    Ok(()) => {
                        if let Some(mut settings) = state.settings.get() {
                            settings.catalog = value.clone();
                            state.settings.set(Some(settings));
                        }
                        state.banner.set(Some(Banner::info("บันทึกบัญชียาแล้ว")));
                    }
                    Err(error) => state.banner.set(Some(Banner::error(error.message))),
                }
            });
        }
    };

    let search_state = state.clone();
    let list_state = state.clone();
    view! {
        <section class="settings-panel">
            <header class="settings-panel__head">
                <h3>"บัญชียาเสพติด"</h3>
                <p>"เพิ่ม icode ที่เภสัชกรอนุมัติ แล้วกำหนดประเภททางกฎหมาย หน่วยพิมพ์ และขนาดการใช้เริ่มต้น"</p>
            </header>

            <div class="picker">
                <div class="search">
                    <span class="search__icon"><IconSearch /></span>
                    <input
                        class="search__input"
                        type="text"
                        placeholder="ค้นหา icode หรือชื่อยาใน drugitems"
                        prop:value=move || search_term.get()
                        on:input=move |ev| search_term.set(event_target_value(&ev))
                    />
                    <button class="button-blue search__submit" type="button" on:click=do_search disabled=move || search_busy.get()>
                        {move || if search_busy.get() { "กำลังค้นหา..." } else { "ค้นหายา" }}
                    </button>
                </div>
                {move || {
                    let rows = search_results.get();
                    (!rows.is_empty()).then(|| {
                        view! {
                            <div class="picker__results">
                                {rows.into_iter().map(|item| {
                                    let add_item = item.clone();
                                    let state_for_add = search_state.clone();
                                    let catalog_for_add = catalog;
                                    view! {
                                        <div class="picker__result">
                                            <div class="picker__result-info">
                                                <span class="picker__result-name">{item.name.clone()}</span>
                                                <span class="picker__result-meta">
                                                    <span class="badge-icode">{item.icode.clone()}</span>
                                                    <span class="mono">{item.strength_label()}</span>
                                                    <span class="muted">{item.trade_name.clone()}</span>
                                                </span>
                                            </div>
                                            <button class="button-secondary picker__add" type="button" on:click=move |_| add_drug_to_catalog(&state_for_add, catalog_for_add, add_item.clone())>
                                                "+ เพิ่ม"
                                            </button>
                                        </div>
                                    }
                                }).collect_view()}
                            </div>
                        }
                    })
                }}
            </div>

            <div class="catalog-list">
                {move || {
                    let mut drugs = catalog.get().drugs;
                    drugs.sort_by_key(|drug| (drug.sort_order, drug.generic_name.to_lowercase()));
                    if drugs.is_empty() {
                        return view! { <p class="muted">"ยังไม่มีรายการยาในบัญชี"</p> }.into_any();
                    }
                    view! {
                        <div class="catalog-list__head">
                            <span>"รายการยา"</span>
                            <span class="mono">{drugs.len()}</span>
                        </div>
                        {drugs.into_iter().enumerate().map(|(index, drug)| {
                            let icode = drug.icode.clone();
                            let current = catalog.get();
                            let item = current.drugs.iter().find(|item| item.icode == icode).cloned().unwrap_or(drug);
                            view! {
                                <CatalogRow
                                    state=list_state.clone()
                                    catalog=catalog
                                    index=index
                                    item=item
                                />
                            }
                        }).collect_view()}
                    }.into_any()
                }}
            </div>

            <div class="settings-actions">
                <button class="button-primary" type="button" on:click=save>
                    "บันทึกบัญชียา"
                </button>
            </div>
            <p class="settings-hint">
                "ระบบจะไม่ยอมบันทึกหากรายการใดขาดชื่อยา ความแรง หน่วยพิมพ์ หรือประเภททางกฎหมาย"
            </p>
        </section>
    }
}

#[component]
fn CatalogRow(
    state: AppState,
    catalog: RwSignal<Catalog>,
    index: usize,
    item: NarcoticDrugConfig,
) -> impl IntoView {
    let icode = item.icode.clone();
    let law_value = item
        .law_category
        .map(|category| match category {
            LawCategory::Narcotic2 => "NARCOTIC_2",
            LawCategory::Psychotropic2 => "PSYCHOTROPIC_2",
        })
        .unwrap_or("");

    let icode_generic = icode.clone();
    let icode_trade = icode.clone();
    let icode_strength = icode.clone();
    let icode_unit = icode.clone();
    let icode_sig = icode.clone();

    let remove = {
        let state = state.clone();
        let icode = icode.clone();
        move |_| {
            let mut next = catalog.get();
            next.remove(&icode);
            catalog.set(next);
            state
                .banner
                .set(Some(Banner::info("นำรายการยาออกจากบัญชีแล้ว กดบันทึกเพื่อยืนยัน")));
        }
    };

    let toggle_active = {
        let icode = icode.clone();
        move |ev| {
            let checked = event_target_checked(&ev);
            let mut next = catalog.get();
            if let Some(drug) = next.drugs.iter_mut().find(|drug| drug.icode == icode) {
                drug.active = checked;
                catalog.set(next);
            }
        }
    };

    let change_category = {
        let icode = icode.clone();
        move |ev| {
            let value = event_target_value(&ev);
            let category = match value.as_str() {
                "NARCOTIC_2" => Some(LawCategory::Narcotic2),
                "PSYCHOTROPIC_2" => Some(LawCategory::Psychotropic2),
                _ => None,
            };
            let mut next = catalog.get();
            if let Some(drug) = next.drugs.iter_mut().find(|drug| drug.icode == icode) {
                drug.law_category = category;
                catalog.set(next);
            }
        }
    };

    view! {
        <div class="catalog-row" data-index=index>
            <div class="catalog-row__identity">
                <span class="badge-icode">{icode}</span>
                <label class="catalog-row__active">
                    <input type="checkbox" checked=item.active on:change=toggle_active />
                    "ใช้งาน"
                </label>
                <button class="button-ghost catalog-row__remove" type="button" on:click=remove>
                    "ลบ"
                </button>
            </div>
            <div class="catalog-row__grid">
                <label class="field">
                    <span class="field__label">"ชื่อตัวยาสำคัญ"</span>
                    <input class="text-input" prop:value=item.generic_name.clone() on:change=move |ev| update_catalog_field(catalog, &icode_generic, "generic", event_target_value(&ev)) />
                </label>
                <label class="field">
                    <span class="field__label">"ชื่อการค้า"</span>
                    <input class="text-input" prop:value=item.trade_name.clone() on:change=move |ev| update_catalog_field(catalog, &icode_trade, "trade", event_target_value(&ev)) />
                </label>
                <label class="field">
                    <span class="field__label">"ความแรง"</span>
                    <input class="text-input mono" prop:value=item.strength_text.clone() on:change=move |ev| update_catalog_field(catalog, &icode_strength, "strength", event_target_value(&ev)) />
                </label>
                <label class="field">
                    <span class="field__label">"หน่วยพิมพ์"</span>
                    <input class="text-input mono" placeholder="Tab / Cap / Amp / Vial" prop:value=item.print_unit.clone() on:change=move |ev| update_catalog_field(catalog, &icode_unit, "unit", event_target_value(&ev)) />
                </label>
                <label class="field">
                    <span class="field__label">"ประเภททางกฎหมาย"</span>
                    <select class="select" prop:value=law_value on:change=change_category>
                        <option value="">"-- เลือก --"</option>
                        <option value="NARCOTIC_2">"ย.ส. ๒"</option>
                        <option value="PSYCHOTROPIC_2">"ว.จ. ๒"</option>
                    </select>
                </label>
                <label class="field catalog-row__sig">
                    <span class="field__label">"ขนาดการใช้เริ่มต้น"</span>
                    <input class="text-input" prop:value=item.default_sig.clone() on:change=move |ev| update_catalog_field(catalog, &icode_sig, "sig", event_target_value(&ev)) />
                </label>
            </div>
        </div>
    }
}

#[component]
fn HospitalTab(state: AppState) -> impl IntoView {
    let hospital = RwSignal::new(state.hospital.get_untracked());
    let source_address = RwSignal::new(String::new());
    let busy = RwSignal::new(false);

    Effect::new(move |_| {
        let current = state.hospital.get();
        if hospital.get_untracked() != current {
            hospital.set(current);
        }
    });

    let load_source = {
        let state = state.clone();
        move |_| {
            busy.set(true);
            let state = state.clone();
            leptos::task::spawn_local(async move {
                match api::fetch_hospital_source().await {
                    Ok(Some(source)) => {
                        let mut next = hospital.get();
                        next.name = source.name;
                        next.phone = source.phone;
                        next.fax = source.fax;
                        hospital.set(next);
                        source_address.set(source.address_text);
                        state.banner.set(Some(Banner::warn(
                            "ดึงข้อมูลจาก HOSxP แล้ว กรุณาตรวจสอบที่อยู่และแก้ไขก่อนพิมพ์",
                        )));
                    }
                    Ok(None) => state
                        .banner
                        .set(Some(Banner::warn("ไม่พบ hospital_profile ใน HOSxP"))),
                    Err(error) => state.banner.set(Some(Banner::error(error.message))),
                }
                busy.set(false);
            });
        }
    };

    let save = {
        let state = state.clone();
        move |_| {
            let value = hospital.get();
            let state = state.clone();
            leptos::task::spawn_local(async move {
                match api::save_hospital(&value).await {
                    Ok(()) => {
                        state.set_hospital(value.clone());
                        if let Some(mut settings) = state.settings.get() {
                            settings.hospital = value;
                            state.settings.set(Some(settings));
                        }
                        state.banner.set(Some(Banner::info("บันทึกหัวใบสั่งจ่ายแล้ว")));
                    }
                    Err(error) => state.banner.set(Some(Banner::error(error.message))),
                }
            });
        }
    };

    view! {
        <section class="settings-panel">
            <header class="settings-panel__head">
                <h3>"หัวใบสั่งจ่าย"</h3>
                <p>"ชื่อและที่อยู่สถานพยาบาลที่พิมพ์บนเอกสาร ต้องตรวจสอบกับใบอนุญาตสถานพยาบาล"</p>
            </header>
            <div class="settings-actions settings-actions--start">
                <button class="button-secondary" type="button" on:click=load_source disabled=move || busy.get()>
                    "ดึงข้อมูลจาก HOSxP"
                </button>
            </div>
            {move || {
                let text = source_address.get();
                (!text.trim().is_empty()).then(|| view! {
                    <p class="settings-hint settings-hint--warn">
                        "ที่อยู่ใน HOSxP: " {text}
                    </p>
                })
            }}
            <div class="form-grid">
                <label class="field form-grid__full">
                    <span class="field__label">"ชื่อสถานพยาบาล"</span>
                    <input class="text-input" prop:value=move || hospital.get().name on:input=move |ev| {
                        let mut next = hospital.get();
                        next.name = event_target_value(&ev);
                        hospital.set(next);
                    } />
                </label>
                <label class="field">
                    <span class="field__label">"เลขที่"</span>
                    <input class="text-input" prop:value=move || hospital.get().house_no on:input=move |ev| {
                        let mut next = hospital.get();
                        next.house_no = event_target_value(&ev);
                        hospital.set(next);
                    } />
                </label>
                <label class="field">
                    <span class="field__label">"หมู่ที่"</span>
                    <input class="text-input" prop:value=move || hospital.get().moo on:input=move |ev| {
                        let mut next = hospital.get();
                        next.moo = event_target_value(&ev);
                        hospital.set(next);
                    } />
                </label>
                <label class="field">
                    <span class="field__label">"ตรอก/ซอย"</span>
                    <input class="text-input" prop:value=move || hospital.get().soi on:input=move |ev| {
                        let mut next = hospital.get();
                        next.soi = event_target_value(&ev);
                        hospital.set(next);
                    } />
                </label>
                <label class="field">
                    <span class="field__label">"ถนน"</span>
                    <input class="text-input" prop:value=move || hospital.get().road on:input=move |ev| {
                        let mut next = hospital.get();
                        next.road = event_target_value(&ev);
                        hospital.set(next);
                    } />
                </label>
                <label class="field">
                    <span class="field__label">"ตำบล/แขวง"</span>
                    <input class="text-input" prop:value=move || hospital.get().subdistrict on:input=move |ev| {
                        let mut next = hospital.get();
                        next.subdistrict = event_target_value(&ev);
                        hospital.set(next);
                    } />
                </label>
                <label class="field">
                    <span class="field__label">"อำเภอ/เขต"</span>
                    <input class="text-input" prop:value=move || hospital.get().district on:input=move |ev| {
                        let mut next = hospital.get();
                        next.district = event_target_value(&ev);
                        hospital.set(next);
                    } />
                </label>
                <label class="field">
                    <span class="field__label">"จังหวัด"</span>
                    <input class="text-input" prop:value=move || hospital.get().province on:input=move |ev| {
                        let mut next = hospital.get();
                        next.province = event_target_value(&ev);
                        hospital.set(next);
                    } />
                </label>
                <label class="field">
                    <span class="field__label">"รหัสไปรษณีย์"</span>
                    <input class="text-input mono" prop:value=move || hospital.get().postal_code on:input=move |ev| {
                        let mut next = hospital.get();
                        next.postal_code = event_target_value(&ev);
                        hospital.set(next);
                    } />
                </label>
                <label class="field">
                    <span class="field__label">"โทรศัพท์"</span>
                    <input class="text-input mono" prop:value=move || hospital.get().phone on:input=move |ev| {
                        let mut next = hospital.get();
                        next.phone = event_target_value(&ev);
                        hospital.set(next);
                    } />
                </label>
                <label class="field">
                    <span class="field__label">"โทรสาร"</span>
                    <input class="text-input mono" prop:value=move || hospital.get().fax on:input=move |ev| {
                        let mut next = hospital.get();
                        next.fax = event_target_value(&ev);
                        hospital.set(next);
                    } />
                </label>
            </div>
            <div class="settings-actions">
                <button class="button-primary" type="button" on:click=save>
                    "บันทึกหัวใบสั่งจ่าย"
                </button>
            </div>
        </section>
    }
}

fn update_catalog_field(catalog: RwSignal<Catalog>, icode: &str, field: &str, value: String) {
    let mut next = catalog.get();
    if let Some(drug) = next.drugs.iter_mut().find(|drug| drug.icode == icode) {
        match field {
            "generic" => drug.generic_name = value,
            "trade" => drug.trade_name = value,
            "strength" => drug.strength_text = value,
            "unit" => drug.print_unit = value,
            "sig" => drug.default_sig = value,
            _ => {}
        }
        catalog.set(next);
    }
}

fn add_drug_to_catalog(state: &AppState, catalog: RwSignal<Catalog>, item: DrugItem) {
    let mut next = catalog.get();
    let sort_order = next.drugs.len() as i32;
    next.upsert(NarcoticDrugConfig {
        icode: item.icode.clone(),
        generic_name: item.name.clone(),
        trade_name: item.trade_name.clone(),
        strength_text: item.strength_label(),
        print_unit: map_unit(&item.units),
        law_category: None,
        default_sig: String::new(),
        sort_order,
        active: true,
    });
    catalog.set(next);
    state.banner.set(Some(Banner::info(format!(
        "เพิ่ม {} แล้ว กรุณาระบุประเภททางกฎหมายและหน่วยพิมพ์",
        item.name
    ))));
}

fn read_connection_input(
    host: RwSignal<String>,
    port: RwSignal<String>,
    database: RwSignal<String>,
    user: RwSignal<String>,
    password: RwSignal<String>,
) -> api::ConnectionInput {
    api::ConnectionInput {
        host: host.get(),
        port: port.get().parse().unwrap_or(3306),
        database: database.get(),
        user: user.get(),
        password: password.get(),
    }
}

fn map_unit(raw: &str) -> String {
    let value = raw.trim();
    match value {
        "เม็ด" | "tablet" | "tab" => "Tab".to_string(),
        "แคปซูล" | "capsule" | "cap" => "Cap".to_string(),
        "ขวด" | "vial" => "Vial".to_string(),
        "หลอด" | "amp" | "ampoule" => "Amp".to_string(),
        "แผ่น" | "patch" => "Patch".to_string(),
        other => other.to_string(),
    }
}
