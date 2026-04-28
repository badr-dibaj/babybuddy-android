use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

// ─── Baby ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Baby {
    pub id: String,
    pub name: String,
    pub birth_date: NaiveDate,
    pub photo_path: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Baby {
    /// Returns age in days.
    pub fn age_days(&self) -> i64 {
        let today = Utc::now().date_naive();
        (today - self.birth_date).num_days()
    }

    /// Returns age as human-readable string: e.g. "3 months", "1 year 2 months".
    pub fn age_display(&self) -> String {
        let days = self.age_days();
        if days < 7 {
            format!("{} day{}", days, if days == 1 { "" } else { "s" })
        } else if days < 30 {
            let w = days / 7;
            format!("{} week{}", w, if w == 1 { "" } else { "s" })
        } else if days < 365 {
            let m = days / 30;
            format!("{} month{}", m, if m == 1 { "" } else { "s" })
        } else {
            let y = days / 365;
            let m = (days % 365) / 30;
            if m == 0 {
                format!("{} year{}", y, if y == 1 { "" } else { "s" })
            } else {
                format!("{} yr {} mo", y, m)
            }
        }
    }
}

// ─── Feeding ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FeedingMethod {
    LeftBreast,
    RightBreast,
    BothBreasts,
    Bottle,
    SolidFood,
    FortifiedBreastMilk,
}

impl FeedingMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            FeedingMethod::LeftBreast => "left breast",
            FeedingMethod::RightBreast => "right breast",
            FeedingMethod::BothBreasts => "both breasts",
            FeedingMethod::Bottle => "bottle",
            FeedingMethod::SolidFood => "solid food",
            FeedingMethod::FortifiedBreastMilk => "fortified breast milk",
        }
    }
    pub fn from_str(s: &str) -> Self {
        match s {
            "left breast" => FeedingMethod::LeftBreast,
            "right breast" => FeedingMethod::RightBreast,
            "both breasts" => FeedingMethod::BothBreasts,
            "bottle" => FeedingMethod::Bottle,
            "solid food" => FeedingMethod::SolidFood,
            "fortified breast milk" => FeedingMethod::FortifiedBreastMilk,
            _ => FeedingMethod::Bottle,
        }
    }
    pub fn display(&self) -> &'static str {
        match self {
            FeedingMethod::LeftBreast => "Left Breast",
            FeedingMethod::RightBreast => "Right Breast",
            FeedingMethod::BothBreasts => "Both Breasts",
            FeedingMethod::Bottle => "Bottle",
            FeedingMethod::SolidFood => "Solid Food",
            FeedingMethod::FortifiedBreastMilk => "Fortified Breast Milk",
        }
    }
    pub fn icon(&self) -> &'static str {
        match self {
            FeedingMethod::LeftBreast | FeedingMethod::RightBreast | FeedingMethod::BothBreasts => "🤱",
            FeedingMethod::Bottle => "🍼",
            FeedingMethod::SolidFood => "🥣",
            FeedingMethod::FortifiedBreastMilk => "🍼",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feeding {
    pub id: String,
    pub baby_id: String,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub method: FeedingMethod,
    pub type_: String,
    pub amount: Option<f64>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl Feeding {
    /// Duration in minutes (None if no end time).
    pub fn duration_minutes(&self) -> Option<i64> {
        self.ended_at.map(|e| (e - self.started_at).num_minutes())
    }

    pub fn duration_display(&self) -> String {
        match self.duration_minutes() {
            Some(m) if m < 60 => format!("{} min", m),
            Some(m) => format!("{}h {:02}m", m / 60, m % 60),
            None => "ongoing".to_string(),
        }
    }
}

// ─── Diaper ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diaper {
    pub id: String,
    pub baby_id: String,
    pub time: DateTime<Utc>,
    pub wet: bool,
    pub solid: bool,
    pub color: Option<String>,
    pub amount: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl Diaper {
    pub fn type_display(&self) -> &'static str {
        match (self.wet, self.solid) {
            (true, true) => "Wet + Solid",
            (true, false) => "Wet",
            (false, true) => "Solid",
            (false, false) => "Dry",
        }
    }
    pub fn icon(&self) -> &'static str {
        match (self.wet, self.solid) {
            (true, true) => "💧💩",
            (true, false) => "💧",
            (false, true) => "💩",
            _ => "🩹",
        }
    }
}

// ─── Weight ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Weight {
    pub id: String,
    pub baby_id: String,
    pub date_: NaiveDate,
    pub weight: f64, // kg
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

// ─── Height ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Height {
    pub id: String,
    pub baby_id: String,
    pub date_: NaiveDate,
    pub height: f64, // cm
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

// ─── Head Circumference ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeadCircumference {
    pub id: String,
    pub baby_id: String,
    pub date_: NaiveDate,
    pub head_circumference: f64, // cm
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

// ─── Tooth ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tooth {
    pub id: String,
    pub baby_id: String,
    pub date_: NaiveDate,
    pub tooth_id: i32, // 0-19
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl Tooth {
    pub fn name(&self) -> &'static str {
        TOOTH_NAMES
            .get(self.tooth_id as usize)
            .copied()
            .unwrap_or("Unknown tooth")
    }
}

pub const TOOTH_NAMES: &[&str] = &[
    "Upper Central Incisor (R)",  // 0
    "Upper Central Incisor (L)",  // 1
    "Upper Lateral Incisor (R)",  // 2
    "Upper Lateral Incisor (L)",  // 3
    "Upper Canine (R)",           // 4
    "Upper Canine (L)",           // 5
    "Upper First Molar (R)",      // 6
    "Upper First Molar (L)",      // 7
    "Upper Second Molar (R)",     // 8
    "Upper Second Molar (L)",     // 9
    "Lower Central Incisor (R)",  // 10
    "Lower Central Incisor (L)",  // 11
    "Lower Lateral Incisor (R)",  // 12
    "Lower Lateral Incisor (L)",  // 13
    "Lower Canine (R)",           // 14
    "Lower Canine (L)",           // 15
    "Lower First Molar (R)",      // 16
    "Lower First Molar (L)",      // 17
    "Lower Second Molar (R)",     // 18
    "Lower Second Molar (L)",     // 19
];

// ─── Medication ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Medication {
    pub id: String,
    pub baby_id: String,
    pub time: DateTime<Utc>,
    pub name: String,
    pub dosage: Option<f64>,
    pub dosage_unit: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl Medication {
    pub fn dosage_display(&self) -> String {
        match (&self.dosage, &self.dosage_unit) {
            (Some(d), Some(u)) => format!("{} {}", d, u),
            (Some(d), None) => format!("{}", d),
            _ => String::new(),
        }
    }
}

// ─── Dashboard Summary ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default)]
pub struct DaySummary {
    pub baby_name: String,
    pub feedings_today: usize,
    pub diapers_today: usize,
    pub last_feeding: Option<DateTime<Utc>>,
    pub last_diaper: Option<DateTime<Utc>>,
    pub latest_weight: Option<f64>,
    pub latest_height: Option<f64>,
}
