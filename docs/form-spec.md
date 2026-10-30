# Official Form Specification - Y.S. 2 / W.C. 2

This document transcribes the official Thai prescription form supplied in
`addict-order/ใบสั่งจ่าย ยส2.pdf` and maps every field to the Addict Order data
model. It is the print contract. The app must not invent fields, remove fields,
or change the legal text.

Source files:

- `addict-order/ใบสั่งจ่าย ยส2.pdf`
- `addict-order/ใบสั่งจ่าย ยส2.docx`

## 1. Identity of the document

| Item | Value |
|---|---|
| Thai title | ใบสั่งจ่ายยาเสพติดให้โทษในประเภท ๒ หรือวัตถุออกฤทธิ์ในประเภท ๒ |
| Form code | แบบ ใบสั่งจ่าย ย.ส. ๒/ว.จ. ๒ |
| Issuing authority | กองควบคุมวัตถุเสพติด |
| Paper | A4 portrait, one page |
| Drug rows | 5 |
| Language | Thai |
| Signature | Handwritten by the prescriber |
| App role | Document builder and printer only |

Important terminology:

- The document is a **prescription** (ใบสั่งจ่าย), not a ward requisition
  (ใบเบิก).
- The app must use "ใบสั่งจ่าย" in the UI and on paper.
- The app does not prescribe. The prescriber signs.

## 2. Full transcription

The following text is reproduced from the official form. Preserve the exact
wording and punctuation in the print component.

### 2.1 Top line

```text
แบบ ใบสั่งจ่าย ย.ส. ๒/ว.จ. ๒
```

### 2.2 Title

```text
ใบสั่งจ่ายยาเสพติดให้โทษในประเภท ๒ หรือวัตถุออกฤทธิ์ในประเภท ๒
```

The title is centered, bold, and underlined.

### 2.3 Date line

```text
วันที่............... เดือน.................................... พ.ศ. ...............
```

The app fills the day, Thai month name, and Buddhist year.

### 2.4 Prescriber block

```text
ข้าพเจ้า................................................................................................................................................

ใบอนุญาตประกอบวิชาชีพเวชกรรม/ทันตกรรม/การสัตวแพทย์ชั้นหนึ่ง เลขที่................................................................

ปฏิบัติงาน ณ สถานพยาบาลชื่อ...............................................................................................................................

ตั้งอยู่เลขที่ ........................................ หมู่ที่ ............................................ ตรอก/ซอย ....................................................

ถนน .................................................................. ตำบล/แขวง .........................................................................................

อำเภอ/เขต ....................................................... จังหวัด ........................................... รหัสไปรษณีย์ ..............................

โทรศัพท์ ............................................................ โทรสาร ................................................................................................
```

### 2.5 Category line

```text
ขอสั่งจ่าย  [ ] ยาเสพติดให้โทษในประเภท ๒  [ ] วัตถุออกฤทธิ์ในประเภท ๒ ดังรายการดังต่อไปนี้
```

The two square checkboxes are part of the official form. The app checks them
from the legal category of the drug lines.

### 2.6 Drug table

| Column | Thai header | Meaning |
|---|---|---|
| 1 | ลำดับ | Sequence number, Thai digits ๑ to ๕ |
| 2 | ชื่อตัวยาสำคัญ | Generic name |
| 3 | ชื่อการค้า | Trade name |
| 4 | ความแรง | Strength |
| 5 | ขนาดการใช้ | Directions for use |
| 6 | จำนวนที่สั่งจ่าย (ระบุหน่วยเป็น Amp, Vial, Tab, Cap, Patch ฯลฯ) | Quantity and unit |

The table has five body rows. The preprinted row numbers are Thai digits.

### 2.7 Patient block

