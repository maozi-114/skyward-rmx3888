#!/system/bin/sh
# Read-only port diagnostics. Run INSIDE TWRP, never by flashing this file.
# Does not mount, insmod, setprop, stop services, read credentials or write partitions.
# Writes output only to the report directory given as argument.
if [ "$(id -u)" != 0 ]; then echo 'Root required'; exit 1; fi
if [ "$(getprop ro.twrp.boot)" != 1 ] && [ "$(getprop ro.twrp.boot)" != true ] && [ ! -d /twres ]; then
    echo 'Refusing: TWRP recovery environment not detected.'; exit 2
fi
OUT="${1:-/tmp/skyward-rmx3888-report}"
mkdir -p "$OUT" || exit 3
{
    echo 'RMX3888 read-only recovery probe v1'
    uname -a
    for p in ro.product.model ro.product.device ro.boot.prjname ro.boot.slot_suffix ro.twrp.version ro.twrp.boot ro.build.version.release ro.build.version.security_patch twrp.modules.loaded twrp.variant.files_copied crypto.ready ro.crypto.state ro.crypto.type; do
        printf '%s=' "$p"; getprop "$p"
    done
    echo '=== confirmed touch driver ==='
    readlink -f /sys/bus/spi/devices/spi0.0/driver/module
    for p in /proc/devinfo/tp /proc/devinfo/lcd; do [ ! -f "$p" ] || cat "$p"; done
    echo '=== firmware presence ==='
    for p in /odm/firmware/tp/23607/FW_S3910_BOE.img /vendor/firmware/tp/23607/FW_S3910_BOE.img /vendor/bin/hw/wpa_supplicant /vendor/bin/hw/android.hardware.weaver-service.nxp /vendor/bin/se_omapi; do ls -ld "$p" 2>/dev/null; done
    echo '=== init service states ==='
    getprop | grep '^\[init.svc\.'
    echo '=== module sources ==='
    for p in /lib/modules /vendor/lib/modules /vendor_dlkm/lib/modules /system_dlkm/lib/modules /system_dlkm/flatten/lib/modules; do ls -ld "$p" 2>/dev/null; done
} > "$OUT/environment.txt" 2>&1
cat /proc/mounts > "$OUT/mounts.txt"
cat /proc/modules > "$OUT/modules.txt"
cat /proc/bus/input/devices > "$OUT/input.txt"
{
    ls -l /dev/dri /dev/graphics 2>/dev/null
    for p in /sys/class/drm/*/status /sys/class/drm/*/modes; do
        [ ! -f "$p" ] || { echo "===$p"; cat "$p"; }
    done
} > "$OUT/display.txt" 2>&1
# Query-only inspection: no binding or restarting of hardware services.
if command -v service >/dev/null 2>&1; then service list > "$OUT/binder-services.txt" 2>&1; fi
printf 'Reports saved in %s\nReview before sharing: device/input identifiers may be present.\n' "$OUT"
