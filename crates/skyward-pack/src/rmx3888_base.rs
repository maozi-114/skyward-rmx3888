//! Explicit conversion of one audited RMX3888 TWRP base to standalone Skyward.
//! Experimental: rebuilding is not evidence of hardware or decrypt compatibility.
use anyhow::Result;
#[cfg(test)]
use anyhow::ensure;
use skyward_bootimg::Ramdisk;
pub const BASE_SHA256: &str = "cc34cd71ce185f77b6c996fd33df83ddb7db7d0c24372953e90a09099cc3f492";

// Remove action blocks whose trigger is supplied by TWRP's C++ recovery.
// Do not synthesize TWRP properties or execute prepdecrypt/variant scripts.
#[cfg(test)]
fn remove_twrp_actions(text: &str) -> String {
    let mut skip = false;
    let mut out = String::new();
    for line in text.lines() {
        let top = !line.starts_with(char::is_whitespace) && !line.is_empty() && !line.starts_with('#');
        if top { skip = line.starts_with("on ") && line.contains("twrp."); }
        if !skip { out.push_str(line); out.push('\n'); }
    }
    out
}
pub fn convert(_rd: &mut Ramdisk, _codename: &str) -> Result<()> {
    anyhow::bail!("RMX3888 conversion is blocked: firmware/ADSP/health and secure-service lifecycle migration is incomplete; no image may be emitted")
}
// Draft kept only for transformation unit tests; never callable by production packing.
#[cfg(test)]
fn draft_convert(rd: &mut Ramdisk, codename: &str) -> Result<()> {
    ensure!(codename == "rmx3888", "audited custom-base conversion is only for rmx3888");
    ensure!(rd.find("twres").is_some(), "expected the audited TWRP tree");
    ensure!(rd.find("system/bin/recovery").is_some(), "missing recovery program");
    // The host replaces system/bin/recovery immediately after conversion.
    // Keep Android init, firmware files and libraries; remove TWRP UI and
    // helper scripts so no TWRP variant/prepdecrypt path can be invoked.
    rd.entries.retain(|e| !(e.name == "twres" || e.name.starts_with("twres/") || matches!(e.name.as_str(), "system/bin/twrp" | "system/bin/variant-script.sh" | "vendor/bin/prepdecrypt.sh" | "vendor/etc/init/variant-script.rc" | "vendor/etc/init/prepdecrypt.rc" | "system/etc/twrp.flags")));
    for e in rd.entries.iter_mut().filter(|e| e.name.ends_with(".rc")) {
        let text = std::str::from_utf8(&e.data)?;
        let text = remove_twrp_actions(text);
        // The variant controller is gone; remove its reset action too.
        let text = text.lines().filter(|l| !l.contains("setprop twrp.")).collect::<Vec<_>>().join("\n") + "\n";
        e.data = text.into_bytes();
    }
    rd.add("skyward/EXPERIMENTAL.txt", 0o644, b"Standalone Skyward using an audited RMX3888 recovery base. TWRP entry and resources removed. Hardware, FBE, bootloader acceptance and rollback are NOT validated. DO NOT FLASH without separate verification.\n".to_vec())?;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn incomplete_port_cannot_emit_an_image() {
        let mut rd = Ramdisk { entries: vec![] };
        assert!(convert(&mut rd, "rmx3888").is_err());
        assert!(rd.entries.is_empty());
    }
    #[test]
    fn removes_twrp_triggers_but_keeps_android_init_actions() {
        let text = "on fs\n    install_keyring\non property:twrp.modules.loaded=true\n    start variant-script\non property:twrp.variant.files_copied=1\n    start prepdecrypt.vendor\non property:servicemanager.ready=true\n    start vendor.ssgtzd\n";
        let out = remove_twrp_actions(text);
        assert!(!out.contains("variant-script")); assert!(!out.contains("prepdecrypt"));
        assert!(out.contains("install_keyring")); assert!(out.contains("start vendor.ssgtzd"));
    }
    #[test]
    fn conversion_removes_ui_without_removing_recovery_entry_before_replacement() {
        let mut rd = Ramdisk { entries: vec![] };
        rd.add("twres/ui.xml", 0o644, vec![]).unwrap();
        rd.add("system/bin/recovery", 0o755, vec![1]).unwrap();
        rd.add("init.recovery.qcom.rc", 0o644, b"on fs\n    install_keyring\n".to_vec()).unwrap();
        draft_convert(&mut rd, "rmx3888").unwrap();
        assert!(rd.find("twres").is_none()); assert!(rd.find("twres/ui.xml").is_none());
        assert!(rd.find("system/bin/recovery").is_some());
    }
}