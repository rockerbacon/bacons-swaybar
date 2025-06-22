#include <locale.h>
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#include <clicks.h>

#include <widgets/battery.h>
#include <widgets/clock.h>
#include <widgets/network.h>
#include <widgets/notifications.h>

#define MAIN_BUFSIZE 256

struct wgt* wgts[] = {
	&wgt_notifications,
	&wgt_battery,
	&wgt_network,
	&wgt_clock,
};
size_t wgt_count = sizeof(wgts) / sizeof(struct wgt*);

int running;

void hndl_term(int) {
	running = 0;
}

int main(void) {
	char buf1[MAIN_BUFSIZE];
	char buf2[MAIN_BUFSIZE];
	char* fbuf;
	char* bbuf;
	struct timespec sleep_duration;

	setlocale(LC_CTYPE, "en_US.UTF-8");

	for (size_t i = 0; i < wgt_count; i++) {
		struct wgt* w = wgts[i];
		if (w->init != NULL) {
			w->init();
		}
	}

	struct sigaction sigint_hndl;
	memset(&sigint_hndl, 0, sizeof(struct sigaction));
	sigint_hndl.sa_handler = hndl_term;
	sigint_hndl.sa_flags = SA_RESETHAND;
	sigaction(SIGINT, &sigint_hndl, NULL);
	sigaction(SIGTERM, &sigint_hndl, NULL);

	clicks_start_thread(wgts, wgt_count);

	fprintf(stdout, "{\"version\":1,\"click_events\":true}\n[");

	fbuf = buf1;
	memset(fbuf, 0, MAIN_BUFSIZE);
	bbuf = buf2;

	size_t bufslice = MAIN_BUFSIZE / wgt_count;

	running = 1;
	while(running) {
		int changes = 0;
		for (size_t i = 0; i < wgt_count; i++) {
			char* new = bbuf + i*bufslice;

			int output = wgts[i]->display(new, bufslice);

			if (changes == 0 && output > 0) {
				char* old = fbuf + i*bufslice;

				if (strcmp(new, old) != 0) {
					changes = 1;
				}
			}
		}

		char *tmp = fbuf;
		fbuf = bbuf;
		bbuf = tmp;

		if (changes != 0) {
			fprintf(
				stdout, "[{\"name\":\"0\",\"full_text\":\"   %s   \"}", fbuf
			);

			for (size_t i = 1; i < wgt_count; i++) {
				fprintf(
					stdout, ",{\"name\":\"%lu\",\"full_text\":\"   %s   \"}",
					i, fbuf + i*bufslice
				);
			}

			fprintf(stdout, "],\n");
			fflush(stdout);
		}

		clk_sync_interval(&sleep_duration);
		nanosleep(&sleep_duration, NULL);
	}

	clicks_stop_thread();

	for (size_t i = 0; i < wgt_count; i++) {
		if (wgts[i]->destroy != NULL) {
			wgts[i]->destroy();
		}
	}

	return 0;
}
