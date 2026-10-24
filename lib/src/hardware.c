#include <errno.h>
#include <stdio.h>
#include <string.h>

#include "../include/hardware.h"
#include "../include/driver.h"

#define DMI_SYS_VENDOR "/sys/class/dmi/id/sys_vendor"
#define DMI_PRODUCT_NAME "/sys/class/dmi/id/product_name"
#define DMI_BOARD_NAME "/sys/class/dmi/id/board_name"
#define DMI_BOARD_VENDOR "/sys/class/dmi/id/board_vendor"
#define DMI_CHASSIS_VENDOR "/sys/class/dmi/id/chassis_vendor"

static int read_dmi_string(
    const char *path,
    char *buffer,
    size_t size)
{
    FILE *file;
    size_t length;

    if (!path || !buffer || size < 2)
        return -EINVAL;

    buffer[0] = '\0';

    file = fopen(path, "r");
    if (!file)
        return -errno;

    if (!fgets(buffer, size, file)) {
        int error = ferror(file) ? errno : ENODATA;
        fclose(file);
        return -error;
    }

    fclose(file);

    length = strlen(buffer);

    while (length > 0 &&
           (buffer[length - 1] == '\n' ||
            buffer[length - 1] == '\r' ||
            buffer[length - 1] == ' ' ||
            buffer[length - 1] == '\t')) {
        buffer[--length] = '\0';
    }

    return 0;
}

static int read_interface(
    AcerContext *ctx,
    char *buffer,
    size_t size)
{
    return acer_driver_interface(
        ctx,
        buffer,
        (unsigned int)size);
}

static int is_acer_vendor(
    const AcerHardwareInfo *info)
{
    if (!info)
        return 0;

    if (!strcasecmp(info->vendor, "Acer"))
        return 1;

    if (!strcasecmp(info->board_vendor, "Acer"))
        return 1;

    if (!strcasecmp(info->chassis_vendor, "Acer"))
        return 1;

    return 0;
}

static int is_clevo_interface(
    const AcerHardwareInfo *info)
{
    if (!info)
        return 0;

    return !strcmp(info->interface_name, "acer_acpi");
}

static AcerHardwarePlatform identify_platform(
    const AcerHardwareInfo *info)
{
    if (!info)
        return ACER_HARDWARE_UNKNOWN;

    if (is_acer_vendor(info) && is_clevo_interface(info))
        return ACER_HARDWARE_CLEVO_DCHU;

    return ACER_HARDWARE_UNKNOWN;
}

int acer_hardware_detect(
    AcerContext *ctx,
    AcerHardwareInfo *info)
{
    int result;

    if (!ctx || !info)
        return -EINVAL;

    memset(info, 0, sizeof(*info));

    result = read_dmi_string(
        DMI_SYS_VENDOR,
        info->vendor,
        sizeof(info->vendor));

    if (result < 0)
        return result;

    result = read_dmi_string(
        DMI_PRODUCT_NAME,
        info->product_name,
        sizeof(info->product_name));

    if (result < 0)
        return result;

    result = read_dmi_string(
        DMI_BOARD_NAME,
        info->board_name,
        sizeof(info->board_name));

    if (result < 0)
        return result;

    result = read_dmi_string(
        DMI_BOARD_VENDOR,
        info->board_vendor,
        sizeof(info->board_vendor));

    if (result < 0)
        return result;

    result = read_dmi_string(
        DMI_CHASSIS_VENDOR,
        info->chassis_vendor,
        sizeof(info->chassis_vendor));

    if (result < 0)
        return result;

    result = read_interface(
        ctx,
        info->interface_name,
        sizeof(info->interface_name));

    if (result < 0)
        return result;

    info->platform = identify_platform(info);
    info->supported = info->platform != ACER_HARDWARE_UNKNOWN;

    return 0;
}

int acer_hardware_is_supported(
    AcerContext *ctx)
{
    AcerHardwareInfo info;
    int result;

    result = acer_hardware_detect(ctx, &info);

    if (result < 0)
        return result;

    return info.supported;
}

const char *acer_hardware_platform_name(
    AcerHardwarePlatform platform)
{
    switch (platform) {
    case ACER_HARDWARE_CLEVO_DCHU:
        return "Clevo DCHU";

    case ACER_HARDWARE_UNKNOWN:
    default:
        return "Unknown";
    }
}
