package com.babybuddy.app

import android.app.NativeActivity
import android.os.Bundle

/**
 * MainActivity is a thin wrapper around NativeActivity.
 *
 * All UI and application logic runs inside the Rust library (libbabybuddy.so)
 * via the Slint Android integration. This Kotlin file is intentionally minimal —
 * its sole purpose is to satisfy the Android launcher intent.
 *
 * The `android:hasCode="true"` manifest attribute is set implicitly because
 * we declare a Kotlin activity. The native library is loaded automatically by
 * NativeActivity via `android:value="babybuddy"` in meta-data.
 *
 * For Slint native-activity, the entry point is `android_main` exported from
 * the Rust cdylib.
 */
class MainActivity : NativeActivity() {

    companion object {
        // Load the Rust shared library. The name must match the `name` in
        // Cargo.toml [lib] section and the file libbabybuddy.so.
        init {
            System.loadLibrary("babybuddy")
        }
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        // Nothing to do here — Slint takes over via JNI/NativeActivity protocol.
    }
}
