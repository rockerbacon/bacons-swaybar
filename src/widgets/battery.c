#include <stdio.h>
#include <stdlib.h>

#include <sway.h>
#include <sysfs.h>
#include <widgets/battery.h>

#define ICN_BATT_HIG 0x1f50b
#define ICN_BATT_LOW 0x1faab
#define ICN_BATT_CHR 0x26a1
#define ICN_SQR_EMPTY 0x25ab
#define ICN_SQR_FULL 0x25a0

#define BATT_CRIT 10
#define BATT_LOW 33
#define BATT_HIG 66

struct sfs_param ac_online;
struct sfs_param batt_capacity;
int batt_capacity_max;
size_t anim_cycle = 0;
const int chr_anim[] = {ICN_SQR_EMPTY, ICN_SQR_FULL};

void batt_display(FILE* out) {
	int charging = sfs_read_char(&ac_online) == '1';
	int capacity = sfs_read_int(&batt_capacity);

	if (charging) {
		const int anim_sqr = chr_anim[anim_cycle];
		anim_cycle ^= 1;

		if (capacity < BATT_LOW) {
			fprintf(
				out, "%lc%lc%lc%lc",
				ICN_BATT_CHR, anim_sqr, ICN_SQR_EMPTY, ICN_SQR_EMPTY
			);
		} else if (capacity < BATT_HIG) {
			fprintf(
				out, "%lc%lc%lc%lc",
				ICN_BATT_CHR, ICN_SQR_FULL, anim_sqr, ICN_SQR_EMPTY
			);
		} else if (capacity < batt_capacity_max - 5) {
			fprintf(
				out, "%lc%lc%lc%lc",
				ICN_BATT_CHR, ICN_SQR_FULL, ICN_SQR_FULL, anim_sqr
			);
		} else {
			fprintf(
				out, "%lc%lc%lc%lc",
				ICN_BATT_CHR, ICN_SQR_FULL, ICN_SQR_FULL, ICN_SQR_FULL
			);
		}
	} else {
		if (capacity < BATT_CRIT) {
			fprintf(
				out, "%lc%lc%lc%lc",
				ICN_BATT_LOW, ICN_SQR_EMPTY, ICN_SQR_EMPTY, ICN_SQR_EMPTY
			);
		} else if (capacity < BATT_LOW) {
			fprintf(
				out, "%lc%lc%lc%lc",
				ICN_BATT_LOW, ICN_SQR_FULL, ICN_SQR_EMPTY, ICN_SQR_EMPTY
			);
		} else if (capacity < BATT_HIG) {
			fprintf(
				out, "%lc%lc%lc%lc",
				ICN_BATT_HIG, ICN_SQR_FULL, ICN_SQR_FULL, ICN_SQR_EMPTY
			);
		} else {
			fprintf(
				out, "%lc%lc%lc%lc",
				ICN_BATT_HIG, ICN_SQR_FULL, ICN_SQR_FULL, ICN_SQR_FULL
			);
		}
	}
}

void batt_init(void) {
	const char* max_charge_str = getenv("MAX_CHARGE");
	if (max_charge_str == NULL) {
		batt_capacity_max = 100;
	} else {
		batt_capacity_max = strtol(max_charge_str, NULL, 10);
	}
	sfs_param_init(
		"/sys/class/power_supply/BAT0/capacity", &batt_capacity
	);
	sfs_param_init(
		"/sys/class/power_supply/AC/online", &ac_online
	);
}

void batt_on_click(void) {
	sway_exec("display-battery-stats", NULL);
}

struct wgt wgt_battery = {
	batt_display,
	batt_init,
	batt_on_click,
};
