# Design System - Addict Order

## 1. Concept

Addict Order is a controlled-document workstation. The user is not browsing a
dashboard. The user is preparing a legal paper that a prescriber will sign.
The interface must feel secure, precise, and easy to audit.

The visual system is derived from `icon-master.svg`: a medicine lockbox with a
deep blue gradient, a signal-yellow seal band, black outlines, and a hard
offset shadow. The design language is called **Vault Blue**.

Three ideas drive the system:

1. **The cabinet is blue.** Deep blue is the brand, the login surface, and the
   trust anchor.
2. **The seal is yellow.** Yellow marks the action, the selected item, and the
   thing that needs attention. It is not decoration.
3. **The document is paper.** The official form is black on white. It never
   receives brand colors, gradients, or shadows.

The app chrome can be bold. The paper must be formal.

## 2. Signature element

The signature element is the **official A4 form on the vault desk**.

- The form is centered on a light blue-gray desk.
- The desk has a faint 24 px grid, like a drafting table.
- The paper has a 2 px black border and a hard 12 px offset shadow.
- The form itself is pure white with black ink.
- The preview is the print output. There is no separate print layout.

The second signature element is the **login brand panel**: the blue gradient
from the SVG, the lockbox mark at large scale, and a yellow seal line.

## 3. Color

### 3.1 Brand

| Token | Value | Role |
|---|---|---|
| `--blue-700` | `#1D4ED8` | Primary brand blue, links, selected info |
| `--blue-900` | `#1E3A8A` | Deep brand blue, login gradient end |
| `--blue-500` | `#3B82F6` | Lockbox body, secondary blue |
| `--blue-100` | `#DBEAFE` | Selected row tint, info background |
| `--blue-50` | `#EFF6FF` | Quiet blue surface |
| `--yellow-400` | `#FACC15` | Signal yellow, primary action, seal band |
| `--yellow-100` | `#FEF9C3` | Selected card tint, warning-adjacent highlight |
| `--ink` | `#111111` | Text, official paper outline, primary action |
| `--paper` | `#FFFFFF` | The official form and card surfaces |
| `--desk` | `#E8EDF7` | The working desk behind the paper |
| `--desk-grid` | `rgba(29, 78, 216, 0.035)` | Drafting grid lines |

### 3.2 Text

| Token | Value | Role |
|---|---|---|
| `--text-primary` | `#111111` | Headings, body, table data |
| `--text-secondary` | `#4B5563` | Metadata, helper text |
| `--text-muted` | `#6B7280` | Placeholders, disabled labels |
| `--text-on-blue` | `#FFFFFF` | Text on blue surfaces |
| `--text-on-yellow` | `#111111` | Text on yellow surfaces |

### 3.3 Semantic

| Token | Value | Role |
|---|---|---|
| `--success` | `#15803D` | Confirmed, complete, connected |
| `--success-soft` | `#DCFCE7` | Success background |
| `--warning` | `#B45309` | Attention, incomplete optional field |
| `--warning-soft` | `#FEF3C7` | Warning background |
| `--error` | `#DC2626` | Required field missing, failed action |
| `--error-soft` | `#FEE2E2` | Error background |
| `--focus` | `#FACC15` | Keyboard focus ring |
| `--line` | `#111111` | Strong outline: paper, primary action, dialogs |
| `--line-soft` | `#CBD5E1` | 1 px chrome borders and quiet dividers |
| `--line-control` | `#94A3B8` | Form-control borders |

### 3.4 Color rules

- Yellow is a signal. Use it for the primary action, the selected item, and
  attention. Do not use it for large backgrounds.
- Blue is trust and information. Use it for the brand, links, selected info,
  and the connection state. Do not use blue for errors.
- Black is reserved for the official paper, the primary action, and dialogs.
  Screen chrome uses 1 px slate borders so the document stays the only strong
  object on the desk.
