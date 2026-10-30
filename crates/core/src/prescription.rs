//! Order-line to form-line mapping.

use std::collections::BTreeMap;

use addict_order_models::{Catalog, LawCategory, NarcoticDrugConfig, OrderLine, PrescriptionLine};

/// Errors raised while mapping raw order lines to the official form.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum FormError {
    #[error("no catalog entry for icode {0}")]
    MissingCatalogEntry(String),
    #[error("catalog entry {0} has no law category")]
    MissingLawCategory(String),
    #[error("order line has no printable identity")]
    InvalidLine,
}

/// Maps raw order lines to printable lines using the pharmacist-managed
/// catalog. Identical `(icode, sig, doctor)` groups are merged and their
/// quantities summed. Different sigs or doctors stay separate.
pub fn build_lines(
    order_lines: &[OrderLine],
    catalog: &Catalog,
) -> Result<Vec<PrescriptionLine>, FormError> {
    let mut grouped: BTreeMap<(String, String, Option<String>), PrescriptionLine> = BTreeMap::new();

    for order in order_lines {
        if order.quantity <= 0 {
            continue;
        }
        let config = catalog
            .find_active(&order.icode)
            .ok_or_else(|| FormError::MissingCatalogEntry(order.icode.clone()))?;
        let line = line_from_order(order, config)?;
        let key = (
            line.icode.clone(),
            line.sig.clone(),
            line.source_doctor_code.clone(),
        );
        if let Some(existing) = grouped.get_mut(&key) {
            existing.quantity = existing.quantity.saturating_add(line.quantity);
        } else {
            grouped.insert(key, line);
        }
    }

    Ok(grouped.into_values().collect())
}

fn line_from_order(
    order: &OrderLine,
    config: &NarcoticDrugConfig,
) -> Result<PrescriptionLine, FormError> {
    let law_category = config
        .law_category
        .ok_or_else(|| FormError::MissingLawCategory(config.icode.clone()))?;
    let generic_name = first_non_empty(&config.generic_name, &order.generic_name);
    let trade_name = first_non_empty(&config.trade_name, &order.trade_name);
    let strength_text = first_non_empty(&config.strength_text, &order.strength);
    let sig = first_non_empty(&order.sig_text, &config.default_sig);
    let line_id = if order.hos_guid.trim().is_empty() {
        format!(
            "{}:{}:{}",
            order.vn,
            order.item_no.unwrap_or_default(),
            order.icode
        )
    } else {
        order.hos_guid.clone()
    };

    if generic_name.is_empty() && strength_text.is_empty() {
        return Err(FormError::InvalidLine);
    }

    Ok(PrescriptionLine {
        line_id,
        icode: order.icode.clone(),
        generic_name,
        trade_name,
        strength_text,
        sig,
        quantity: order.quantity,
        print_unit: config.print_unit.clone(),
        law_category,
        source_vn: order.vn.clone(),
        source_item_no: order.item_no,
        source_doctor_code: order.doctor_code.clone(),
    })
}

fn first_non_empty(primary: &str, fallback: &str) -> String {
    let primary = primary.trim();
    if !primary.is_empty() {
        primary.to_string()
    } else {
        fallback.trim().to_string()
    }
}

/// Derives the two legal-category checkboxes from the lines.
pub fn category_flags(lines: &[PrescriptionLine]) -> (bool, bool) {
    let mut narcotic = false;
    let mut psychotropic = false;
    for line in lines {
        match line.law_category {
            LawCategory::Narcotic2 => narcotic = true,
            LawCategory::Psychotropic2 => psychotropic = true,
        }
    }
    (narcotic, psychotropic)
}

#[cfg(test)]
mod tests {
    use super::*;
    use addict_order_models::CategoryFlags;

    fn catalog() -> Catalog {
        let mut catalog = Catalog::default();
        catalog.upsert(NarcoticDrugConfig {
            icode: "1001".into(),
            generic_name: "Morphine sulfate".into(),
            trade_name: "MST".into(),
            strength_text: "30 mg".into(),
            print_unit: "Tab".into(),
            law_category: Some(LawCategory::Narcotic2),
            default_sig: "1 tab twice daily".into(),
            sort_order: 0,
            active: true,
        });
        catalog
    }

    fn order(guid: &str, qty: i64, sig: &str, doctor: Option<&str>) -> OrderLine {
        OrderLine {
            vn: "690001".into(),
            hn: "0000001".into(),
            icode: "1001".into(),
            item_no: Some(1),
            hos_guid: guid.into(),
            quantity: qty,
            drugusage_code: None,
            sp_use_code: None,
            sig_text: sig.into(),
            doctor_code: doctor.map(str::to_string),
            doctor_name: "นพ.วิชัย".into(),
            doctor_license_no: Some("12345".into()),
            generic_name: "Morphine sulfate".into(),
            trade_name: "MST".into(),
            strength: "30 mg".into(),
            source_units: "เม็ด".into(),
            order_date: None,
            order_time: None,
        }
    }

    #[test]
    fn maps_one_order_to_one_line() {
        let lines = build_lines(
            &[order("g1", 10, "1 tab twice daily", Some("0001"))],
            &catalog(),
        )
        .expect("mapping succeeds");
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].quantity, 10);
        assert_eq!(lines[0].print_unit, "Tab");
        assert_eq!(lines[0].law_category, LawCategory::Narcotic2);
    }

    #[test]
    fn merges_identical_lines_and_keeps_different_sigs_separate() {
        let lines = build_lines(
            &[
                order("g1", 10, "1 tab twice daily", Some("0001")),
                order("g2", 5, "1 tab twice daily", Some("0001")),
                order("g3", 3, "1 tab at night", Some("0001")),
            ],
            &catalog(),
        )
        .expect("mapping succeeds");
        assert_eq!(lines.len(), 2);
        let merged = lines
            .iter()
            .find(|line| line.sig == "1 tab twice daily")
            .expect("merged line");
        assert_eq!(merged.quantity, 15);
    }

    #[test]
    fn missing_catalog_entry_is_an_error() {
        let mut catalog = catalog();
        catalog.drugs.clear();
        let error = build_lines(&[order("g1", 10, "sig", None)], &catalog).expect_err("must fail");
        assert_eq!(error, FormError::MissingCatalogEntry("1001".into()));
    }

    #[test]
    fn missing_law_category_is_an_error() {
        let mut catalog = catalog();
        catalog.drugs[0].law_category = None;
        let error = build_lines(&[order("g1", 10, "sig", None)], &catalog).expect_err("must fail");
        assert_eq!(error, FormError::MissingLawCategory("1001".into()));
    }

    #[test]
    fn category_flags_cover_both_categories() {
        let lines = build_lines(&[order("g1", 10, "sig", None)], &catalog()).expect("mapping");
        let flags = CategoryFlags::from_lines(&lines);
        assert!(flags.narcotic_2);
        assert!(!flags.psychotropic_2);
    }
}
