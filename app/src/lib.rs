// BabyBuddy — entry point
// Conditional compilation: `android` feature -> android_main
//                          `desktop` feature -> re-exported for main.rs

#![allow(unused_imports)]

pub mod db;
pub mod ui;
pub mod app;

// Re-export for desktop binary
pub use app::run_app;

// Android entry point — uses slint's re-export of AndroidApp to guarantee
// the exact same type instance that slint::android::init() expects.
#[cfg(feature = "android")]
mod android_entry {
    use slint::android::AndroidApp;
    use log::info;

    #[no_mangle]
    fn android_main(app: AndroidApp) {
        android_logger::init_once(
            android_logger::Config::default()
                .with_max_level(log::LevelFilter::Info)
                .with_tag("BabyBuddy"),
        );
        info!("BabyBuddy starting (Android)");
        crate::app::run_app_android(app);
    }
}