- Red is failure. Use it only for required-field errors and destructive
  actions.
- Green is completion. Use it only for confirmed and connected states.
- The printed form uses black and white only.

## 4. Typography

### 4.1 UI fonts

| Role | Font | Fallback |
|---|---|---|
| Latin UI | Inter | `-apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif` |
| Thai UI | IBM Plex Sans Thai | `"Leelawadee UI", Tahoma, sans-serif` |
| Codes and numbers | IBM Plex Mono | `"SF Mono", Menlo, Consolas, monospace` |

Assign fonts by content role, not by fallback luck:

- Thai text: IBM Plex Sans Thai.
- Latin labels and headings: Inter.
- HN, CID, icode, dates, quantities, and codes: IBM Plex Mono with
  `font-variant-numeric: tabular-nums`.
- Mixed Thai and Latin: Inter first, IBM Plex Sans Thai second, so Latin glyphs
  use Inter and Thai glyphs use Plex Thai.

### 4.2 UI scale

| Token | Size | Weight | Line height | Use |
|---|---|---|---|---|
| `display` | 32 px | 800 | 1.1 | Login brand headline |
| `h1` | 24 px | 700 | 1.2 | Screen title |
| `h2` | 20 px | 700 | 1.25 | Section title |
| `h3` | 16 px | 700 | 1.3 | Card title, dialog title |
| `body` | 15 px | 400 | 1.5 | Primary text |
| `body-medium` | 15 px | 500 | 1.5 | Emphasized data |
| `small` | 13 px | 400 | 1.45 | Helper text |
| `label` | 13 px | 700 | 1.3 | Field labels |
| `caption` | 12 px | 400 | 1.4 | Metadata |
| `micro` | 11 px | 700 | 1.3 | Uppercase chips, table headers |
| `button` | 14 px | 700 | 1.2 | Buttons |
| `mono` | 14 px | 400 | 1.4 | Codes, HN, CID |
| `mono-sm` | 13 px | 400 | 1.4 | Table numbers |

Use weight and spacing for emphasis, not extra colors. Keep body text at 15 px
or larger. Do not set table data below 12.5 px.

### 4.3 Print fonts

The official form uses TH SarabunPSK. The app should bundle a verified font.

| Role | Size | Weight |
|---|---|---|
| Title | 18 pt | Bold |
| Form code | 14 pt | Bold, gray |
| Body and date | 15 pt | Regular |
| Table header and body | 14 pt | Bold header, regular body |
| Patient block | 14 pt | Regular |
| Notes and legal box | 12 pt | Regular |

If TH SarabunPSK cannot be bundled, use Sarabun (OFL) and verify the printed
layout. Do not use a Latin-only font for Thai print text.

## 5. Shape and elevation

### 5.1 Shape

The SVG is square and geometric. The UI follows it.

| Token | Value | Use |
|---|---|---|
| `radius-0` | `0` | The official paper, table cells, hard panels |
| `radius-sm` | `2 px` | Buttons, inputs, badges |
| `radius-md` | `4 px` | Cards, dialogs |
| `radius-full` | `9999 px` | Only status dots and avatars |

Do not use large rounded corners. The app is a document station, not a
consumer app.

### 5.2 Stroke hierarchy

Strokes carry hierarchy. Use the lightest stroke that still separates the
element from its background.

| Level | Stroke | Use |
|---|---|---|
| Strong | `2 px #111111` | Official A4 sheet, primary and accent buttons, login card, dialogs |
| Control | `1 px #94A3B8` | Text inputs, selects, search field, small icon buttons |
| Chrome | `1 px #CBD5E1` | Top bar and command bar separators, step cards, inspector cards, list rows, settings nav |

Rules:

- Never nest one strong outline inside another. A card inside a dialog uses a
  chrome stroke, not a strong one.
- Do not put a hard shadow on chrome cards. Use spacing, a 1 px rule, and the
  surface tint instead.
