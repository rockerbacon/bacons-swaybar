#include <locale.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <unistd.h>

#include <clicks.h>
#include <sighandler.h>
#include <widgets/battery.h>
#include <widgets/clock.h>
#include <widgets/network.h>
#include <widgets/notifications.h>
#include <widgets/widget.h>

#define PID_PATH_MAX_SIZE 256
#define RUNDIR_MAX_SIZE PID_PATH_MAX_SIZE - 32
#define WGT_BUFSIZE 32
#define RENDER_TIME_WARN_MS 2

char pid_fpath[PID_PATH_MAX_SIZE];

void create_pid_file(void) {
	pid_fpath[0] = '\0';

	char rundir[RUNDIR_MAX_SIZE];
	const char* env_rundir = getenv("XDG_RUNTIME_DIR");
	if (env_rundir == NULL) {
		snprintf(rundir, RUNDIR_MAX_SIZE, "/tmp/bacons-swaybar");
	} else {
		snprintf(rundir, RUNDIR_MAX_SIZE, "%s/bacons-swaybar", env_rundir);
	}

	if (access(rundir, F_OK) != 0) {
		if (mkdir(rundir, 0750) < 0) {
			fprintf(stderr, "ERROR: could not create directory %s\n", rundir);
			return;
		}
	}

	int i = 0;
	int selected_name = 0;
	while (selected_name == 0) {
		snprintf(pid_fpath, PID_PATH_MAX_SIZE, "%s/bacons-swaybar%d.pid", rundir, i++);
		if (access(pid_fpath, F_OK) == 0) {
			fprintf(stderr, "WARN: another instance is already running\n");
		} else {
			selected_name = 1;
		}
	}

	FILE* f = fopen(pid_fpath, "w");
	if (f == NULL) {
		fprintf(stderr, "ERROR: failed to create file %s\n", pid_fpath);
		pid_fpath[0] = '\0';
		return;
	}
	fprintf(f, "%d\n", getpid());
	fclose(f);
}

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

	create_pid_file();

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
			fprintf(stderr, "WARN: update took more than 1 second\n");
		} else {
			long elapsed_ms = (clk_nsec() - nsbegin) / 1e6;
			if (elapsed_ms > RENDER_TIME_WARN_MS) {
				fprintf(stderr, "WARN: update took %ldms\n", elapsed_ms);
			}
		}

		sig_clear();
		clk_sync_interval(&sleep_duration);
		nanosleep(&sleep_duration, NULL);
	}

	for (size_t i = 0; i < wgt_count; i++) {
		if (wgts[i]->destroy != NULL) {
			wgts[i]->destroy();
		}
	}

	if (pid_fpath[0] != '\0') {
		if (unlink(pid_fpath) != 0) {
			fprintf(stderr, "ERROR: failed to cleanup pid file\n");
		}
	}

	return 0;
}
