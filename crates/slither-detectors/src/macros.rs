//! `define_detectors!` macro.
//!
//! Used by each tier's `register()` to push every detector into the global
//! registry with one declarative call. Equivalent to manually writing
//! `out.push(Box::new(X));` per entry but easier to scan for completeness.
//!
//! ```ignore
//! define_detectors!(out, [
//!     Suicidal,
//!     UnprotectedUpgrade,
//!     Rtlo,
//! ]);
//! ```

#[macro_export]
macro_rules! define_detectors {
    ($out:expr, [ $( $det:ident ),* $(,)? ]) => {{
        $( $out.push(Box::new($det) as Box<dyn $crate::Detector + Send + Sync>); )*
    }};
}