- Status colors (error, warning) may override the stroke color but not the
  weight.
- The printed form is separate and keeps its black form rules.

### 5.3 Shadows

The SVG uses a solid offset shadow with no blur. On screen, that hard shadow
is reserved for objects that float above the desk.

| Token | Value | Use |
|---|---|---|
| `shadow-sm` | `3px 3px 0 #111111` | Primary and accent buttons, tooltips, banners |
| `shadow-md` | `6px 6px 0 #111111` | Search results, empty state |
| `shadow-lg` | `10px 10px 0 #111111` | Dialogs, login card |
| `shadow-paper` | `12px 12px 0 #111111` | The A4 sheet on the desk |

Rules:

- No soft blurred shadows for structural elements.
- The shadow is part of the object, not a glow. Interface cards and inspector
  panels have no shadow at all.
- On press, the primary or accent button moves 2 px toward the shadow and the
  shadow shrinks.
- The printed form has no shadow.

## 6. Layout

### 6.1 Desktop window

- Default: 1280 x 900.
- Minimum: 1024 x 700.
- This is a mouse and keyboard desktop app. There is no mobile layout.

### 6.2 Login gate

```text
┌──────────────────────────────────────────────────────────────────────┐
│  BLUE BRAND PANEL (52%)                  │  WHITE LOGIN PANEL (48%)  │
│                                          │                           │
│  [lockbox mark]  Addict Order            │        [ตั้งค่า]          │
│  ระบบใบสั่งจ่าย ย.ส. ๒ / ว.จ. ๒          │                           │
│                                          │   ┌───────────────────┐   │
│  Controlled document.                    │   │ YELLOW SEAL BAR   │   │
│  Verified source.                        │   ├───────────────────┤   │
│  One official form.                      │   │ เข้าสู่ระบบ        │   │
│                                          │   │ ชื่อผู้ใช้ [____] │   │
│  [large lockbox illustration]            │   │ รหัสผ่าน  [____] │   │
│                                          │   │ [เข้าสู่ระบบ]      │   │
│                                          │   └───────────────────┘   │
└──────────────────────────────────────────────────────────────────────┘
```

Details:

- Left panel: `linear-gradient(135deg, #1D4ED8, #1E3A8A)`, matching the SVG.
- A faint 24 px grid in white at 4% opacity.
- The lockbox mark is 220 px or larger, with the SVG hard shadow.
- The app name is Inter 800, white.
- The Thai subtitle is IBM Plex Sans Thai, blue-100.
- The right panel is `--desk`.
- The login card is white, 2 px black border, `shadow-lg`, radius 4 px.
- The card has a 10 px yellow top bar.
- The primary button is black with yellow text and `shadow-sm`.
- The settings action is a ghost button with a gear icon.
- If the connection is not configured, show an amber notice inside the card
  with a "ตั้งค่าการเชื่อมต่อ" button.

### 6.3 Main shell

```text
┌──────────────────────────────────────────────────────────────────────┐
│ TOP BAR  [mark] Addict Order · ระบบใบสั่งจ่าย · ● │ user · logout    │
├──────────────────────────────────────────────────────────────────────┤
│ COMMAND BAR                                                          │
│ [1 ผู้ป่วย: search.......] [2 Visit: select + date] [3 ผู้ออกใบสั่ง] │
│ [4 ตรวจสอบและพิมพ์ · status........] [ดึงรายการ] [ยืนยัน] [พิมพ์]    │
├──────────────────────────────────────────────────────────────────────┤
│ STAGE (vault desk)                          │ INSPECTOR               │
│                                             │ Patient                 │
│        ┌─────────────────────────┐          │ Visit                   │
│        │  OFFICIAL A4 FORM       │          │ Prescriber              │
│        │  (preview = print)      │          │ Checks                  │
│        │                         │          │ Warnings                │
│        │                         │          │ [confirm] [print]       │
│        └─────────────────────────┘          │                         │
└──────────────────────────────────────────────────────────────────────┘
```