```text
ให้แก่ (ชื่อผู้รับการรักษาหรือชื่อเจ้าของสัตว์ซึ่งรับการรักษา) ........................................................................................................

บัตรประจำตัวประชาชนหรือบัตรประจำตัวอื่นที่ทางราชการออกให้ เลขที่ ..................................................................................

ที่อยู่ เลขที่ ................. หมู่ที่ .................. ตรอก/ซอย ............................................ ถนน .............................................................

ตำบล/แขวง ........................................... อำเภอ/เขต ............................................ จังหวัด .........................................................

รหัสไปรษณีย์ ......................................... โทรศัพท์ …....................................................................................................................
```

### 2.8 Signature line

```text
(ลายมือชื่อ) .............................................................. ผู้ออกใบสั่งจ่าย
```

The signature line is always blank. The app prints the prescriber name and
licence in the prescriber block, but never a signature.

### 2.9 Notes

```text
หมายเหตุ :

๑.  ให้ขีดฆ่าข้อความที่ไม่ต้องการออก

๒.  ให้ผู้รับการรักษาหรือเจ้าของสัตว์ซึ่งรับการรักษาส่งมอบต้นฉบับของเอกสารฉบับนี้ให้แก่สถานพยาบาลที่จ่ายยาเสพติดให้โทษหรือวัตถุออกฤทธิ์
    ตามใบสั่งจ่ายให้ และให้สถานพยาบาลที่จ่ายยาเสพติดให้โทษหรือวัตถุออกฤทธิ์เก็บรักษาเอกสารฉบับนี้ไว้ เพื่อเป็นหลักฐานประกอบการจ่าย
    ยาเสพติดให้โทษหรือวัตถุออกฤทธิ์

๓.  ให้สถานพยาบาลที่สั่งจ่ายยาเสพติดให้โทษหรือวัตถุออกฤทธิ์เก็บรักษาสำเนาเอกสารฉบับนี้ไว้ เพื่อเป็นหลักฐานประกอบการจ่ายยาเสพติดให้โทษ
    หรือวัตถุออกฤทธิ์
```

### 2.10 Legal box

```text
ผู้รับอนุญาตจำหน่าย จะจำหน่ายยาเสพติดให้โทษในประเภท ๒ หรือวัตถุออกฤทธิ์ในประเภท ๒ ให้แก่บุคคลภายนอกที่ไม่ใช่ผู้ป่วยหรือสัตว์ป่วย
ซึ่งตนให้การรักษาได้ เฉพาะกรณีที่มีใบสั่งจ่ายยาเสพติดให้โทษในประเภท ๒ หรือวัตถุออกฤทธิ์ในประเภท ๒ จากผู้ประกอบวิชาชีพเวชกรรม
ผู้ประกอบวิชาชีพทันตกรรม หรือผู้ประกอบวิชาชีพการสัตวแพทย์ชั้นหนึ่ง แล้วแต่กรณี
```

The legal box is part of the official form. It must not be removed or
paraphrased.

## 3. Measured layout

Measurements were taken from the supplied PDF, which is A4 portrait at
595.32 x 841.92 pt.

### 3.1 Fonts in the source PDF

| Role | Font | Size |
|---|---|---|
| Title | TH SarabunPSK Bold | 18 pt |
| Form code | TH SarabunPSK Bold | 14.04 pt, gray |
| Body and date | TH SarabunPSK | 15 pt |
| Table header and table body | TH SarabunPSK / Bold | 14.04 pt |
| Patient block | TH SarabunPSK | 14.04 pt |
| Notes and legal box | TH SarabunPSK | 12 pt |
| Checkbox glyph | Wingdings 2 | 15 pt |

### 3.2 Page positions

| Element | Position in PDF |
|---|---|
| Page | 595.32 x 841.92 pt |
| Top-left form code | x 82.34, y 36.31 |
| Title | x 140.06 to 483.87, y 73.62 to 93.42 |
| Title underline | x 188.90 to 432.70, y 99.07 |
| Date line | x 312.29 to 542.02, y 115.07 |
| Prescriber block | x 85.10 to 540.10, y 152.75 to 287.12 |
| Category line | x 117.86 to 509.14, y 296.30 |
| Table | x 71.18 to 552.69, y 322.13 to 472.39 |
| Patient block | x 85.10 to 541.33, y 478.22 to 566.98 |
| Signature line | x 276.53 to 541.18, y 601.60 |
| Notes | x 85.10 to 573.05, y 629.11 to 732.34 |
| Legal box | x 67.60 to 551.60, y 737.94 to 790.34 |

