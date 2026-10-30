//! The official A4 prescription sheet. The screen preview and the printed
//! output share this DOM and the print stylesheet.

use addict_order_models::{
    CategoryFlags, PatientDetail, be_year, format_qty_i64, thai_digit, thai_month_full,
};
use chrono::Datelike;
use leptos::prelude::*;

use crate::state::AppState;

#[derive(Debug, Clone, Default)]
struct SheetLine {
    generic: String,
    trade: String,
    strength: String,
    sig: String,
    qty_unit: String,
}

#[derive(Debug, Clone, Default)]
struct SheetData {
    day: String,
    month: String,
    year: String,
    patient_name: String,
    patient_cid: String,
    patient_house: String,
    patient_moo: String,
    patient_soi: String,
    patient_road: String,
    patient_subdistrict: String,
    patient_district: String,
    patient_province: String,
    patient_postal: String,
    patient_phone: String,
    prescriber_name: String,
    prescriber_license: String,
    hospital_name: String,
    hospital_house: String,
    hospital_moo: String,
    hospital_soi: String,
    hospital_road: String,
    hospital_subdistrict: String,
    hospital_district: String,
    hospital_province: String,
    hospital_postal: String,
    hospital_phone: String,
    hospital_fax: String,
    lines: Vec<SheetLine>,
    flags: CategoryFlags,
}

fn sheet_data(state: &AppState) -> SheetData {
    let patient = state.selected_patient.get().unwrap_or(PatientDetail {
        hn: String::new(),
        cid: None,
        full_name_th: String::new(),
        birth_date: None,
        sex: None,
        phone: String::new(),
        address: Default::default(),
        address_hint: String::new(),
    });
    let visit = state.selected_visit.get();
    let prescriber = state.prescriber.get();
    let hospital = state.hospital.get();
    let lines = state.lines.get();
    let flags = CategoryFlags::from_lines(&lines);
    let (day, month, year) = visit
        .as_ref()
        .map(|visit| {
            (
                visit.visit_date.day().to_string(),
                thai_month_full(visit.visit_date).to_string(),
                be_year(visit.visit_date.year()).to_string(),
            )
        })
        .unwrap_or_default();

    SheetData {
        day,
        month,
        year,
        patient_name: patient.full_name_th,
        patient_cid: patient.cid.unwrap_or_default(),
        patient_house: patient.address.house_no,
        patient_moo: patient.address.moo,
        patient_soi: patient.address.soi,
        patient_road: patient.address.road,
        patient_subdistrict: patient.address.subdistrict,
        patient_district: patient.address.district,
        patient_province: patient.address.province,
        patient_postal: patient.address.postal_code,
        patient_phone: patient.phone,
        prescriber_name: prescriber.name,
        prescriber_license: prescriber.license_no,
        hospital_name: hospital.name,
        hospital_house: hospital.house_no,
        hospital_moo: hospital.moo,
        hospital_soi: hospital.soi,
        hospital_road: hospital.road,
        hospital_subdistrict: hospital.subdistrict,
        hospital_district: hospital.district,
        hospital_province: hospital.province,
        hospital_postal: hospital.postal_code,
        hospital_phone: hospital.phone,
        hospital_fax: hospital.fax,
        lines: lines
            .iter()
            .map(|line| SheetLine {
                generic: line.generic_name.clone(),
                trade: line.trade_name.clone(),
                strength: line.strength_text.clone(),
                sig: line.sig.clone(),
                qty_unit: format!("{} {}", format_qty_i64(line.quantity), line.print_unit),
            })
            .collect(),
        flags,
    }
}

#[component]
fn FormLine(label: String, value: String) -> impl IntoView {
    view! {
        <div class="f-line">
            <span class="f-label">{label}</span>
            <span class="f-value">{value}</span>
        </div>
    }
}

