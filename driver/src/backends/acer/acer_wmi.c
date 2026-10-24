/*!
 * Copyright (c) 2026 Stanly Silas <vstanlysilas@gmail.com>
 *
 * This file is part of acer-keyboard.
 *
 * acer-keyboard is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This software is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this software.  If not, see <https://www.gnu.org/licenses/>.
 */
#define pr_fmt(fmt) KBUILD_MODNAME ": " fmt
#include <linux/acpi.h>
#include <linux/module.h>
#include <linux/wmi.h>
#include <linux/version.h>
#include "acer_interfaces.h"

static int acer_wmi_evaluate(u32 wmi_method_id, u32 wmi_arg, union acpi_object **result)
{
	struct acpi_buffer acpi_buffer_in = { (acpi_size)sizeof(wmi_arg),
					      &wmi_arg };
	struct acpi_buffer acpi_buffer_out = { ACPI_ALLOCATE_BUFFER, NULL };
	union acpi_object *acpi_result;
	acpi_status status_acpi;
	int return_status = 0;

	status_acpi =
		wmi_evaluate_method(ACER_WMI_METHOD_GUID, 0x00, wmi_method_id,
				    &acpi_buffer_in, &acpi_buffer_out);

	if (unlikely(ACPI_FAILURE(status_acpi))) {
		pr_err("failed to evaluate wmi method\n");
		return -EIO;
	}

	acpi_result = (union acpi_object *)acpi_buffer_out.pointer;
	if (!acpi_result) {
		pr_err("failed to evaluate WMI method\n");
		return_status = -1;
	}
	else {
		if (!IS_ERR_OR_NULL(result)) {
			*result = acpi_result;
		}
	}

	return return_status;
}

static int acer_wmi_interface_method_call(u8 cmd, u32 arg, union acpi_object **result_value)
{
	return acer_wmi_evaluate(cmd, arg, result_value);
}

struct acer_interface_t acer_wmi_interface = {
	.string_id = ACER_INTERFACE_WMI_STRID,
	.method_call = acer_wmi_interface_method_call,
};

#if LINUX_VERSION_CODE < KERNEL_VERSION(5, 3, 0)
static int acer_wmi_probe(struct wmi_device *wdev)
#else
static int acer_wmi_probe(struct wmi_device *wdev, const void *dummy_context)
#endif
{
	int status;
	union acpi_object *out_obj;

	pr_debug("acer_wmi driver probe\n");

	if (!wmi_has_guid(ACER_WMI_EVENT_GUID)) {
		pr_debug("probe: Acer event guid missing\n");
		return -ENODEV;
	}

	if (!wmi_has_guid(ACER_WMI_METHOD_GUID)) {
		pr_debug("probe: Acer method guid missing\n");
		return -ENODEV;
	}

	// Since the WMI GUIDs aren't unique let's (at least)
	// check the return of some "known existing general" method
	status = acer_wmi_evaluate(0x52, 0, &out_obj);
	if (status < 0) {
		pr_debug("probe: Acer GUIDs present but method call failed\n");
		return -ENODEV;
	}
	if (out_obj->type != ACPI_TYPE_INTEGER || (out_obj->type == ACPI_TYPE_INTEGER && (u32)out_obj->integer.value == 0xffffffff)) {
		pr_debug(
			"probe: Acer GUIDs present but method returned unexpected value\n");
		ACPI_FREE(out_obj);
		return -ENODEV;
	}
	ACPI_FREE(out_obj);

	// Add this interface
	acer_keyboard_add_interface(&acer_wmi_interface);

	pr_info("interface initialized\n");

	return 0;
}

#if LINUX_VERSION_CODE < KERNEL_VERSION(5, 13, 0)
static int acer_wmi_remove(struct wmi_device *wdev)
#else
static void acer_wmi_remove(struct wmi_device *wdev)
#endif
{
	pr_debug("acer_wmi driver remove\n");
	acer_keyboard_remove_interface(&acer_wmi_interface);
#if LINUX_VERSION_CODE < KERNEL_VERSION(5, 13, 0)
	return 0;
#endif
}

static void acer_wmi_notify(struct wmi_device *wdev, union acpi_object *dummy)
{
	u32 event_value;
	union acpi_object *out_obj;
	int status;

	status = acer_wmi_evaluate(0x01, 0, &out_obj);
	if (!status) {
		if (out_obj->type == ACPI_TYPE_INTEGER) {
			event_value = (u32)out_obj->integer.value;
			pr_info("firmware hotkey event: source=WMI code=0x%04x\n",
				event_value);
		} else {
			pr_err("return type not integer, use acer_evaluate_method2\n");
		}
		ACPI_FREE(out_obj);
	}
	pr_debug("acer_wmi notify\n");
	if (!IS_ERR_OR_NULL(acer_wmi_interface.event_callb)) {
		// Execute registered callback
		acer_wmi_interface.event_callb(event_value);
	}
}

static const struct wmi_device_id acer_wmi_device_ids[] = {
	// Listing one should be enough, for a driver that "takes care of all anyways"
	// also prevents probe (and handling) per "device"
	{ .guid_string = ACER_WMI_EVENT_GUID },
	{ }
};

static struct wmi_driver acer_wmi_driver = {
	.driver = {
		.name = ACER_INTERFACE_WMI_STRID,
		.owner = THIS_MODULE
	},
	.id_table = acer_wmi_device_ids,
	.probe = acer_wmi_probe,
	.remove = acer_wmi_remove,
	.notify = acer_wmi_notify,
};

module_wmi_driver(acer_wmi_driver);

MODULE_AUTHOR("Stanly Silas <vstanlysilas@gmail.com>");
MODULE_DESCRIPTION("Driver for Acer WMI interface");
MODULE_VERSION("0.1.0");
MODULE_LICENSE("GPL");

MODULE_DEVICE_TABLE(wmi, acer_wmi_device_ids);
MODULE_ALIAS_ACER_WMI();