The inspector is 320 px wide on screens 1280 px and wider. Below 1200 px it
moves below the form as a horizontal summary. The form is always the dominant
object.

### 6.4 Command bar

The command bar is the only control surface above the document. It is a
numbered workflow rail, not a loose stack of fields.

Row 1: three compact step cards.

1. **ผู้ป่วย**
   - The search input is capped by the card width, never the window width.
   - Placeholder: `HN / CID / ชื่อ-สกุล`.
   - Hint below the input: `HN 7 หลัก · CID 13 หลัก · ชื่อหรือนามสกุล`.
   - Results open in an anchored dropdown below the card. They do not reflow
     the stage.
   - When a patient is selected, the search collapses to a patient card with
     HN and name, a masked CID in the card footer, and a "เปลี่ยนผู้ป่วย"
     action in the step header.
2. **Visit**
   - The select fills the card.
   - The prescription date is a read-only footer inside the card, derived from
     the selected visit.
   - The step header shows the number of visits in the current window.
3. **ผู้ออกใบสั่งจ่าย**
   - One picker button shows the selected name.
   - The licence sits in the card footer, aligned with the CID and visit-date
     footers on the other cards.
   - A missing licence is shown as an amber footer warning.
   - The picker is disabled until a patient is selected.
   - A doctor chosen by the operator is pinned for the current patient.
     Fetching orders or changing the visit must not replace that choice. The
     step header shows `เลือกเอง` while the choice is pinned.

Row 2: document actions.

- A full-width `4 · ตรวจสอบและพิมพ์` card aligns with the three step cards.
- A live status line shows the next action, row count (`n / 5`), and print
  state.
- "ดึงรายการยา" is secondary.
- "ยืนยันข้อมูล" is yellow with black text.
- "พิมพ์ใบสั่งจ่าย" is black with yellow text, disabled until confirmed.
- The buttons keep workflow order from left to right: fetch, confirm, print.

Step numbers turn into green check marks when the step is complete. Yellow is
reserved for the primary confirm action; it is not used as a second progress
color.

### 6.5 Stage

- Background: `--desk` with a 24 px grid.
- The A4 form is centered with 40 px padding.
- The form scales to fit the available width. A zoom control shows 75%, 100%,
  or "fit".
- The form is clickable. Clicking the patient block, prescriber block, or a
  drug row opens the matching edit dialog.
- Screen-only edit affordances use a dashed blue outline on hover. They are
  hidden in print.

### 6.6 Inspector

The inspector is a stack of square cards:

1. **Patient**
   - Name, HN, CID (full on the selected card), birth date, sex, phone.
   - Address summary.
   - "แก้ไขที่อยู่" action.
2. **Visit**
   - Date, time, department, doctor.
   - "เปลี่ยน visit" action.
3. **Prescriber**
   - Name, licence number, source badge (`จาก HOSxP` or `กรอกเอง`).
   - "เลือกแพทย์" action.
   - When the operator has pinned a prescriber, show that the choice will not
     be replaced by the order doctor when drug lines are fetched.
4. **Directions for use**
   - One textarea per drug line, pre-filled from the cleaned HOSxP sig.
   - The operator can shorten or correct the text before printing.
   - Editing a sig clears the confirmation snapshot.
5. **Checks**
   - Required fields complete.
   - Address complete.
   - Legal category complete.
   - Row count and five-row limit.
   - The rendered sheet fits one A4 page.
   - Warnings in amber, errors in red.
6. **Actions**
   - Confirm.
   - Print.
   - Clear patient.

The inspector is a convenience. The form preview is the source of truth.

## 7. Components

### 7.1 Buttons

