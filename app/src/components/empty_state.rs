//! Empty states for the desk and the catalog.

use leptos::prelude::*;

use crate::components::icons::{BrandMark, IconSettings};
use crate::state::{AppState, SettingsTab};

#[component]
pub fn EmptyDesk(state: AppState) -> impl IntoView {
    let open_settings = {
        let state = state.clone();
        move |_| {
            state.settings_tab.set(SettingsTab::Catalog);
            state.settings_open.set(true);
        }
    };
    view! {
        <div class="empty-desk">
            <BrandMark />
            <h2>"เริ่มจากค้นหาผู้ป่วย"</h2>
            <p>"ค้นด้วย HN 7 หลัก, CID 13 หลัก หรือชื่อ-สกุล แล้วเลือก visit ที่ต้องการออกใบสั่งจ่าย"</p>
            <button class="button-secondary" type="button" on:click=open_settings>
                <IconSettings />
                "ตั้งค่าบัญชียา"
            </button>
        </div>
    }
}

#[component]
pub fn CatalogEmpty(state: AppState) -> impl IntoView {
    let open_settings = {
        let state = state.clone();
        move |_| {
            state.settings_tab.set(SettingsTab::Catalog);
            state.settings_open.set(true);
        }
    };
    view! {
        <div class="empty-desk empty-desk--warn">
            <BrandMark />
            <h2>"ยังไม่ได้ตั้งค่าบัญชียาเสพติด"</h2>
            <p>"เพิ่ม icode ที่ได้รับอนุมัติจากเภสัชกร พร้อมประเภททางกฎหมาย หน่วยพิมพ์ และขนาดการใช้เริ่มต้น"</p>
            <button class="button-accent" type="button" on:click=open_settings>
                <IconSettings />
                "เปิดตั้งค่าบัญชียา"
            </button>
        </div>
    }
}
