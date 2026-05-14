use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DeclKind {
    Function,
    Object,
}

#[derive(Clone, Debug)]
struct Decl {
    kind: DeclKind,
    text: String,
}

#[derive(Clone, Debug)]
struct ManifestSymbol {
    section: String,
    original_name: String,
    normalized_name: String,
}

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("manifest dir"));
    let workspace_root = manifest_dir
        .parent()
        .expect("emu-abi crate should live under workspace root")
        .to_path_buf();
    let bindings_path = workspace_root.join("why2025-badge-sys-bindings/src/bindings.rs");
    let symbols_path =
        workspace_root.join("why2025-badge-sys-bindings/firmware/badgevms/symbols.yml");
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR"));

    println!("cargo:rerun-if-changed={}", bindings_path.display());
    println!("cargo:rerun-if-changed={}", symbols_path.display());
    for source in rust_sources(&manifest_dir.join("src")) {
        println!("cargo:rerun-if-changed={}", source.display());
    }

    let bindings = parse_bindings(&bindings_path);
    let manifest_symbols = parse_manifest(&symbols_path);
    let mut manual_exports = parse_local_exports(&manifest_dir.join("src"));
    manual_exports.extend(
        macro_generated_exports()
            .iter()
            .map(|symbol| (*symbol).to_string()),
    );

    generate_stubs(
        &out_dir.join("generated_stubs.rs"),
        &manifest_symbols,
        &bindings,
        &manual_exports,
    );
    generate_linker_test(
        &out_dir.join("generated_linker_test.rs"),
        &manifest_symbols,
        &bindings,
    );
}

fn macro_generated_exports() -> &'static [&'static str] {
    &[
        "window_create",
        "window_framebuffer_create",
        "window_destroy",
        "window_title_get",
        "window_title_set",
        "window_position_get",
        "window_position_set",
        "window_size_get",
        "window_size_set",
        "window_flags_get",
        "window_flags_set",
        "window_framebuffer_size_get",
        "window_framebuffer_size_set",
        "window_framebuffer_format_get",
        "window_framebuffer_get",
        "window_present",
        "window_event_poll",
        "get_screen_info",
        "wifi_get_status",
        "wifi_get_connection_status",
        "wifi_get_connection_station",
        "wifi_connect",
        "wifi_disconnect",
        "wifi_scan_free_station",
        "wifi_scan_get_num_results",
        "wifi_scan_get_result",
        "wifi_station_get_ssid",
        "wifi_station_get_bssid",
        "wifi_station_get_primary_channel",
        "wifi_station_get_secondary_channel",
        "wifi_station_get_rssi",
        "wifi_station_get_mode",
        "wifi_station_wps",
        "wifi_set_connection_parameters",
        "curl_easy_init",
        "curl_easy_perform",
        "curl_easy_cleanup",
        "curl_easy_strerror",
        "curl_slist_append",
        "curl_slist_free_all",
        "curl_global_init",
        "curl_global_cleanup",
        "inet_ntoa",
        "inet_aton",
        "accept",
        "bind",
        "connect",
        "listen",
        "socket",
        "freeaddrinfo",
        "getaddrinfo",
        "_Exit",
        "_exit",
        "abort",
        "exit",
    ]
}

fn rust_sources(root: &Path) -> Vec<PathBuf> {
    let mut sources = Vec::new();
    collect_rust_sources(root, &mut sources);
    sources.sort();
    sources
}

fn collect_rust_sources(root: &Path, sources: &mut Vec<PathBuf>) {
    let entries =
        fs::read_dir(root).unwrap_or_else(|err| panic!("failed to read {}: {err}", root.display()));
    for entry in entries {
        let entry = entry.expect("dir entry");
        let path = entry.path();
        if path.is_dir() {
            collect_rust_sources(&path, sources);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            sources.push(path);
        }
    }
}

fn parse_bindings(path: &Path) -> BTreeMap<String, Decl> {
    let contents = fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
    let mut declarations = BTreeMap::new();
    let mut in_extern_block = false;
    let mut current_decl = String::new();

    for raw_line in contents.lines() {
        let trimmed = raw_line.trim();

        if trimmed == "unsafe extern \"C\" {" {
            in_extern_block = true;
            continue;
        }
        if !in_extern_block {
            continue;
        }
        if trimmed == "}" {
            break;
        }

        if current_decl.is_empty() {
            if trimmed.starts_with("pub fn ") || trimmed.starts_with("pub static ") {
                current_decl.push_str(trimmed);
                if trimmed.ends_with(';') {
                    store_decl(&mut declarations, &current_decl);
                    current_decl.clear();
                }
            }
            continue;
        }

        current_decl.push('\n');
        current_decl.push_str(trimmed);
        if trimmed.ends_with(';') {
            store_decl(&mut declarations, &current_decl);
            current_decl.clear();
        }
    }

    declarations
}