| Variant | Background | Text | Border | Shadow | Use |
|---|---|---|---|---|---|
| `button-primary` | `#111111` | `#FACC15` | 2 px `#111` | `shadow-sm` | Print, dominant action |
| `button-accent` | `#FACC15` | `#111111` | 2 px `#111` | `shadow-sm` | Confirm, selected action |
| `button-secondary` | `#FFFFFF` | `#111111` | 1 px `#94A3B8` | none | Fetch, cancel, secondary |
| `button-blue` | `#1D4ED8` | `#FFFFFF` | 1 px `#1D4ED8` | none | Search, info action |
| `button-danger` | `#DC2626` | `#FFFFFF` | 1 px `#DC2626` | none | Clear, destructive |
| `button-ghost` | transparent | `#111111` | none | none | Quiet actions |

All buttons:

- radius 2 px
- height 40 px, compact 34 px
- padding 0 16 px
- font 14 px / 700
- `:active` translates primary and accent 2 px right and 2 px down, shadow
  becomes 1 px; secondary and blue only change their background
- `:disabled` background `#E5E7EB`, text `#6B7280`, border `#9CA3AF`, no shadow
- focus ring 3 px `#FACC15` with 2 px offset

### 7.2 Inputs

- Background white.
- Border 1 px `#94A3B8`.
- Radius 2 px.
- Height 40 px, compact 34 px.
- Font 15 px.
- Focus: 3 px yellow outline, 1 px control border.
- Error: 2 px red border, red helper text (error states may use the stronger
  weight because they are transient and must stand out).
- Disabled: gray background, no shadow.
- Labels are 13 px / 700, above the field.
- Required labels use a small red dot or the word "จำเป็น".

### 7.3 Search

- The search lives inside step 1 of the command rail and is capped at the step
  width. It must not stretch across the window.
- Input 40 px high, 1 px `#94A3B8` border, radius 2 px, blue magnifier on the
  left.
- Right search button, compact blue with white text and a 1 px blue border.
- Hint below the input: `HN 7 หลัก · CID 13 หลัก · ชื่อหรือนามสกุล`.
- Results panel: anchored below the step card, white, 1 px `#CBD5E1` border,
  `shadow-md`, max height 320 px.
- Result row: HN chip in mono, name, masked CID, right chevron.
- Hover or keyboard focus: `--yellow-100`.
- Empty state: a banner, not an inline results row.

### 7.4 Selected patient card

- Blue-50 background, 1 px blue-700 border with a 4 px blue left accent.
- HN chip in mono and name in Thai on one line.
- Masked CID in the card footer so all three step cards share a bottom rule.
- A "เปลี่ยนผู้ป่วย" text action sits in the step header.
- Replacing the patient clears the visit, drug lines, prescriber, and print
  snapshot.

### 7.5 Visit selector

- A native select that fills the step card. Each option shows date, time, and
  department.
- The prescription date is a read-only footer inside the same card.
- If no visits: the footer shows "ไม่พบ visit ใน 30 วัน" in amber.
- The step header shows the number of visits in the 30-day window.

### 7.6 Drug table

| Column | Width | Style |
|---|---|---|
| ลำดับ | 40 px | Mono, Thai digit |
| ชื่อตัวยาสำคัญ | 1fr | 15 px / 600 |
| ชื่อการค้า | 160 px | 14 px |
| ความแรง | 120 px | Mono |
| ขนาดการใช้ | 1.4fr | 14 px |
| จำนวน | 96 px | Mono, right |
| หน่วย | 80 px | Mono |
| สถานะ | 100 px | Badge |

On the form preview, the table is the official six-column table. The app's
inspector may show a wider internal table with status and edit actions.

### 7.7 Law category badge

- `ย.ส. ๒`: blue-700 background, white text.
- `ว.จ. ๒`: yellow-400 background, black text.
- Unknown: red-soft background, red text, "ยังไม่ระบุ".
- Always include the text. Do not rely on color alone.

### 7.8 Official form preview

