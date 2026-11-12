//! Stage Skyward beside TWRP without replacing TWRP's init or recovery entry.
use anyhow::{ensure, Result};
use skyward_bootimg::Ramdisk;
use skyward_config::DeviceConfig;

pub fn stage(rd: &mut Ramdisk, binary: Vec<u8>, system_init: Vec<u8>, config: Vec<u8>, font: Vec<u8>, mono: Vec<u8>, license: Vec<u8>) -> Result<()> {
    let cfg = DeviceConfig::parse(std::str::from_utf8(&config)?)?;
    ensure!(cfg.device.codename == "rmx3888", "this experimental overlay is only for rmx3888");
    ensure!(rd.entries.iter().any(|e| e.name == "twres" || e.name.starts_with("twres/")), "expected a TWRP base");
    ensure!(rd.find("system/bin/recovery").is_some(), "base has no recovery entry");
    ensure!(rd.find("skyward/device.toml").is_none(), "base already contains Skyward");
    // Do NOT add /skyward/device.toml: its presence tells Skyward the root
    // is its own recovery. A staged TWRP root must never satisfy that test.
    for (name, mode, data) in [
        ("skyward-stage/skyward", 0o755, binary),
        ("skyward-stage/system-init", 0o755, system_init),
        ("skyward-stage/device.toml", 0o644, config),
        ("skyward-stage/MiSansVF.ttf", 0o644, font),
        ("skyward-stage/KirkMono-Regular.ttf", 0o644, mono),
        ("skyward-stage/KirkMono-OFL.txt", 0o644, license),
    ] { rd.add(name, mode, data)?; }
    rd.add("skyward-stage/README.txt", 0o644, b"Experimental staging only. TWRP remains the recovery. Do not run Skyward recovery/startup inside TWRP: it changes misc, mounts and services. Independent Skyward startup/decryption is NOT implemented.\n".to_vec())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn preserves_twrp_and_does_not_mark_root_as_skyward() {
        let mut rd = Ramdisk { entries: vec![] };
        rd.add("twres/marker", 0o644, vec![]).unwrap();
        rd.add("system/bin/recovery", 0o755, b"original recovery".to_vec()).unwrap();
        rd.add("init.recovery.qcom.rc", 0o644, b"original init".to_vec()).unwrap();
        stage(&mut rd, vec![1], vec![2], include_bytes!("../../../devices/rmx3888.toml").to_vec(), vec![], vec![], vec![]).unwrap();
        assert_eq!(rd.find("system/bin/recovery").unwrap().data, b"original recovery");
        assert_eq!(rd.find("init.recovery.qcom.rc").unwrap().data, b"original init");
        assert!(rd.find("skyward/device.toml").is_none());
        assert!(rd.find("skyward-stage/skyward").is_some());
    }
}