### 3.3 Table columns

| Column | Left edge (pt) | Width (pt) | Width (mm) |
|---|---|---|---|
| ลำดับ | 71.18 | 32.88 | 11.6 |
| ชื่อตัวยาสำคัญ | 104.06 | 83.04 | 29.3 |
| ชื่อการค้า | 187.10 | 89.91 | 31.7 |
| ความแรง | 277.01 | 70.58 | 24.9 |
| ขนาดการใช้ | 347.59 | 78.00 | 27.5 |
| จำนวนที่สั่งจ่าย | 425.59 | 127.10 | 44.8 |
| Total | 71.18 | 481.51 | 169.9 |

The table header is approximately 19.6 mm tall. Each body row is approximately
6.65 mm tall.

### 3.4 Margins

| Side | Measurement |
|---|---|
| Top | 12.7 mm |
| Left | 25.1 mm |
| Right | 15.0 mm |
| Bottom | 18.2 mm |

The exact print CSS will be tuned against the PDF and a real printer. The
static reference is [`print-template.html`](print-template.html).

## 4. Field mapping

| Form field | Primary source | Fallback | Required |
|---|---|---|---|
| Date: day | Selected visit date | None | Yes |
| Date: month | Selected visit date, Thai month | None | Yes |
| Date: Buddhist year | Selected visit date + 543 | None | Yes |
| ข้าพเจ้า | Prescriber name | Manual selection | Yes |
| เลขใบประกอบวิชาชีพ | `doctor.licenseno` | Session-only manual entry | Yes |
| สถานพยาบาลชื่อ | `hospital_profile` | Settings override | Yes |
| ที่อยู่สถานพยาบาล | `hospital_profile` | Settings override, pharmacist-verified | Yes |
| โทรศัพท์ / โทรสาร | `hospital_profile` | Settings override | No |
| ประเภท ย.ส. ๒ | Catalog `law_category` | None | Yes |
| ประเภท ว.จ. ๒ | Catalog `law_category` | None | Yes |
| ลำดับ | Row index | None | Yes |
| ชื่อตัวยาสำคัญ | Catalog generic name | `drugitems.name` | Yes |
| ชื่อการค้า | Catalog trade name | `drugitems.trade_name` | No |
| ความแรง | Catalog strength | `drugitems.strength` | Yes |
| ขนาดการใช้ | Order sig text | Catalog default sig | Yes |
| จำนวนที่สั่งจ่าย | `opitemrece.qty` int(11) | None in v1; no decimals | Yes |
| หน่วย | Catalog print unit | Verified unit mapping | Yes |
| ชื่อผู้รับการรักษา | `patient` name fields | None | Yes |
| เลขบัตร | `patient.cid` | Other official ID | Yes |
| ที่อยู่ผู้ป่วย | `thaiaddress` subdistrict join | Manual edit | Warning |
| โทรศัพท์ผู้ป่วย | `hometel`, then `informtel`, then `worktel`, then `mobile_phone_number` | Manual edit | No |
| ลายมือชื่อ | None | None | Handwritten |

### 4.1 Patient name format

HOSxP commonly stores a prefix in `pname` and the given and family names in
`fname` and `lname`. The print name is assembled as:

```text
{pname}{fname} {lname}
```

Example: `นายสมชาย ใจดี`.

Verify the exact spacing with a real patient before hardcoding it. Some
instances store the prefix with a trailing space, some do not.

### 4.2 CID format

The form accepts a national ID card or another official ID. The app should:

- Use `patient.cid` when it is a valid 13-digit value.
- Show the CID in the form preview.
- Mask the CID in the search result list.
- Allow an alternative official ID type and number when CID is absent.
- Never generate or guess a CID.

### 4.3 Address format

The patient address block has separate blanks:

