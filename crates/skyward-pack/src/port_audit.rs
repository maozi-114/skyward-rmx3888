//! Read-only compatibility audit. Never builds or flashes an image.
use skyward_bootimg::Ramdisk;

pub fn report(rd: &Ramdisk) -> String {
    let mut out = String::from("Port audit (read-only; not a bootability verdict)\n");
    let custom = rd.entries.iter().any(|e| e.name == "FFiles" || e.name.starts_with("twres"));
    out.push_str(if custom { "base=custom-recovery\n" } else { "base=no-known-custom-marker\n" });
    for path in ["system/bin/recovery", "prop.default", "system/etc/recovery.fstab"] {
        out.push_str(&format!("{path}={}\n", if rd.find(path).is_some() { "present" } else { "missing" }));
    }
    for (needle, warning) in [
        ("twrp.modules.loaded", "module/firmware startup depends on TWRP property"),
        ("twrp.variant.files_copied", "variant/decrypt startup depends on TWRP property"),
        ("prepdecrypt", "version and mount lifecycle needs review before Skyward bring-up"),
        ("se_omapi", "OMAPI service dependency: verify availability for NXP weaver"),
    ] {
        for entry in rd.entries.iter().filter(|e| e.name.ends_with(".rc") || e.name.ends_with(".sh")) {
            let text = String::from_utf8_lossy(&entry.data);
            if text.lines().filter(|l| !l.trim_start().starts_with('#')).any(|l| l.contains(needle)) {
                out.push_str(&format!("BLOCKER {}: {} [{}]\n", entry.name, warning, needle));
            }
        }
    }
    if let (Some(fstab), Some(flags)) = (rd.find("system/etc/recovery.fstab"), rd.find("system/etc/twrp.flags")) {
        let metadata_type = |data: &[u8], twrp: bool| -> Option<String> {
            String::from_utf8_lossy(data).lines().filter(|l| !l.trim_start().starts_with('#')).find_map(|line| {
                let cols: Vec<_> = line.split_whitespace().collect();
                if twrp && cols.first() == Some(&"/metadata") { cols.get(1).map(|s| s.to_string()) }
                else if !twrp && cols.get(1) == Some(&"/metadata") { cols.get(2).map(|s| s.to_string()) }
                else { None }
            })
        };
        if let (Some(a), Some(b)) = (metadata_type(&fstab.data, false), metadata_type(&flags.data, true)) {
            if a != b { out.push_str(&format!("BLOCKER metadata filesystem mismatch: recovery.fstab={a}, twrp.flags={b}\n")); }
        }
    }
    out.push_str("No image written. No custom-base safety checks bypassed.\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn identifies_twrp_startup_and_metadata_conflicts() {
        let mut rd = Ramdisk { entries: vec![] };
        rd.add("twres/marker", 0o644, vec![]).unwrap();
        rd.add("init.recovery.qcom.rc", 0o644, b"on property:twrp.modules.loaded=true\n    start variant-script\non property:twrp.variant.files_copied=1\n    start prepdecrypt.vendor\n    start vendor.se_omapi\n".to_vec()).unwrap();
        rd.add("system/etc/recovery.fstab", 0o644, b"/dev/block/by-name/metadata /metadata f2fs defaults wait\n".to_vec()).unwrap();
        rd.add("system/etc/twrp.flags", 0o644, b"/metadata ext4 /dev/block/by-name/metadata flags=display=metadata\n".to_vec()).unwrap();
        let text = report(&rd);
        for part in ["base=custom-recovery", "twrp.modules.loaded", "twrp.variant.files_copied", "OMAPI", "metadata filesystem mismatch"] { assert!(text.contains(part), "{part}"); }
    }
    #[test]
    fn comments_do_not_trigger_lifecycle_warning() {
        let mut rd = Ramdisk { entries: vec![] };
        rd.add("init.rc", 0o644, b"# twrp.modules.loaded is not used\n".to_vec()).unwrap();
        assert!(!report(&rd).contains("BLOCKER"));
    }
}