- White A4 sheet, 2 px black border, `shadow-paper`.
- The sheet uses the print stylesheet, not a screen-only imitation.
- The screen wrapper may scale the sheet with CSS transform.
- Clickable regions have a screen-only dashed outline.
- The form code, title, date, prescriber block, category checkboxes, table,
  patient block, signature line, notes, and legal box are all present.
- The signature line is blank.
- No app logo.

### 7.9 Confirmation dialog

- Title: "ยืนยันข้อมูลใบสั่งจ่าย".
- Summary grid:
  - patient
  - visit date
  - prescriber and licence
  - hospital
  - row count
  - category flags
- Checklist:
  - required fields complete (green)
  - address complete (green or amber)
  - five-row limit (green or red)
  - licence present (green or red)
  - one-page fit (green or red)
- Buttons:
  - "ยืนยันและล็อกข้อมูล" yellow.
  - "ยกเลิก" secondary.
- After confirmation, the dialog closes and the print button becomes active.
- The confirmation time is shown in the inspector.
- If the rendered sheet exceeds one A4 page, the dialog cannot open. The
  operator shortens the sig in the directions editor; directions are never
  clipped silently.

### 7.10 Settings modal

- Width 960 px, max height 90vh.
- Left nav 200 px: Connection, Catalog, Hospital, Print.
- Active nav: white background, 4 px yellow left bar, black text.
- Inactive nav: desk background, gray text.
- Content area scrolls.
- Footer: export/import settings, status line.
- The catalog panel is the largest:
  - search `drugitems`
  - add icode
  - table with generic name, trade name, strength, unit, law category,
    default sig, active
  - validation badges
- The hospital panel:
  - name, house no, moo, soi, road, subdistrict, district, province, postal
    code, phone, fax
  - a live preview of the hospital block
- The print panel:
  - font choice
  - row limit (fixed at 5 in v1)
  - copy policy (disabled until approved)
  - print test button

### 7.11 Banners

- Inline at the top of the stage, not floating over the paper.
- Success: green-soft, green text, black border.
- Warning: warning-soft, warning text, black border.
- Error: error-soft, error text, black border.
- Dismiss button.
- Success auto-dismisses after 5 seconds. Warnings and errors stay.

## 8. Screen flow

```text
Startup
  -> login gate
      -> settings (connection)
      -> officer login
          -> main shell
              -> search patient
                  -> select patient
                      -> select visit
                          -> pull configured narcotic lines
                              -> review form
                                  -> confirm snapshot
                                      -> print
```

### 8.1 Empty states

- No connection: login card notice with a settings action.
- No catalog: main shell shows "ยังไม่ได้ตั้งค่าบัญชียาเสพติด" with a settings
  action. Printing is disabled.
- No patient: stage shows a quiet desk with a short instruction.
- No visits: visit selector empty state.
- No configured narcotic lines: "ไม่พบรายการยาเสพติดที่ตั้งค่าไว้ใน visit นี้"
  with a link to the catalog settings.
- No prescriber licence: red check in the inspector with a manual entry action.

## 9. Print CSS

The print stylesheet is part of the product, not an afterthought.

```css
@media print {
  @page {
    size: A4 portrait;
    margin: 0;
  }

  .top-bar,
  .command-bar,
  .inspector,
  .banner,
  .dialog-backdrop,
  .screen-only {
    display: none !important;
  }

  .stage {
    display: block;
    padding: 0;
    background: #ffffff;
  }

  .sheet {
    width: 210mm;
    min-height: 297mm;
    margin: 0;
    border: 0;
    box-shadow: none;
    transform: none;
  }
}
```

The final form uses the measured positions from
[`form-spec.md`](form-spec.md). The print reference is
[`print-template.html`](print-template.html).

## 10. Motion

- State transitions: 120 ms ease-out.
- Primary and accent button press: 80 ms, translate 2 px, shadow shrink.
  Secondary and blue buttons only change background.
