//! The pharmacist-managed narcotic catalog.

use serde::{Deserialize, Serialize};

/// Legal category printed on the official form.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LawCategory {
    #[serde(rename = "NARCOTIC_2")]
    Narcotic2,
    #[serde(rename = "PSYCHOTROPIC_2")]
    Psychotropic2,
}

impl LawCategory {
    pub fn label_th(self) -> &'static str {
        match self {
            Self::Narcotic2 => "ยาเสพติดให้โทษในประเภท ๒",
            Self::Psychotropic2 => "วัตถุออกฤทธิ์ในประเภท ๒",
        }
    }

    pub fn short_label_th(self) -> &'static str {
        match self {
            Self::Narcotic2 => "ย.ส. ๒",
            Self::Psychotropic2 => "ว.จ. ๒",
        }
    }
}

/// One configured icode and the values printed on the official form.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NarcoticDrugConfig {
    pub icode: String,
    pub generic_name: String,
    #[serde(default)]
    pub trade_name: String,
    pub strength_text: String,
    pub print_unit: String,
    #[serde(default)]
    pub law_category: Option<LawCategory>,
    #[serde(default)]
    pub default_sig: String,
    #[serde(default)]
    pub sort_order: i32,
    #[serde(default = "default_active")]
    pub active: bool,
}

fn default_active() -> bool {
    true
}

impl NarcoticDrugConfig {
    pub fn is_print_ready(&self) -> bool {
        !self.icode.trim().is_empty()
            && !self.generic_name.trim().is_empty()
            && !self.strength_text.trim().is_empty()
            && !self.print_unit.trim().is_empty()
            && self.law_category.is_some()
    }
}

/// A `drugitems` search hit used while configuring the catalog.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DrugItem {
    pub icode: String,
    pub name: String,
    #[serde(default)]
    pub trade_name: String,
    #[serde(default)]
    pub strength: String,
    #[serde(default)]
    pub units: String,
    #[serde(default)]
    pub dosageform: String,
    #[serde(default)]
    pub drug_control_type_id: Option<String>,
}

impl DrugItem {
    pub fn strength_label(&self) -> String {
        match (self.strength.trim(), self.units.trim()) {
            ("", "") => String::new(),
            (strength, "") => strength.to_string(),
            ("", units) => units.to_string(),
            (strength, units) => format!("{strength} {units}"),
        }
    }
}

/// The full configured catalog.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Catalog {
    pub drugs: Vec<NarcoticDrugConfig>,
}

impl Catalog {
    pub fn is_empty(&self) -> bool {
        self.drugs.is_empty()
    }

    pub fn active_drugs(&self) -> Vec<&NarcoticDrugConfig> {
        let mut drugs: Vec<&NarcoticDrugConfig> =
            self.drugs.iter().filter(|drug| drug.active).collect();
        drugs.sort_by_key(|drug| (drug.sort_order, drug.generic_name.to_lowercase()));
        drugs
    }

    pub fn icodes(&self) -> Vec<String> {
        self.active_drugs()
            .into_iter()
            .map(|drug| drug.icode.clone())
            .collect()
    }

    pub fn find(&self, icode: &str) -> Option<&NarcoticDrugConfig> {
        self.drugs.iter().find(|drug| drug.icode == icode)
    }

    pub fn find_active(&self, icode: &str) -> Option<&NarcoticDrugConfig> {
        self.drugs
            .iter()
            .find(|drug| drug.icode == icode && drug.active)
    }

    /// Adds or replaces one catalog entry, keeping the list sorted by
    /// `sort_order` then generic name.
    pub fn upsert(&mut self, drug: NarcoticDrugConfig) {
        if let Some(existing) = self.drugs.iter_mut().find(|item| item.icode == drug.icode) {
            *existing = drug;
        } else {
            self.drugs.push(drug);
        }
        self.drugs
            .sort_by_key(|item| (item.sort_order, item.generic_name.to_lowercase()));
    }

    pub fn remove(&mut self, icode: &str) {
        self.drugs.retain(|drug| drug.icode != icode);
    }

    pub fn validation_errors(&self) -> Vec<String> {
        let mut errors = Vec::new();
        for drug in &self.drugs {
            if drug.icode.trim().is_empty() {
                errors.push("พบรายการที่ไม่มี icode".to_string());
                continue;
            }
            if drug.generic_name.trim().is_empty() {
                errors.push(format!("{}: ไม่มีชื่อตัวยาสำคัญ", drug.icode));
            }
            if drug.strength_text.trim().is_empty() {
                errors.push(format!("{}: ไม่มีความแรง", drug.icode));
            }
            if drug.print_unit.trim().is_empty() {
                errors.push(format!("{}: ไม่มีหน่วยพิมพ์", drug.icode));
            }
            if drug.law_category.is_none() {
                errors.push(format!("{}: ยังไม่ระบุประเภททางกฎหมาย", drug.icode));
            }
        }
        errors
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn drug(icode: &str, category: Option<LawCategory>) -> NarcoticDrugConfig {
        NarcoticDrugConfig {
            icode: icode.into(),
            generic_name: "Morphine sulfate".into(),
            trade_name: "MST".into(),
            strength_text: "30 mg".into(),
            print_unit: "Tab".into(),
            law_category: category,
            default_sig: "1 tab twice daily".into(),
            sort_order: 0,
            active: true,
        }
    }

    #[test]
    fn active_drugs_are_sorted_and_filtered() {
        let mut catalog = Catalog::default();
        let mut inactive = drug("2", Some(LawCategory::Narcotic2));
        inactive.active = false;
        catalog.upsert(drug("1", Some(LawCategory::Narcotic2)));
        catalog.upsert(inactive);
        assert_eq!(catalog.icodes(), vec!["1"]);
    }

    #[test]
    fn validation_requires_the_print_fields() {
        let mut catalog = Catalog::default();
        catalog.upsert(drug("1", None));
        let errors = catalog.validation_errors();
        assert!(errors.iter().any(|error| error.contains("ประเภททางกฎหมาย")));
        catalog.drugs[0].law_category = Some(LawCategory::Narcotic2);
        assert!(catalog.validation_errors().is_empty());
    }

    #[test]
    fn law_category_serde_uses_the_form_contract() -> Result<(), serde_json::Error> {
        assert_eq!(
            serde_json::to_string(&LawCategory::Narcotic2)?,
            "\"NARCOTIC_2\""
        );
        assert_eq!(
            serde_json::to_string(&LawCategory::Psychotropic2)?,
            "\"PSYCHOTROPIC_2\""
        );
        let parsed: LawCategory = serde_json::from_str("\"NARCOTIC_2\"")?;
        assert_eq!(parsed, LawCategory::Narcotic2);
        Ok(())
    }

    #[test]
    fn upsert_replaces_the_same_icode() {
        let mut catalog = Catalog::default();
        catalog.upsert(drug("1", Some(LawCategory::Narcotic2)));
        let mut updated = drug("1", Some(LawCategory::Psychotropic2));
        updated.generic_name = "Updated".into();
        catalog.upsert(updated);
        assert_eq!(catalog.drugs.len(), 1);
        assert_eq!(catalog.drugs[0].generic_name, "Updated");
    }
}
