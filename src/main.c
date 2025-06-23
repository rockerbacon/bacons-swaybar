#include <locale.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#include <clicks.h>
#include <sighandler.h>
#include <widgets/battery.h>
#include <widgets/clock.h>
#include <widgets/network.h>
#include <widgets/notifications.h>
#include <widgets/widget.h>

#define WGT_BUFSIZE 32
#define RENDER_TIME_WARN_MS 2

int main(void) {
	struct timespec sleep_duration;

	struct wgt* wgts[] = {
		&wgt_notifications,
		&wgt_battery,
		&wgt_network,
		&wgt_clock,
	};

	size_t wgt_count = sizeof(wgts) / sizeof(struct wgt*);

	time_t wgt_last_upd[wgt_count];

	char buf1[WGT_BUFSIZE*wgt_count];
	char buf2[WGT_BUFSIZE*wgt_count];
	char* wgt_fbuf[wgt_count];
	char* wgt_bbuf[wgt_count];

	memset(wgt_last_upd, 0, sizeof(wgt_last_upd));

	sig_register_hndls();

	setlocale(LC_CTYPE, "en_US.UTF-8");

	for (size_t i = 0; i < wgt_count; i++) {
		if (wgts[i]->init != NULL) {
			wgts[i]->init();
		}

		wgt_fbuf[i] = buf1 + WGT_BUFSIZE*i;
		wgt_bbuf[i] = buf2 + WGT_BUFSIZE*i;
		wgt_fbuf[i][0] = '\0';
		wgt_bbuf[i][0] = '\0';
	}

	clicks_start_thread(wgts, wgt_count);

	fprintf(stdout, "{\"version\":1,\"click_events\":true}\n[");

	while(sig_term == 0) {
		clk_upd();
		time_t sbegin = clk_sec();
		long nsbegin = clk_nsec();

		int changes = 0;
		for (size_t i = 0; i < wgt_count; i++) {
			if (sbegin - wgt_last_upd[i] >= wgts[i]->upd_sec) {
				int output = wgts[i]->display(wgt_bbuf[i], WGT_BUFSIZE);

				wgt_last_upd[i] = sbegin;

				if (output > 0) {
					char* tmp = wgt_fbuf[i];
					wgt_fbuf[i] = wgt_bbuf[i];
					wgt_bbuf[i] = tmp;

					if (changes == 0) {
						if (strcmp(wgt_fbuf[i], wgt_bbuf[i]) != 0) {
							changes = 1;
						}
					}
				}
			}
		}

		if (changes != 0) {
			fprintf(
				stdout, "[{\"name\":\"0\",\"full_text\":\"   %s   \"}",
				wgt_fbuf[0]
			);

			for (size_t i = 1; i < wgt_count; i++) {
				fprintf(
					stdout, ",{\"name\":\"%lu\",\"full_text\":\"   %s   \"}",
					i, wgt_fbuf[i]
				);
			}

			fprintf(stdout, "],\n");
			fflush(stdout);
		}

		clk_upd();
		if (clk_sec() - sbegin > 0) {
			fprintf(stderr, "WARN: rendering took more than 1 second\n");
		} else {
			long elapsed_ms = (clk_nsec() - nsbegin) / 1e6;
			if (elapsed_ms > RENDER_TIME_WARN_MS) {
				fprintf(stderr, "WARN: rendering took %ldms\n", elapsed_ms);
			}
		}

		clk_sync_interval(&sleep_duration);
		nanosleep(&sleep_duration, NULL);
	}

	for (size_t i = 0; i < wgt_count; i++) {
		if (wgts[i]->destroy != NULL) {
			wgts[i]->destroy();
		}
	}

	return 0;
}
