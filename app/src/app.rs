// BabyBuddy — core application logic
// Shared between Android (android_main) and Desktop (main)

use crate::db::{
    models::{DaySummary, FeedingMethod},
    Database,
};
use crate::ui::{format_date, format_relative, format_time};

use chrono::{NaiveDate, Utc};
use log::{error, info};
use slint::VecModel;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

// Include the generated Slint code.
slint::include_modules!();

// ─── DB path helpers ─────────────────────────────────────────────────────────

fn db_path_local() -> String {
    // Desktop/test: use current directory
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    format!("{}/.babybuddy.db", home)
}

#[cfg(feature = "android")]
fn db_path_android(app: &android_activity::AndroidApp) -> String {
    app.internal_data_path()
        .map(|p| format!("{}/babybuddy.db", p.to_string_lossy()))
        .unwrap_or_else(|| "/data/data/com.babybuddy.app/babybuddy.db".to_string())
}

// ─── Slint conversion helpers ─────────────────────────────────────────────────

fn to_slint_baby(b: &crate::db::models::Baby) -> BabyData {
    BabyData {
        id: b.id.clone().into(),
        name: b.name.clone().into(),
        birth_date: b.birth_date.format("%Y-%m-%d").to_string().into(),
        age_display: b.age_display().into(),
        photo_path: b.photo_path.clone().unwrap_or_default().into(),
    }
}

fn to_slint_feeding(f: &crate::db::models::Feeding) -> FeedingData {
    FeedingData {
        id: f.id.clone().into(),
        baby_id: f.baby_id.clone().into(),
        time_display: format_time(&f.started_at).into(),
        method_display: f.method.display().into(),
        method_icon: f.method.icon().into(),
        duration_display: f.duration_display().into(),
        amount: f.amount.unwrap_or(0.0) as f32,
        notes: f.notes.clone().unwrap_or_default().into(),
    }
}

fn to_slint_diaper(d: &crate::db::models::Diaper) -> DiaperData {
    DiaperData {
        id: d.id.clone().into(),
        baby_id: d.baby_id.clone().into(),
        time_display: format_time(&d.time).into(),
        type_display: d.type_display().into(),
        icon: d.icon().into(),
        color: d.color.clone().unwrap_or_default().into(),
        notes: d.notes.clone().unwrap_or_default().into(),
    }
}

fn to_slint_weight(w: &crate::db::models::Weight) -> WeightData {
    WeightData {
        id: w.id.clone().into(),
        date_display: format_date(&w.date_).into(),
        value: w.weight as f32,
        value_display: format!("{:.2}", w.weight).into(),
    }
}

fn to_slint_height(h: &crate::db::models::Height) -> HeightData {
    HeightData {
        id: h.id.clone().into(),
        date_display: format_date(&h.date_).into(),
        value: h.height as f32,
        value_display: format!("{:.1}", h.height).into(),
    }
}

fn to_slint_hc(h: &crate::db::models::HeadCircumference) -> HeadCircumferenceData {
    HeadCircumferenceData {
        id: h.id.clone().into(),
        date_display: format_date(&h.date_).into(),
        value: h.head_circumference as f32,
        value_display: format!("{:.1}", h.head_circumference).into(),
    }
}

fn to_slint_tooth(t: &crate::db::models::Tooth) -> ToothData {
    ToothData {
        id: t.id.clone().into(),
        date_display: format_date(&t.date_).into(),
        tooth_id: t.tooth_id,
        name: t.name().into(),
    }
}

fn to_slint_medication(m: &crate::db::models::Medication) -> MedicationData {
    MedicationData {
        id: m.id.clone().into(),
        time_display: format_time(&m.time).into(),
        name: m.name.clone().into(),
        dosage_display: m.dosage_display().into(),
        notes: m.notes.clone().unwrap_or_default().into(),
    }
}

fn to_slint_summary(s: &DaySummary) -> SummaryData {
    SummaryData {
        feedings_today: s.feedings_today as i32,
        diapers_today: s.diapers_today as i32,
        last_feeding: s.last_feeding.as_ref().map(format_relative).unwrap_or_else(|| "—".into()).into(),
        last_diaper: s.last_diaper.as_ref().map(format_relative).unwrap_or_else(|| "—".into()).into(),
        latest_weight: s.latest_weight.map(|w| format!("{:.2}", w)).unwrap_or_default().into(),
        latest_height: s.latest_height.map(|h| format!("{:.1}", h)).unwrap_or_default().into(),
    }
}