fn store_decl(declarations: &mut BTreeMap<String, Decl>, declaration: &str) {
    let declaration = declaration.trim().to_string();
    if let Some(remainder) = declaration.strip_prefix("pub fn ") {
        let name = remainder
            .split('(')
            .next()
            .expect("function name")
            .trim()
            .to_string();
        declarations.insert(
            name,
            Decl {
                kind: DeclKind::Function,
                text: declaration,
            },
        );
        return;
    }

    if let Some(remainder) = declaration.strip_prefix("pub static ") {
        let remainder = remainder.strip_prefix("mut ").unwrap_or(remainder);
        let name = remainder
            .split(':')
            .next()
            .expect("object name")
            .trim()
            .to_string();
        declarations.insert(
            name,
            Decl {
                kind: DeclKind::Object,
                text: declaration,
            },
        );
    }
}

fn parse_manifest(path: &Path) -> Vec<ManifestSymbol> {
    let contents = fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
    let mut current_section: Option<String> = None;
    let mut symbols = Vec::new();
    let tracked_sections = [
        "simple_function",
        "simple_function_extern",
        "wrapped_function",
        "simple_object",
        "wrapped_object",
    ];

    for raw_line in contents.lines() {
        let trimmed = raw_line.trim();
        if trimmed.is_empty() || trimmed == "---" || trimmed.starts_with('#') {
            continue;
        }

        if let Some(section) = trimmed.strip_suffix(':') {
            current_section = tracked_sections
                .contains(&section)
                .then(|| section.to_string());
            continue;
        }

        let Some(section) = current_section.as_ref() else {
            continue;
        };

        let Some(symbol) = trimmed.strip_prefix("- ") else {
            continue;
        };

        let original_name = symbol.trim().to_string();
        let normalized_name = if original_name == "_ctype_" {
            "_ctype_b".to_string()
        } else {
            original_name.clone()
        };

        symbols.push(ManifestSymbol {
            section: section.clone(),
            original_name,
            normalized_name,
        });
    }

    let mut deduped = Vec::new();
    let mut seen = BTreeSet::new();
    for symbol in symbols {
        if seen.insert(symbol.normalized_name.clone()) {
            deduped.push(symbol);
        }
    }
    deduped
}

fn parse_local_exports(src_dir: &Path) -> BTreeSet<String> {
    let mut exports = BTreeSet::new();

    for source in rust_sources(src_dir) {
        let contents = fs::read_to_string(&source)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", source.display()));
        let mut saw_no_mangle = false;
        let mut export_name: Option<String> = None;
        let mut macro_block_depth = 0usize;

        for raw_line in contents.lines() {
            let trimmed = raw_line.trim();

            if macro_block_depth == 0
                && trimmed.contains("! {")
                && !trimmed.starts_with("macro_rules!")
            {
                macro_block_depth = brace_delta(trimmed);
                if let Some(symbol) = extract_macro_decl_name(trimmed) {
                    exports.insert(symbol);
                }
                continue;
            }

            if macro_block_depth != 0 {
                if let Some(symbol) = extract_macro_decl_name(trimmed) {
                    exports.insert(symbol);
                }
                macro_block_depth = macro_block_depth
                    .saturating_add(trimmed.matches('{').count())
                    .saturating_sub(trimmed.matches('}').count());
                continue;
            }

            if trimmed.starts_with("#[unsafe(no_mangle)]") {
                saw_no_mangle = true;
                continue;
            }

            if let Some(name) = trimmed
                .strip_prefix("#[unsafe(export_name = \"")
                .and_then(|rest| rest.strip_suffix("\")]"))
            {
                export_name = Some(name.to_string());
                continue;
            }

            if trimmed.starts_with("#") || trimmed.starts_with("//") || trimmed.is_empty() {
                continue;
            }

            if !(saw_no_mangle || export_name.is_some()) {
                continue;
            }

            if let Some(symbol) = extract_exported_item_name(trimmed) {
                exports.insert(export_name.take().unwrap_or(symbol));
            }

            saw_no_mangle = false;
            export_name = None;
        }
    }

    exports
}

fn brace_delta(line: &str) -> usize {
    line.matches('{')
        .count()
        .saturating_sub(line.matches('}').count())
}

fn extract_macro_decl_name(line: &str) -> Option<String> {
    let remainder = line.strip_prefix("fn ")?;
    let name = remainder.split('(').next()?.trim();
    (!name.is_empty()).then(|| name.to_string())
}

fn extract_exported_item_name(line: &str) -> Option<String> {
    for prefix in [
        "pub unsafe extern \"C\" fn ",
        "pub extern \"C\" fn ",
        "pub static mut ",
        "pub static ",
    ] {
        if let Some(remainder) = line.strip_prefix(prefix) {
            let name = if prefix.contains(" fn ") {
                remainder.split('(').next()?
            } else {
                remainder.split(':').next()?
            };
            return Some(name.trim().to_string());
        }
    }

    None
}