```text
เลขที่ {house_no} หมู่ที่ {moo} ตรอก/ซอย {soi} ถนน {road}
ตำบล/แขวง {subdistrict} อำเภอ/เขต {district} จังหวัด {province}
รหัสไปรษณีย์ {postal_code} โทรศัพท์ {phone}
```

The hospital address block uses the same components from the hospital profile.

**Verified on Sra Bos:** The patient address join is
`thaiaddress.addressid = CONCAT(chwpart, amppart, tmbpart)`. It covers 99.99% of
patients. `thaiaddress.name` is the subdistrict name. `pocode` is missing on
7.7% of subdistricts, so some forms will have no postal code. `full_name` needs
a format check before it is split into district and province. `patient_address`
has 0 rows and is not used.

If a component is unknown, leave the dotted line blank. Do not print a wrong
value. The operator may complete it by hand.

**Hospital profile warning:** The verified `hospital_profile` row on Sra Bos has
the hospital name, phone, and fax, but the address is incomplete and conflicts
with the hospital name. The pharmacist must correct it before the first real
print. A settings override is allowed for the printed header.

### 4.4 Quantity and unit

The form asks for the quantity and a unit such as Amp, Vial, Tab, Cap, or Patch.

- The quantity comes from `opitemrece.qty`, which is int(11) on Sra Bos.
- The app does not support decimal quantities in v1. There are 0 / 198,392
  decimal rows on the verified instance.
- The unit comes from the catalog `print_unit`.
- The raw HOSxP unit may be Thai, for example `เม็ด`.
- The app must not print a Thai dispensing unit in the quantity column unless
  the pharmacist has approved that unit for the form.
- If the catalog has no print unit, block printing and ask the pharmacist to
  configure it.

### 4.5 Directions for use

The order may carry a `drugusage` code and an `sp_use` code. The connector joins
the lookup tables and returns the available text.

Preferred text order must be verified against real data. A safe default is:

1. The longest non-empty text from the sig lookup.
2. The catalog default sig.
3. Block printing if both are empty.

The app must not generate a sig from the dose and frequency unless the
pharmacist explicitly approves that behavior.

### 4.6 Legal category

The catalog entry has one of:

- `NARCOTIC_2` for ยาเสพติดให้โทษในประเภท ๒
- `PSYCHOTROPIC_2` for วัตถุออกฤทธิ์ในประเภท ๒

The form checks the boxes from the line categories.

**Verified on Sra Bos:** `drugitems.drug_control_type_id` exists but is empty
on all 659 drugitems. `opitemrece_narcotic` has 0 rows. The catalog
`law_category` is the only source of truth and must be filled by the pharmacist
before the pilot. This is blocker B4.

Open question: whether a single form may contain both categories. If the
pharmacist says no, the app must split the prescription by category and print
separate forms.

## 5. Print rules

### 5.1 Must

- Print A4 portrait.
- Print black text on white paper.
- Print the exact official Thai text.
- Print the form code in the top-left corner.
- Print the title centered, bold, and underlined.
- Print the date, prescriber block, category checkboxes, table, patient block,
  signature line, notes, and legal box.
- Leave the signature line blank for the prescriber.
- Print the prescriber name and licence number in the prescriber block.
- Print the patient name and CID in the patient block.
- Print up to five drug rows.
- Print the legal category checkbox or checkboxes.
- Use a font that matches the official form as closely as possible.

### 5.2 Must not

- Do not print the app logo.
- Do not print brand colors, gradients, or shadows.
- Do not print a signature image or a generated signature.
- Do not print a barcode or QR code unless the official form requires it.
- Do not add marketing text.
- Do not remove the legal box.
- Do not change the official wording.
- Do not print a drug row that is not in the configured catalog.
- Do not print an incomplete required field.

### 5.3 Copies

The official notes mention:

- The patient sends the original to the dispensing hospital.
- The dispensing hospital keeps the original as evidence.
- The prescribing hospital keeps a copy.

The app must confirm the required number of copies with the pharmacist. Options:

