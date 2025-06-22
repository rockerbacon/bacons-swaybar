#define _POSIX_C_SOURCE 199309L

#include <locale.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#include <widgets/battery.h>
#include <widgets/clock.h>

#define BUFFSIZE 512

int main(void) {
	char buff[BUFFSIZE];
	struct wgt* wgts[] = {
		&wgt_battery,
		&wgt_clock,
	};
	size_t wgt_count = sizeof(wgts) / sizeof(struct wgt*);
	struct timespec sleep_duration;

	memset(buff, 0, BUFFSIZE);

	setlocale(LC_CTYPE, "en_US.UTF-8");

	for (size_t i = 0; i < wgt_count; i++) {
		struct wgt* w = wgts[i];
		if (w->init != NULL) {
			w->init();
		}
	}

	while(1) {
		size_t offset = 0;
		for (size_t i = 0; i < wgt_count; i++) {
			struct wgt* w = wgts[i];
			if (w->display != NULL) {
				offset += w->display(buff+offset, BUFFSIZE-offset);
			}

			if (offset >= BUFFSIZE) {
				fprintf(stderr, "FATAL: buffer overflow\n");
				exit(1);
			}
		}

		fprintf(stdout, "%s\n", buff);
		clk_sync_interval(&sleep_duration);
		nanosleep(&sleep_duration, NULL);
	}

	return 0;
}