fn family_for(symbol: &str, section: &str) -> &'static str {
    if symbol.starts_with("window_") || symbol == "get_screen_info" {
        return "graphics";
    }
    if symbol.starts_with("wifi_") {
        return "wifi";
    }
    if symbol.starts_with("curl_") {
        return "networking";
    }
    if matches!(
        symbol,
        "accept"
            | "bind"
            | "connect"
            | "freeaddrinfo"
            | "getaddrinfo"
            | "inet_aton"
            | "inet_ntoa"
            | "listen"
            | "socket"
    ) {
        return "networking";
    }
    if symbol.starts_with("application_") {
        return "application";
    }
    if symbol.starts_with("ota_") {
        return "ota";
    }
    if symbol.starts_with("process_")
        || symbol.starts_with("thread_")
        || symbol.starts_with("task_")
        || matches!(
            symbol,
            "wait" | "get_num_tasks" | "device_get" | "die" | "vaddr_to_paddr" | "get_mac_address"
        )
    {
        return "badgevms";
    }
    if symbol.starts_with("__riscv_save_") || symbol.starts_with("__riscv_restore_") {
        return "riscv-millicode";
    }
    if matches!(symbol, "regcomp" | "regerror" | "regexec" | "regfree") {
        return "regex";
    }
    if matches!(symbol, "setjmp" | "longjmp") {
        return "setjmp";
    }
    if symbol.starts_with("__") {
        return "compiler-rt";
    }
    match section {
        "wrapped_function" => "wrapped-libc",
        "wrapped_object" => "wrapped-libc-object",
        _ => "generated-stub",
    }
}

fn generate_stubs(
    out_path: &Path,
    manifest_symbols: &[ManifestSymbol],
    bindings: &BTreeMap<String, Decl>,
    manual_exports: &BTreeSet<String>,
) {
    let mut generated = String::new();
    generated.push_str("use crate::types::*;\n\n");

    for symbol in manifest_symbols {
        if manual_exports.contains(&symbol.normalized_name) {
            continue;
        }

        let decl = bindings.get(&symbol.normalized_name).unwrap_or_else(|| {
            panic!(
                "missing binding declaration for manifest symbol {}",
                symbol.original_name
            )
        });

        match decl.kind {
            DeclKind::Function => {
                let signature = decl
                    .text
                    .strip_prefix("pub fn ")
                    .expect("function declaration")
                    .trim_end_matches(';')
                    .replace("...", "mut _args: ...");
                generated.push_str("#[allow(unused_variables)]\n");
                generated.push_str("#[unsafe(no_mangle)]\n");
                generated.push_str("pub unsafe extern \"C\" fn ");
                generated.push_str(&signature);
                generated.push_str(" {\n");
                generated.push_str("    crate::runtime::abort_unimplemented_symbol(");
                generated.push_str(&format!(
                    "{:?}, {:?}",
                    symbol.original_name,
                    family_for(&symbol.original_name, &symbol.section)
                ));
                generated.push_str(")\n}\n\n");
            }
            DeclKind::Object => {
                panic!(
                    "manifest object {} is not implemented manually and cannot be auto-generated safely",
                    symbol.original_name
                );
            }
        }
    }

    fs::write(out_path, generated)
        .unwrap_or_else(|err| panic!("failed to write {}: {err}", out_path.display()));
}

fn generate_linker_test(
    out_path: &Path,
    manifest_symbols: &[ManifestSymbol],
    bindings: &BTreeMap<String, Decl>,
) {
    let mut generated = String::new();
    generated.push_str("use crate::types::*;\n\n");
    generated.push_str("unsafe extern \"C\" {\n");

    for symbol in manifest_symbols {
        let decl = bindings.get(&symbol.normalized_name).unwrap_or_else(|| {
            panic!(
                "missing binding declaration for linker test symbol {}",
                symbol.original_name
            )
        });
        generated.push_str("    ");
        generated.push_str(&decl.text.replace('\n', "\n    "));
        generated.push('\n');
    }
    generated.push_str("}\n\n");
    generated.push_str("#[test]\nfn link_all_manifest_symbols() {\n    unsafe {\n");
    for symbol in manifest_symbols {
        let decl = bindings
            .get(&symbol.normalized_name)
            .expect("binding declaration");
        match decl.kind {
            DeclKind::Function => {
                generated.push_str("        assert_ne!(");
                generated.push_str(&symbol.normalized_name);
                generated.push_str(" as *const (), core::ptr::null());\n");
            }
            DeclKind::Object => {
                generated.push_str("        assert_ne!(core::ptr::addr_of!(");
                generated.push_str(&symbol.normalized_name);
                generated.push_str(") as *const (), core::ptr::null());\n");
            }
        }
    }
    generated.push_str("    }\n}\n");

    fs::write(out_path, generated)
        .unwrap_or_else(|err| panic!("failed to write {}: {err}", out_path.display()));
}