1. Print one original. The hospital photocopies it for its file.
2. Print two copies: original and copy. Marking them may deviate from the
   official form, so this needs approval.
3. Print one original and one copy with a small margin label outside the
   official content. This also needs approval.

Do not implement a copy policy until it is confirmed.

### 5.4 More than five drug lines

The official form has five rows. Options:

1. Block printing and require the operator to split the prescription into
   multiple forms. This is the v1 default.
2. Print multiple official forms, each complete, with a page marker outside the
   official content.
3. Use a continuation sheet approved by the Narcotics Control Division.

Do not implement options 2 or 3 without written approval.

### 5.5 Prescription number and hospital stamp

The supplied form has no visible prescription number, book number, or hospital
stamp field. Confirm whether the hospital adds one by hand or by stamp.

If a number is required, it must be printed in a way that does not alter the
official layout. A separate approval is needed.

## 6. Legal and operational checklist

These questions must be answered by the pharmacist, the hospital legal or
quality team, and the Narcotics Control Division as appropriate.

| ID | Question | Why it matters | Answer |
|---|---|---|---|
| L1 | Is the supplied PDF the current official form? | The form may have a revised edition | |
| L2 | Can one form contain both Category 2 narcotics and Category 2 psychotropics? | Determines checkbox behavior and splitting | |
| L3 | Is a five-row limit acceptable? | Determines v1 scope and pagination | |
| L4 | Is a continuation form allowed? | Needed for more than five lines | |
| L5 | How many copies must be printed? | Original, prescribing hospital copy, or both | |
| L6 | Is a prescription number, book number, or hospital stamp required? | May require an extra field | |
| L7 | Is the patient address mandatory on the printed form? | Determines whether missing address blocks print | |
| L8 | Is the patient phone mandatory? | Determines validation | |
| L9 | Is the prescriber licence number mandatory? | Determines whether missing licence blocks print | |
| L10 | Is a handwritten signature required in ink? | Determines whether digital signatures are allowed | |
| L11 | Is a hospital stamp required? | May require a stamp area | |
| L12 | Is the patient's CID mandatory, or is another official ID acceptable? | Determines identity validation | |
| L13 | Is the trade name column mandatory? | Some drugs have no trade name | |
| L14 | Are decimal quantities allowed for liquids? | Determines quantity validation | No on Sra Bos: `qty` is int(11), 0 / 198,392 decimal rows |
| L15 | Which units are approved: Amp, Vial, Tab, Cap, Patch, mL, other? | Determines catalog unit options | |
| L16 | Is IPD in scope? | Determines visit and order queries | |
| L17 | Is a local print audit log required? | Determines privacy and retention design | |
| L18 | What is the retention policy for printed forms and PDFs? | Determines operational policy | |
| L19 | Which font is acceptable for the official print? | Determines font bundling | |
| L20 | Who may print: all staff, pharmacists only, or a new role? | Determines authorization | |

## 7. Print acceptance test

Before the pilot, print the form on the hospital printer and compare it with the
official PDF.

Checklist:

- [ ] A4 portrait, no scaling to fit.
- [ ] Top-left form code present.
- [ ] Title centered, bold, underlined.
- [ ] Date line aligned right.
- [ ] Prescriber block complete.
- [ ] Both category checkboxes present.
- [ ] Table columns and widths match.
- [ ] Five rows present.
- [ ] Patient block complete.
- [ ] Signature line blank.
- [ ] Notes 1 to 3 present.
- [ ] Legal box present.
- [ ] No app logo or brand color.
- [ ] No browser header or footer.
- [ ] Thai text is not clipped or overlapping.
- [ ] Font size is legible at arm's length.
- [ ] Barcode or QR absent unless approved.
- [ ] Page count is one for up to five rows.

## 8. Static print reference

[`print-template.html`](print-template.html) is a standalone HTML file with the
official layout and fictional sample data. Open it in a browser and print to A4
to review the layout. It is a reference, not production code.

The final Leptos component must use the same measurements and must be reviewed
against this reference.