// ─── Refresh all data ─────────────────────────────────────────────────────────

pub fn refresh_all(ui: &AppWindow, db: &Database, selected_idx: usize) {
    let babies = db.get_babies().unwrap_or_default();
    let babies_vec: Vec<BabyData> = babies.iter().map(to_slint_baby).collect();
    let babies_model: Rc<VecModel<BabyData>> = Rc::new(VecModel::from(babies_vec));
    ui.set_babies(babies_model.into());

    if babies.is_empty() {
        return;
    }

    let baby = &babies[selected_idx.min(babies.len() - 1)];
    let baby_id = &baby.id;

    let summary = db.get_day_summary(baby_id, &baby.name).unwrap_or_default();
    ui.set_summary(to_slint_summary(&summary));

    let feedings: Vec<FeedingData> = db.get_feedings(baby_id, 50).unwrap_or_default().iter()
        .map(to_slint_feeding).collect();
    ui.set_feedings(Rc::new(VecModel::from(feedings)).into());

    let diapers: Vec<DiaperData> = db.get_diapers(baby_id, 50).unwrap_or_default().iter()
        .map(to_slint_diaper).collect();
    ui.set_diapers(Rc::new(VecModel::from(diapers)).into());

    let weights: Vec<WeightData> = db.get_weights(baby_id).unwrap_or_default().iter()
        .map(to_slint_weight).collect();
    ui.set_weights(Rc::new(VecModel::from(weights)).into());

    let heights: Vec<HeightData> = db.get_heights(baby_id).unwrap_or_default().iter()
        .map(to_slint_height).collect();
    ui.set_heights(Rc::new(VecModel::from(heights)).into());

    let hcs: Vec<HeadCircumferenceData> = db.get_head_circumferences(baby_id).unwrap_or_default().iter()
        .map(to_slint_hc).collect();
    ui.set_hcs(Rc::new(VecModel::from(hcs)).into());

    let teeth: Vec<ToothData> = db.get_teeth(baby_id).unwrap_or_default().iter()
        .map(to_slint_tooth).collect();
    ui.set_teeth(Rc::new(VecModel::from(teeth)).into());

    let meds: Vec<MedicationData> = db.get_medications(baby_id, 100).unwrap_or_default().iter()
        .map(to_slint_medication).collect();
    ui.set_medications(Rc::new(VecModel::from(meds)).into());
}

// ─── Wire all callbacks ───────────────────────────────────────────────────────

