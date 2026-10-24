/*!
 * Copyright (c) 2019-2023 TUXEDO Computers GmbH <tux@tuxedocomputers.com>
 * Copyright (c) 2026 Stanly Silas <vstanlysilas@gmail.com>
 *
 * This file is part of acer-io.
 *
 * acer-io is free software: you can redistribute it and/or modify
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
#include <linux/kernel.h>
#include <linux/module.h>
#include <linux/init.h>
#include <linux/device.h>
#include <linux/ioctl.h>
#include <linux/fs.h>
#include <linux/cdev.h>
#include <linux/uaccess.h>
#include <linux/delay.h>
#include <linux/version.h>
#include <linux/dmi.h>
#include "../backends/acer/acer_interfaces.h"
#include "../../include/acer_io_ioctl.h"

MODULE_DESCRIPTION("Hardware interface for Acer laptops");
MODULE_AUTHOR("Stanly Silas <vstanlysilas@gmail.com>");
MODULE_VERSION("0.1.0");
MODULE_LICENSE("GPL");

MODULE_ALIAS_ACER_INTERFACES();
MODULE_ALIAS("wmi:" ACER_WMI_METHOD_GUID);

// Initialized in module init, global for ioctl interface
static u32 id_check_acer;

/**
 * strstr version of dmi_match
 */
static bool __attribute__ ((unused)) dmi_string_in(enum dmi_field f, const char *str)
{
	const char *info = dmi_get_system_info(f);

	if (info == NULL || str == NULL)
		return info == str;

	return strstr(info, str) != NULL;
}

static u32 acer_identify(void)
{
	return acer_get_active_interface_id(NULL) == 0 ? 1 : 0;
}

/*static int fop_open(struct inode *inode, struct file *file)
{
	return 0;
}

static int fop_release(struct inode *inode, struct file *file)
{
	return 0;
}*/

static long acer_ioctl_interface(struct file *file, unsigned int cmd, unsigned long arg)
{
	u32 result = 0;
	int status;
	u32 copy_result;
	u32 argument = (u32) arg;

	u32 acer_arg;

	const char str_no_if[] = "";
	char *str_acer_if;
	
	pr_err("acer_io ioctl cmd = 0x%x\n", cmd);
	
	switch (cmd) {
		case R_CL_HW_IF_STR:
			if (acer_get_active_interface_id(&str_acer_if) == 0) {
				copy_result = copy_to_user((char *) arg, str_acer_if, strlen(str_acer_if) + 1);
			} else {
				copy_result = copy_to_user((char *) arg, str_no_if, strlen(str_no_if) + 1);
			}
			break;
		case R_CL_FANINFO1:
			status = acer_evaluate_method(ACER_CMD_GET_FANINFO1, 0, &result);
			pr_err("FANINFO1 raw = 0x%08x\n", result);
			copy_result = copy_to_user((int32_t *) arg, &result, sizeof(result));
			break;
		case R_CL_FANINFO2:
			status = acer_evaluate_method(ACER_CMD_GET_FANINFO2, 0, &result);
			pr_err("FANINFO2 raw = 0x%08x\n", result);
			copy_result = copy_to_user((int32_t *) arg, &result, sizeof(result));
			break;
		case R_CL_FANINFO3:
			status = acer_evaluate_method(ACER_CMD_GET_FANINFO3, 0, &result);
			pr_err("FANINFO3 raw = 0x%08x\n", result);
			copy_result = copy_to_user((int32_t *) arg, &result, sizeof(result));
			break;
		case R_CL_DCHU_STATUS: {
			u8 dchu_status[ACER_DCHU_STATUS_SIZE];

			status = acer_dchu_read_status(dchu_status, sizeof(dchu_status));
			if (status)
				return status;
			if (copy_to_user((void __user *)arg, dchu_status,
					 sizeof(dchu_status)))
				return -EFAULT;
			break;
		}
		/*case R_CL_FANINFO4:
			status = acer_evaluate_method(ACER_CMD_GET_FANINFO4, 0);
			copy_to_user((int32_t *) arg, &result, sizeof(result));
			break;*/
		case R_CL_WEBCAM_SW:
			status = acer_evaluate_method(ACER_CMD_GET_WEBCAM_SW, 0, &result);
			copy_result = copy_to_user((int32_t *) arg, &result, sizeof(result));
			break;
		case R_CL_FLIGHTMODE_SW:
			status = acer_evaluate_method(ACER_CMD_GET_FLIGHTMODE_SW, 0, &result);
			copy_result = copy_to_user((int32_t *) arg, &result, sizeof(result));
			break;
		case R_CL_TOUCHPAD_SW:
			status = acer_evaluate_method(ACER_CMD_GET_TOUCHPAD_SW, 0, &result);
			copy_result = copy_to_user((int32_t *) arg, &result, sizeof(result));
			break;
	}

	switch (cmd) {
		case W_CL_FANSPEED:
	                copy_result = copy_from_user(
		            &argument, (int32_t __user *)arg, sizeof(argument));
	                if (copy_result)
		            return -EFAULT;

	                if (argument == 0x00ffffff)
		            status = acer_dchu_set_fan_mode(1);
	                else
		            status = -EINVAL;

	                if (status)
		            return status;

	                msleep(100);
	                break;

                case W_CL_FANAUTO:
	                copy_result = copy_from_user(
		            &argument, (int32_t __user *)arg, sizeof(argument));
	                if (copy_result)
		            return -EFAULT;

                status = acer_dchu_set_fan_mode(0);
	                if (status)
		            return status;

	                break;
		case W_CL_WEBCAM_SW:
			copy_result = copy_from_user(&argument, (int32_t *) arg, sizeof(argument));
			status = acer_evaluate_method(ACER_CMD_GET_WEBCAM_SW, 0, &result);
			// Only set status if it isn't already the right value
			// (workaround for old and/or buggy WMI interfaces that toggle on write)
			if ((argument & 0x01) != (result & 0x01)) {
				acer_evaluate_method(ACER_CMD_SET_WEBCAM_SW, argument, &result);
			}
			break;
		case W_CL_FLIGHTMODE_SW:
			copy_result = copy_from_user(&argument, (int32_t *) arg, sizeof(argument));
			acer_evaluate_method(ACER_CMD_SET_FLIGHTMODE_SW, argument, &result);
			break;
		case W_CL_TOUCHPAD_SW:
			copy_result = copy_from_user(&argument, (int32_t *) arg, sizeof(argument));
			acer_evaluate_method(ACER_CMD_SET_TOUCHPAD_SW, argument, &result);
			break;
		case W_CL_PERF_PROFILE:
			copy_result = copy_from_user(&argument, (int32_t *) arg, sizeof(argument));
			acer_arg = (ACER_CMD_OPT_SUB_SET_PERF_PROF << 0x18) | (argument & 0xff);
			acer_evaluate_method(ACER_CMD_OPT, acer_arg, &result);
			break;
	}

	return 0;
}

