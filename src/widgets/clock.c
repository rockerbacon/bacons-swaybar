#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <time.h>

#include <widgets/widget.h>

#define PRECISION_MIN 1
#define PRECISION_SEC 2
#define PRECISION_MSEC 3

#define CLK_UPD_SEC 0

int precision;
int refresh;
struct timespec ts;
struct tm dt;

int clk_display(char* buf, size_t bufsize) {
	localtime_r(&ts.tv_sec, &dt);

	int output = snprintf(
		buf, bufsize, "%d-%02d-%02d %02d:%02d",
		dt.tm_year + 1900, dt.tm_mon + 1, dt.tm_mday, dt.tm_hour, dt.tm_min
	);

	if (precision >= PRECISION_SEC) {
		output += snprintf(buf+output, bufsize-output, ":%02d", dt.tm_sec);
	}

	if (precision >= PRECISION_MSEC) {
		int msec = ts.tv_nsec / 1e6;
		output += snprintf(buf+output, bufsize-output, ".%03d", msec);
	}

	return output;
}

void clk_init(void) {
	const char* precision_str = getenv("CLOCK_PRECISION");
	const char* refresh_str = getenv("CLOCK_REFRESH");

	if (precision_str == NULL) {
		precision = PRECISION_MIN;
	} else if (strcmp(precision_str, "milliseconds") == 0) {
		precision = PRECISION_MSEC;
	} else if (strcmp(precision_str, "seconds") == 0) {
		precision = PRECISION_SEC;
	} else if (strcmp(precision_str, "minutes") == 0) {
		precision = PRECISION_MIN;
	} else {
		fprintf(stderr, "FATAL: invalid CLOCK_PRECISION\n");
		exit(1);
	}

	if (refresh_str == NULL) {
		refresh = PRECISION_SEC;
	} else if (strcmp(refresh_str, "seconds") == 0) {
		refresh = PRECISION_SEC;
	} else if (strcmp(refresh_str, "minutes") == 0) {
		refresh = PRECISION_MIN;
	} else {
		fprintf(stderr, "FATAL: invalid CLOCK_REFRESH\n");
		exit(1);
	}
}

void clk_sync_interval(struct timespec* sleep_duration) {
	sleep_duration->tv_nsec = 1e9 - ts.tv_nsec;

	if (refresh == PRECISION_SEC) {
		sleep_duration->tv_sec = 0;
	} else {
		sleep_duration->tv_sec = 59 - dt.tm_sec;
	}
}

time_t clk_sec(void) {
	return ts.tv_sec;
}

long clk_nsec(void) {
	return ts.tv_nsec;
}

void clk_upd(void) {
	clock_gettime(CLOCK_REALTIME, &ts);
}

struct wgt wgt_clock = {
	CLK_UPD_SEC, NULL, clk_display, clk_init, NULL
};