- Dialog open: 120 ms fade and 4 px rise.
- No bounce, elastic, or spring easing.
- `prefers-reduced-motion: reduce` disables all transitions and animations.
- The selected state appears immediately. The color is the message.

## 11. Accessibility

- All text meets WCAG AA contrast.
- Yellow is never the only signal. Pair it with text or an icon.
- Focus is always visible: 3 px yellow ring with a 2 px offset.
- Buttons have accessible names.
- The search input has a label and a hint.
- The form preview is real text, not an image.
- The confirmation dialog traps focus and restores it on close.
- The connection dot has an `aria-label` with the full Thai status.
- The print button is disabled, not hidden, when not confirmed.
- The app supports keyboard navigation for search, results, visit selection,
  form editing, confirmation, and print.
- Do not use color alone for required, warning, or error states.

## 12. Responsive behavior

| Width | Behavior |
|---|---|
| 1280 px and wider | Form and inspector side by side |
| 1200 to 1279 px | Inspector narrows to 280 px |
| 1024 to 1199 px | Inspector moves below the form as a horizontal summary |
| Below 1024 px | Not supported; show a message that the window is too small |

The form scales to fit the available width. The print output is always 100%
A4 and never scaled.

## 13. Do and do not

### Do

- Use the blue gradient only on the login brand panel.
- Use yellow for the primary action and the selected state.
- Reserved black strokes and hard shadows for the paper, primary actions, and
  dialogs; use 1 px slate strokes for the rest of the screen.
- Keep the official form black on white.
- Show the full CID only on the selected patient card and the form preview.
- Put the prescriber name and licence in the form, but leave the signature
  line blank.
- Show a clear warning when a required field is missing.
- Make the confirmation snapshot explicit and visible.
- Use mono for HN, CID, icode, dates, and quantities.
- Test the print on the hospital printer.

### Do not

- Do not put the app logo or brand color on the official form.
- Do not print a signature or a signature image.
- Do not use rounded consumer-style cards.
- Do not use soft shadows or glows.
- Do not use gradients outside the login brand panel.
- Do not use yellow for errors or red for attention.
- Do not use color alone to communicate legal category or validation.
- Do not build a second print layout.
- Do not scale the printed form to fit the screen.
- Do not show patient data after logout or patient change.
- Do not log patient data.
- Do not add marketing copy to the work surface.

## 14. Design tokens summary

```css
:root {
  --blue-700: #1d4ed8;
  --blue-900: #1e3a8a;
  --blue-500: #3b82f6;
  --blue-100: #dbeafe;
  --blue-50: #eff6ff;
  --yellow-400: #facc15;
  --yellow-100: #fef9c3;
  --ink: #111111;
  --paper: #ffffff;
  --desk: #e8edf7;
  --desk-grid: rgba(29, 78, 216, 0.035);
  --text-primary: #111111;
  --text-secondary: #4b5563;
  --text-muted: #6b7280;
  --success: #15803d;
  --success-soft: #dcfce7;
  --warning: #b45309;
  --warning-soft: #fef3c7;
  --error: #dc2626;
  --error-soft: #fee2e2;
  --line: #111111;
  --line-soft: #cbd5e1;
  --line-control: #94a3b8;
  --focus: #facc15;
  --shadow-sm: 3px 3px 0 #111111;
  --shadow-md: 6px 6px 0 #111111;
  --shadow-lg: 10px 10px 0 #111111;
  --shadow-paper: 12px 12px 0 #111111;
  --radius-0: 0;
  --radius-sm: 2px;
  --radius-md: 4px;
  --font-sans: "Inter", "IBM Plex Sans Thai", sans-serif;
  --font-thai: "IBM Plex Sans Thai", "Leelawadee UI", sans-serif;
  --font-mono: "IBM Plex Mono", "SF Mono", Menlo, Consolas, monospace;
}
```
