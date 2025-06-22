#include <locale.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#include <clicks.h>

#include <widgets/battery.h>
#include <widgets/clock.h>
#include <widgets/network.h>
#include <widgets/notifications.h>

int main(void) {
	struct wgt* wgts[] = {
		&wgt_notifications,
		&wgt_battery,
		&wgt_network,
		&wgt_clock,
	};
	size_t wgt_count = sizeof(wgts) / sizeof(struct wgt*);
	struct timespec sleep_duration;

	setlocale(LC_CTYPE, "en_US.UTF-8");

	for (size_t i = 0; i < wgt_count; i++) {
		struct wgt* w = wgts[i];
		if (w->init != NULL) {
			w->init();
		}
	}

	clicks_start_thread(wgts, wgt_count);

	fprintf(stdout, "{\"version\":1,\"click_events\":true}\n[");

	while(1) {
		fprintf(stdout, "[{\"name\":\"0\",\"full_text\":\"   ");
		wgts[0]->display(stdout);
		fprintf(stdout, "   \"}");

		for (size_t i = 1; i < wgt_count; i++) {
			fprintf(stdout, ",{\"name\":\"%lu\",\"full_text\":\"   ", i);
			wgts[i]->display(stdout);
			fprintf(stdout, "   \"}");
		}

		fprintf(stdout, "],\n");
		fflush(stdout);

		clk_sync_interval(&sleep_duration);
		nanosleep(&sleep_duration, NULL);
	}

	return 0;
}
