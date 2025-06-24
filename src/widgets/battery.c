#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

#include <sway.h>
#include <sysfs.h>
#include <widgets/battery.h>

#define ICN_NO_BATT 0x1f50c
#define ICN_BATT_HIG 0x1f50b
#define ICN_BATT_LOW 0x1faab
#define ICN_BATT_CHR 0x26a1
#define ICN_SQR_EMPTY 0x25ab
#define ICN_SQR_FULL 0x25a0

#define BATT_CRIT 10
#define BATT_LOW 33
#define BATT_HIG 66

#define BATT_UPD_SEC 13

#define BATT_SFS_CAPACITY "/sys/class/power_supply/BAT0/capacity"
#define BATT_SFS_AC_ON "/sys/class/power_supply/AC/online"

struct sfs_param batt_ac_online;
struct sfs_param batt_capacity;
int batt_bar[] = {ICN_SQR_EMPTY, ICN_SQR_FULL};
int batt_capacity_max;
int batt_exists;

void batt_destroy(void) {
	sfs_param_destroy(&batt_ac_online);
	sfs_param_destroy(&batt_capacity);
}

int batt_display(char* buf, size_t bufsize) {
	if (batt_exists == 0) {
		return snprintf(buf, bufsize, "%lc", ICN_NO_BATT);
	}

	int charging = sfs_read_char(&batt_ac_online) == '1';
	int capacity = sfs_read_int(&batt_capacity);
	int main_icn;

	if (charging) {
		main_icn = ICN_BATT_CHR;
	} else if (capacity < BATT_CRIT) {
		main_icn = ICN_BATT_LOW;
	} else {
		main_icn = ICN_BATT_HIG;
	}

	return snprintf(
		buf, bufsize, "%lc %lc%lc%lc",
		main_icn,
		batt_bar[capacity > BATT_CRIT],
		batt_bar[capacity > BATT_LOW],
		batt_bar[capacity > BATT_HIG]
	);
}

void batt_init(void) {
	const char* max_charge_str = getenv("MAX_CHARGE");
	if (max_charge_str == NULL) {
		batt_capacity_max = 100;
	} else {
		batt_capacity_max = strtol(max_charge_str, NULL, 10);
	}

	if (access(BATT_SFS_CAPACITY, F_OK) == 0) {
		batt_exists = 1;
		sfs_param_init(
			BATT_SFS_CAPACITY, &batt_capacity
		);
		sfs_param_init(
			BATT_SFS_AC_ON, &batt_ac_online
		);
	} else {
		batt_exists = 0;
	}
}

void batt_on_click(void) {
	sway_exec("display-battery-stats", NULL);
}

struct wgt wgt_battery = {
	BATT_UPD_SEC, batt_destroy, batt_display, batt_init, batt_on_click
};
