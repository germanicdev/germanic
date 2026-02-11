//! # Build-Script für GERMANIC
//!
//! Kompiliert FlatBuffer-Schemas zu Rust-Code und fixt den bekannten
//! Cross-Namespace Bug (#5275) durch Post-Processing.
//!
//! ## Der Bug (seit 2019 offen, wird nie gefixt)
//!
//! flatc generiert fehlerhafte relative Pfade wie `super::super::common::`.
//! Diese funktionieren nur, wenn ALLE Namespaces in EINER Datei liegen.
//!
//! ## Unsere Lösung
//!
//! Nach der Codegenerierung ersetzen wir die fehlerhaften Pfade durch
//! korrekte absolute `crate::`-Pfade. Ein simpler String-Replace.
//!
//! Siehe: https://github.com/google/flatbuffers/issues/5275

use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    // =========================================================================
    // KONFIGURATION
    // =========================================================================

    // Relativer Pfad zu den Schemas (von crates/germanic/ aus)
    let schema_dir = Path::new("../../schemas");
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR nicht gesetzt");

    // Schemas in ABHÄNGIGKEITSREIHENFOLGE (Basis zuerst!)
    // KRITISCH: Wenn Schema A von Schema B abhängt, muss B VOR A stehen!
    let schemas = [
        "common/meta.fbs", // Keine Abhängigkeiten - Basis
        "de/praxis.fbs",   // Könnte später meta.fbs referenzieren
    ];

    // =========================================================================
    // SCHRITT 1: flatc-Verfügbarkeit prüfen
    // =========================================================================

    let flatc_version = Command::new("flatc").arg("--version").output();

    match flatc_version {
        Ok(output) => {
            let version = String::from_utf8_lossy(&output.stdout);
            println!("cargo:warning=GERMANIC: Verwende {}", version.trim());
        }
        Err(_) => {
            panic!(
                r#"
╔═══════════════════════════════════════════════════════════════╗
║  FEHLER: flatc nicht gefunden!                                ║
║                                                               ║
║  Bitte installiere den FlatBuffers Compiler:                  ║
║                                                               ║
║  macOS:   brew install flatbuffers                            ║
║  Linux:   apt install flatbuffers-compiler                    ║
║                                                               ║
║  Oder lade von: https://github.com/google/flatbuffers/releases║
╚═══════════════════════════════════════════════════════════════╝
"#
            );
        }
    }

    // =========================================================================
    // SCHRITT 2: FlatBuffers kompilieren
    // =========================================================================
    //
    // WICHTIG: Alle Schemas in EINEM Aufruf kompilieren!
    // Das ist entscheidend für korrekte Namespace-Auflösung.

    let schema_paths: Vec<_> = schemas.iter().map(|s| schema_dir.join(s)).collect();

    let mut cmd = Command::new("flatc");
    cmd.arg("--rust")
        .arg("-o")
        .arg(&out_dir)
        .arg("-I")
        .arg(schema_dir); // Include-Pfad für Cross-Referenzen

    for path in &schema_paths {
        cmd.arg(path);
    }

    println!("cargo:warning=GERMANIC: Führe aus: {:?}", cmd);

    let status = cmd.status().expect("flatc konnte nicht ausgeführt werden");

    if !status.success() {
        panic!(
            r#"
╔═══════════════════════════════════════════════════════════════╗
║  FEHLER: flatc fehlgeschlagen!                                ║
║                                                               ║
║  Prüfe deine Schema-Dateien mit:                              ║
║  flatc --rust -I ../../schemas ../../schemas/de/praxis.fbs    ║
╚═══════════════════════════════════════════════════════════════╝
"#
        );
    }

    // =========================================================================
    // SCHRITT 3: POST-PROCESSING - Cross-Namespace Bug fixen
    // =========================================================================
    //
    // flatc generiert:   super::super::germanic::common::...
    // Wir brauchen:      crate::generated::germanic::common::...
    //
    // Der Fix ist ein simpler String-Replace im generierten Code.

    fix_cross_namespace_pfade(&out_dir);

    // =========================================================================
    // SCHRITT 4: Rebuild-Trigger setzen
    // =========================================================================

    for schema in &schemas {
        println!("cargo:rerun-if-changed=../../schemas/{}", schema);
    }
    println!("cargo:rerun-if-changed=build.rs");

    // Debug-Info: Zeige was generiert wurde
    zeige_generierte_dateien(&out_dir);

    println!(
        "cargo:warning=GERMANIC: FlatBuffer-Code generiert in {}",
        out_dir
    );
}

