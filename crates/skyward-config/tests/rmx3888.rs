use skyward_config::DeviceConfig;

#[test]
fn rmx3888_observed_configuration_is_valid_but_not_stock_ready() {
    let cfg = DeviceConfig::parse(include_str!("../../../devices/rmx3888.toml")).unwrap();
    assert_eq!(cfg.device.codename, "rmx3888");
    assert_eq!((cfg.display.width, cfg.display.height, cfg.display.density), (1264, 2780, 560));
    assert!(cfg.device.ab);
    assert_eq!(cfg.modules.load, ["oplus_bsp_tp_tcm_S3910"]);
    assert!(cfg.stock.is_none());
    assert!(cfg.update.is_none());
    assert!(cfg.wlan.is_none());
    assert!(cfg.services.is_empty());
}