#[component]
pub fn DocumentSheet(state: AppState) -> impl IntoView {
    view! {
        {move || {
            let data = sheet_data(&state);
            let rows = (0..5)
                .map(|index| {
                    let line = data.lines.get(index).cloned().unwrap_or_default();
                    view! {
                        <tr>
                            <td class="f-num">{thai_digit(index + 1)}</td>
                            <td>{line.generic}</td>
                            <td>{line.trade}</td>
                            <td>{line.strength}</td>
                            <td>{line.sig}</td>
                            <td>{line.qty_unit}</td>
                        </tr>
                    }
                })
                .collect_view();
            let narcotic_class = if data.flags.narcotic_2 {
                "f-checkbox f-checkbox--checked"
            } else {
                "f-checkbox"
            };
            let psychotropic_class = if data.flags.psychotropic_2 {
                "f-checkbox f-checkbox--checked"
            } else {
                "f-checkbox"
            };
            let narcotic_mark = if data.flags.narcotic_2 { "✓" } else { "" };
            let psychotropic_mark = if data.flags.psychotropic_2 { "✓" } else { "" };

            view! {
                <article class="sheet">
                    <p class="f-code">"แบบ ใบสั่งจ่าย ย.ส. ๒/ว.จ. ๒"</p>
                    <h1 class="f-title">"ใบสั่งจ่ายยาเสพติดให้โทษในประเภท ๒ หรือวัตถุออกฤทธิ์ในประเภท ๒"</h1>
                    <div class="f-title-rule"></div>

                    <div class="f-date">
                        "วันที่ " <span class="f-value f-value--short">{data.day}</span>
                        " เดือน " <span class="f-value f-value--medium">{data.month}</span>
                        " พ.ศ. " <span class="f-value f-value--short">{data.year}</span>
                    </div>

                    <section class="f-prescriber">
                        <FormLine label="ข้าพเจ้า".to_string() value=data.prescriber_name />
                        <FormLine
                            label="ใบอนุญาตประกอบวิชาชีพเวชกรรม/ทันตกรรม/การสัตวแพทย์ชั้นหนึ่ง เลขที่".to_string()
                            value=data.prescriber_license
                        />
                        <FormLine label="ปฏิบัติงาน ณ สถานพยาบาลชื่อ".to_string() value=data.hospital_name />
                        <div class="f-line">
                            <span class="f-label">"ตั้งอยู่เลขที่"</span>
                            <span class="f-value f-value--short">{data.hospital_house}</span>
                            <span class="f-label">"หมู่ที่"</span>
                            <span class="f-value f-value--short">{data.hospital_moo}</span>
                            <span class="f-label">"ตรอก/ซอย"</span>
                            <span class="f-value f-value--medium">{data.hospital_soi}</span>
                        </div>
                        <div class="f-line">
                            <span class="f-label">"ถนน"</span>
                            <span class="f-value f-value--medium">{data.hospital_road}</span>
                            <span class="f-label">"ตำบล/แขวง"</span>
                            <span class="f-value">{data.hospital_subdistrict}</span>
                        </div>
                        <div class="f-line">
                            <span class="f-label">"อำเภอ/เขต"</span>
                            <span class="f-value f-value--medium">{data.hospital_district}</span>
                            <span class="f-label">"จังหวัด"</span>
                            <span class="f-value f-value--medium">{data.hospital_province}</span>
                            <span class="f-label">"รหัสไปรษณีย์"</span>
                            <span class="f-value f-value--short">{data.hospital_postal}</span>
                        </div>
                        <div class="f-line">
                            <span class="f-label">"โทรศัพท์"</span>
                            <span class="f-value f-value--medium">{data.hospital_phone}</span>
                            <span class="f-label">"โทรสาร"</span>
                            <span class="f-value f-value--medium">{data.hospital_fax}</span>
                        </div>
                    </section>

                    <div class="f-category">
                        <span class="f-category__label">"ขอสั่งจ่าย"</span>
                        <span class=narcotic_class>{narcotic_mark}</span>
                        <span>"ยาเสพติดให้โทษในประเภท ๒"</span>
                        <span class=psychotropic_class>{psychotropic_mark}</span>
                        <span>"วัตถุออกฤทธิ์ในประเภท ๒"</span>
                        <span>"ดังรายการดังต่อไปนี้"</span>
                    </div>

                    <table class="f-table">
                        <colgroup>
                            <col style="width: 11.6mm" />
                            <col style="width: 29.3mm" />
                            <col style="width: 31.7mm" />
                            <col style="width: 24.9mm" />
                            <col style="width: 27.5mm" />
                            <col style="width: 44.8mm" />
                        </colgroup>
                        <thead>
                            <tr>
                                <th>"ลำดับ"</th>
                                <th>"ชื่อตัวยาสำคัญ"</th>
                                <th>"ชื่อการค้า"</th>
                                <th>"ความแรง"</th>
                                <th>"ขนาดการใช้"</th>
                                <th>
                                    "จำนวนที่สั่งจ่าย"
                                    <small>"(ระบุหน่วยเป็น Amp, Vial, Tab, Cap, Patch ฯลฯ)"</small>
                                </th>
                            </tr>
                        </thead>
                        <tbody>{rows}</tbody>
                    </table>

                    <section class="f-patient">
                        <FormLine
                            label="ให้แก่ (ชื่อผู้รับการรักษาหรือชื่อเจ้าของสัตว์ซึ่งรับการรักษา)".to_string()
                            value=data.patient_name
                        />
                        <FormLine
                            label="บัตรประจำตัวประชาชนหรือบัตรประจำตัวอื่นที่ทางราชการออกให้ เลขที่".to_string()
                            value=data.patient_cid
                        />
                        <div class="f-line">
                            <span class="f-label">"ที่อยู่ เลขที่"</span>
                            <span class="f-value f-value--short">{data.patient_house}</span>
                            <span class="f-label">"หมู่ที่"</span>
                            <span class="f-value f-value--short">{data.patient_moo}</span>
                            <span class="f-label">"ตรอก/ซอย"</span>
                            <span class="f-value f-value--medium">{data.patient_soi}</span>
                            <span class="f-label">"ถนน"</span>
                            <span class="f-value f-value--medium">{data.patient_road}</span>
                        </div>
                        <div class="f-line">
                            <span class="f-label">"ตำบล/แขวง"</span>
                            <span class="f-value">{data.patient_subdistrict}</span>
                            <span class="f-label">"อำเภอ/เขต"</span>
                            <span class="f-value f-value--medium">{data.patient_district}</span>
                            <span class="f-label">"จังหวัด"</span>
                            <span class="f-value f-value--medium">{data.patient_province}</span>
                        </div>
                        <div class="f-line">
                            <span class="f-label">"รหัสไปรษณีย์"</span>
                            <span class="f-value f-value--short">{data.patient_postal}</span>
                            <span class="f-label">"โทรศัพท์"</span>
                            <span class="f-value">{data.patient_phone}</span>
                        </div>
                    </section>

                    <div class="f-signature">
                        "(ลายมือชื่อ) .............................................................. ผู้ออกใบสั่งจ่าย"
                    </div>

                    <section class="f-notes">
                        <p class="f-notes__title">"หมายเหตุ :"</p>
                        <p>"๑. ให้ขีดฆ่าข้อความที่ไม่ต้องการออก"</p>
                        <p>
                            "๒. ให้ผู้รับการรักษาหรือเจ้าของสัตว์ซึ่งรับการรักษาส่งมอบต้นฉบับของเอกสารฉบับนี้ให้แก่สถานพยาบาลที่จ่ายยาเสพติดให้โทษหรือวัตถุออกฤทธิ์ ตามใบสั่งจ่ายให้ และให้สถานพยาบาลที่จ่ายยาเสพติดให้โทษหรือวัตถุออกฤทธิ์เก็บรักษาเอกสารฉบับนี้ไว้ เพื่อเป็นหลักฐานประกอบการจ่ายยาเสพติดให้โทษหรือวัตถุออกฤทธิ์"
                        </p>
                        <p>
                            "๓. ให้สถานพยาบาลที่สั่งจ่ายยาเสพติดให้โทษหรือวัตถุออกฤทธิ์เก็บรักษาสำเนาเอกสารฉบับนี้ไว้ เพื่อเป็นหลักฐานประกอบการจ่ายยาเสพติดให้โทษหรือวัตถุออกฤทธิ์"
                        </p>
                    </section>

                    <section class="f-legal">
                        <p>
                            "ผู้รับอนุญาตจำหน่าย จะจำหน่ายยาเสพติดให้โทษในประเภท ๒ หรือวัตถุออกฤทธิ์ในประเภท ๒ ให้แก่บุคคลภายนอกที่ไม่ใช่ผู้ป่วยหรือสัตว์ป่วย ซึ่งตนให้การรักษาได้ เฉพาะกรณีที่มีใบสั่งจ่ายยาเสพติดให้โทษในประเภท ๒ หรือวัตถุออกฤทธิ์ในประเภท ๒ จากผู้ประกอบวิชาชีพเวชกรรม ผู้ประกอบวิชาชีพทันตกรรม หรือผู้ประกอบวิชาชีพการสัตวแพทย์ชั้นหนึ่ง แล้วแต่กรณี"
                        </p>
                    </section>
                </article>
            }
        }}
    }
}