pub fn wire_callbacks(ui: &AppWindow, database: Arc<Mutex<Database>>) {
    let ui_weak = ui.as_weak();

    // refresh-data
    {
        let db = database.clone(); let ui_w = ui_weak.clone();
        ui.on_refresh_data(move || {
            if let Some(ui) = ui_w.upgrade() {
                let db = db.lock().unwrap();
                let idx = ui.get_selected_baby_idx() as usize;
                refresh_all(&ui, &db, idx);
            }
        });
    }

    // baby-selected
    {
        let db = database.clone(); let ui_w = ui_weak.clone();
        ui.on_baby_selected(move |idx| {
            if let Some(ui) = ui_w.upgrade() {
                let db = db.lock().unwrap();
                refresh_all(&ui, &db, idx as usize);
            }
        });
    }

    // save-feeding
    {
        let db = database.clone(); let ui_w = ui_weak.clone();
        ui.on_save_feeding(move |baby_id, method, type_, duration_min, amount, notes| {
            if let Some(ui) = ui_w.upgrade() {
                let db = db.lock().unwrap();
                let now = Utc::now();
                let method_enum = FeedingMethod::from_str(&method);
                let ended = if duration_min > 0.0 {
                    Some(now + chrono::Duration::minutes(duration_min as i64))
                } else { None };
                let amt = if amount > 0.0 { Some(amount as f64) } else { None };
                let notes_opt = if notes.is_empty() { None } else { Some(notes.as_str()) };
                if let Err(e) = db.create_feeding(&baby_id, now, ended, &method_enum, &type_, amt, notes_opt) {
                    error!("save_feeding: {}", e);
                }
                refresh_all(&ui, &db, ui.get_selected_baby_idx() as usize);
            }
        });
    }

    // save-diaper
    {
        let db = database.clone(); let ui_w = ui_weak.clone();
        ui.on_save_diaper(move |baby_id, wet, solid, color, amount, notes| {
            if let Some(ui) = ui_w.upgrade() {
                let db = db.lock().unwrap();
                let c = if color.is_empty() { None } else { Some(color.as_str()) };
                let a = if amount.is_empty() { None } else { Some(amount.as_str()) };
                let n = if notes.is_empty() { None } else { Some(notes.as_str()) };
                if let Err(e) = db.create_diaper(&baby_id, Utc::now(), wet, solid, c, a, n) {
                    error!("save_diaper: {}", e);
                }
                refresh_all(&ui, &db, ui.get_selected_baby_idx() as usize);
            }
        });
    }

    // save-weight
    {
        let db = database.clone(); let ui_w = ui_weak.clone();
        ui.on_save_weight(move |value, date_iso, notes| {
            if let Some(ui) = ui_w.upgrade() {
                let db = db.lock().unwrap();
                let babies = db.get_babies().unwrap_or_default();
                if let Some(baby) = babies.get(ui.get_selected_baby_idx() as usize) {
                    let date = NaiveDate::parse_from_str(&date_iso, "%Y-%m-%d")
                        .unwrap_or_else(|_| Utc::now().date_naive());
                    let n = if notes.is_empty() { None } else { Some(notes.as_str()) };
                    if let Err(e) = db.create_weight(&baby.id, date, value as f64, n) {
                        error!("save_weight: {}", e);
                    }
                    refresh_all(&ui, &db, ui.get_selected_baby_idx() as usize);
                }
            }
        });
    }

    // save-height
    {
        let db = database.clone(); let ui_w = ui_weak.clone();
        ui.on_save_height(move |value, date_iso, notes| {
            if let Some(ui) = ui_w.upgrade() {
                let db = db.lock().unwrap();
                let babies = db.get_babies().unwrap_or_default();
                if let Some(baby) = babies.get(ui.get_selected_baby_idx() as usize) {
                    let date = NaiveDate::parse_from_str(&date_iso, "%Y-%m-%d")
                        .unwrap_or_else(|_| Utc::now().date_naive());
                    let n = if notes.is_empty() { None } else { Some(notes.as_str()) };
                    if let Err(e) = db.create_height(&baby.id, date, value as f64, n) {
                        error!("save_height: {}", e);
                    }
                    refresh_all(&ui, &db, ui.get_selected_baby_idx() as usize);
                }
            }
        });
    }

    // save-hc
    {
        let db = database.clone(); let ui_w = ui_weak.clone();
        ui.on_save_hc(move |value, date_iso, notes| {
            if let Some(ui) = ui_w.upgrade() {
                let db = db.lock().unwrap();
                let babies = db.get_babies().unwrap_or_default();
                if let Some(baby) = babies.get(ui.get_selected_baby_idx() as usize) {
                    let date = NaiveDate::parse_from_str(&date_iso, "%Y-%m-%d")
                        .unwrap_or_else(|_| Utc::now().date_naive());
                    let n = if notes.is_empty() { None } else { Some(notes.as_str()) };
                    if let Err(e) = db.create_head_circumference(&baby.id, date, value as f64, n) {
                        error!("save_hc: {}", e);
                    }
                    refresh_all(&ui, &db, ui.get_selected_baby_idx() as usize);
                }
            }
        });
    }

    // save-medication
    {
        let db = database.clone(); let ui_w = ui_weak.clone();
        ui.on_save_medication(move |name, dosage, unit, _time_iso, notes| {
            if let Some(ui) = ui_w.upgrade() {
                let db = db.lock().unwrap();
                let babies = db.get_babies().unwrap_or_default();
                if let Some(baby) = babies.get(ui.get_selected_baby_idx() as usize) {
                    let d = if dosage > 0.0 { Some(dosage as f64) } else { None };
                    let u = if unit.is_empty() { None } else { Some(unit.as_str()) };
                    let n = if notes.is_empty() { None } else { Some(notes.as_str()) };
                    if let Err(e) = db.create_medication(&baby.id, Utc::now(), &name, d, u, n) {
                        error!("save_medication: {}", e);
                    }
                    refresh_all(&ui, &db, ui.get_selected_baby_idx() as usize);
                }
            }
        });
    }

    // save-baby
    {
        let db = database.clone(); let ui_w = ui_weak.clone();
        ui.on_save_baby(move |name, birth_date_iso| {
            if let Some(ui) = ui_w.upgrade() {
                let db = db.lock().unwrap();
                let date = NaiveDate::parse_from_str(&birth_date_iso, "%Y-%m-%d")
                    .unwrap_or_else(|_| Utc::now().date_naive());
                if let Err(e) = db.create_baby(&name, date, None) {
                    error!("save_baby: {}", e);
                }
                refresh_all(&ui, &db, 0);
            }
        });
    }

    // save-tooth
    {
        let db = database.clone(); let ui_w = ui_weak.clone();
        ui.on_save_tooth(move |tooth_id, date_iso, notes| {
            if let Some(ui) = ui_w.upgrade() {
                let db = db.lock().unwrap();
                let babies = db.get_babies().unwrap_or_default();
                if let Some(baby) = babies.get(ui.get_selected_baby_idx() as usize) {
                    let date = NaiveDate::parse_from_str(&date_iso, "%Y-%m-%d")
                        .unwrap_or_else(|_| Utc::now().date_naive());
                    let n = if notes.is_empty() { None } else { Some(notes.as_str()) };
                    if let Err(e) = db.create_tooth(&baby.id, date, tooth_id, n) {
                        error!("save_tooth: {}", e);
                    }
                    refresh_all(&ui, &db, ui.get_selected_baby_idx() as usize);
                }
            }
        });
    }

    // Delete callbacks (macro)
    macro_rules! del_cb {
        ($method:ident, $on_cb:ident) => {{
            let db = database.clone(); let ui_w = ui_weak.clone();
            ui.$on_cb(move |id| {
                if let Some(ui) = ui_w.upgrade() {
                    let db = db.lock().unwrap();
                    if let Err(e) = db.$method(&id) { error!("delete: {}", e); }
                    refresh_all(&ui, &db, ui.get_selected_baby_idx() as usize);
                }
            });
        }};
    }

    del_cb!(delete_feeding,           on_delete_feeding);
    del_cb!(delete_diaper,            on_delete_diaper);
    del_cb!(delete_weight,            on_delete_weight);
    del_cb!(delete_height,            on_delete_height);
    del_cb!(delete_head_circumference, on_delete_hc);
    del_cb!(delete_medication,        on_delete_medication);
    del_cb!(delete_baby,              on_delete_baby);
    del_cb!(delete_tooth,             on_delete_tooth);
}

// ─── Desktop run ─────────────────────────────────────────────────────────────

pub fn run_app() {
    let path = db_path_local();
    let database = match Database::open(&path) {
        Ok(db) => Arc::new(Mutex::new(db)),
        Err(e) => { eprintln!("DB error: {}", e); return; }
    };

    let ui = AppWindow::new().expect("AppWindow::new failed");
    {
        let db = database.lock().unwrap();
        refresh_all(&ui, &db, 0);
    }
    wire_callbacks(&ui, database);
    ui.run().expect("Slint run failed");
}

// ─── Android run ─────────────────────────────────────────────────────────────

#[cfg(feature = "android")]
pub fn run_app_android(app: android_activity::AndroidApp) {
    slint::android::init(app.clone()).expect("Slint Android init failed");

    let path = db_path_android(&app);
    let database = match Database::open(&path) {
        Ok(db) => Arc::new(Mutex::new(db)),
        Err(e) => { log::error!("DB error: {}", e); return; }
    };

    let ui = AppWindow::new().expect("AppWindow::new failed");
    {
        let db = database.lock().unwrap();
        refresh_all(&ui, &db, 0);
    }
    wire_callbacks(&ui, database);
    ui.run().expect("Slint run failed");
}