/// Fixt die fehlerhaften Cross-Namespace-Pfade im generierten Code.
///
/// # Der Bug (#5275)
///
/// flatc's Rust-Codegen generiert `super::super::...`-Pfade für
/// Cross-Namespace-Referenzen. Diese funktionieren nur, wenn ALLE
/// Namespaces in EINER Datei liegen.
///
/// Da wir separate Dateien haben, müssen wir die relativen Pfade
/// durch absolute `crate::`-Pfade ersetzen.
fn fix_cross_namespace_pfade(out_dir: &str) {
    // =========================================================================
    // PFAD-MAPPINGS
    // =========================================================================
    //
    // Format: (was flatc generiert, was wir brauchen)
    //
    // WICHTIG: Diese Mappings müssen angepasst werden, wenn:
    // - Neue Schemas mit Cross-Namespace-Referenzen hinzukommen
    // - Die Modulstruktur in lib.rs geändert wird
    //
    // Die Reihenfolge kann relevant sein - spezifischere Patterns zuerst!

    let mappings = [
        // ─────────────────────────────────────────────────────────────────────
        // praxis.fbs (in crate::generated::praxis) referenziert
        // germanic.common.* aus meta.fbs (in crate::generated::meta)
        // ─────────────────────────────────────────────────────────────────────
        //
        // flatc generiert:    super::super::germanic::common::GermanicMeta
        // Wir brauchen:       crate::generated::meta::germanic::common::GermanicMeta
        //
        (
            "super::super::germanic::common::",
            "crate::generated::meta::germanic::common::",
        ),
        // Fallback für tiefere Namespace-Hierarchien
        (
            "super::super::super::germanic::common::",
            "crate::generated::meta::germanic::common::",
        ),
        // ─────────────────────────────────────────────────────────────────────
        // Falls meta.fbs jemals praxis.fbs referenziert (unwahrscheinlich)
        // ─────────────────────────────────────────────────────────────────────
        (
            "super::super::de::gesundheit::",
            "crate::generated::praxis::de::gesundheit::",
        ),
    ];

    // Finde alle generierten Dateien
    let generierte_dateien = finde_generierte_dateien(out_dir);

    for datei_pfad in generierte_dateien {
        if let Ok(inhalt) = fs::read_to_string(&datei_pfad) {
            let mut gefixt = inhalt.clone();
            let mut aenderungen = 0;

            for (alt, neu) in &mappings {
                if gefixt.contains(*alt) {
                    let anzahl = gefixt.matches(*alt).count();
                    gefixt = gefixt.replace(*alt, neu);
                    aenderungen += anzahl;

                    println!(
                        "cargo:warning=GERMANIC: {} Ersetzungen in {}: {} → {}",
                        anzahl,
                        datei_pfad.file_name().unwrap_or_default().to_string_lossy(),
                        alt,
                        neu
                    );
                }
            }

            // Nur schreiben wenn sich was geändert hat
            if gefixt != inhalt {
                fs::write(&datei_pfad, &gefixt).expect("Konnte gefixte Datei nicht schreiben");

                println!(
                    "cargo:warning=GERMANIC: {} Cross-Namespace-Pfade in {} gefixt",
                    aenderungen,
                    datei_pfad.file_name().unwrap_or_default().to_string_lossy()
                );
            }
        }
    }
}

/// Findet alle *_generated.rs Dateien im OUT_DIR (rekursiv).
fn finde_generierte_dateien(out_dir: &str) -> Vec<std::path::PathBuf> {
    let mut dateien = Vec::new();

    fn rekursiv(pfad: &Path, dateien: &mut Vec<std::path::PathBuf>) {
        if let Ok(eintraege) = fs::read_dir(pfad) {
            for eintrag in eintraege.flatten() {
                let pfad = eintrag.path();
                if pfad.is_dir() {
                    rekursiv(&pfad, dateien);
                } else if pfad
                    .file_name()
                    .is_some_and(|n| n.to_string_lossy().ends_with("_generated.rs"))
                {
                    dateien.push(pfad);
                }
            }
        }
    }

    rekursiv(Path::new(out_dir), &mut dateien);
    dateien
}

/// Zeigt die generierten Dateien für Debug-Zwecke.
fn zeige_generierte_dateien(out_dir: &str) {
    let dateien = finde_generierte_dateien(out_dir);

    if dateien.is_empty() {
        println!("cargo:warning=GERMANIC: Keine *_generated.rs Dateien gefunden!");
    } else {
        println!(
            "cargo:warning=GERMANIC: {} generierte Dateien:",
            dateien.len()
        );
        for datei in &dateien {
            // Relativer Pfad für bessere Lesbarkeit
            let relativ = datei
                .strip_prefix(out_dir)
                .unwrap_or(datei)
                .display();
            println!("cargo:warning=GERMANIC:   - {}", relativ);
        }
    }
}