static long fop_ioctl(struct file *file, unsigned int cmd, unsigned long arg)
{
	long status;
	// u32 result = 0;
	u32 copy_result;

	const char *module_version = THIS_MODULE->version;
	switch (cmd) {
		case R_MOD_VERSION:
			copy_result = copy_to_user((char *) arg, module_version, strlen(module_version) + 1);
			break;
		// Hardware id checks, 1 = positive, 0 = negative
		case R_HWCHECK_CL:
			id_check_acer = acer_identify();
			copy_result = copy_to_user((void *) arg, (void *) &id_check_acer, sizeof(id_check_acer));
			break;
	}

	status = acer_ioctl_interface(file, cmd, arg);
	if (status != 0) return status;

	return 0;
}

static struct file_operations fops_dev = {
	.owner              = THIS_MODULE,
	.unlocked_ioctl     = fop_ioctl
//	.open               = fop_open,
//	.release            = fop_release
};

struct class *acer_io_device_class;
dev_t acer_io_device_handle;

static struct cdev acer_io_cdev;

static int __init acer_io_init(void)
{
	int err;

	// Hardware identification
	id_check_acer = acer_identify();

#ifdef DEBUG
	pr_debug("DEBUG is defined\n");

	if (id_check_acer == 0) {
		pr_debug("No matching hardware found on module load\n");
	}
#endif

	err = alloc_chrdev_region(&acer_io_device_handle, 0, 1, "acer_io_cdev");
	if (err != 0) {
		pr_err("Failed to allocate chrdev region\n");
		return err;
	}
	cdev_init(&acer_io_cdev, &fops_dev);
	err = (cdev_add(&acer_io_cdev, acer_io_device_handle, 1));
	if (err < 0) {
		pr_err("Failed to add cdev\n");
		unregister_chrdev_region(acer_io_device_handle, 1);
	}

#if LINUX_VERSION_CODE < KERNEL_VERSION(6, 4, 0)
	acer_io_device_class = class_create(THIS_MODULE, "acer_io");
#else
	acer_io_device_class = class_create("acer_io");
#endif

	device_create(acer_io_device_class, NULL, acer_io_device_handle, NULL, "acer_io");
	pr_debug("Module init successful\n");
	
	return 0;
}

static void __exit acer_io_exit(void)
{
	device_destroy(acer_io_device_class, acer_io_device_handle);
	class_destroy(acer_io_device_class);
	cdev_del(&acer_io_cdev);
	unregister_chrdev_region(acer_io_device_handle, 1);
	pr_debug("Module exit\n");
}

module_init(acer_io_init);
module_exit(acer_io_exit);
