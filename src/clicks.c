#include <poll.h>
#include <stdio.h>
#include <string.h>
#include <threads.h>

#include <clicks.h>
#include <sighandler.h>
#include <unistd.h>
#include <widgets/widget.h>

#define CLICKS_BUF_SIZE 32
#define CLICKS_PAYLOAD_MAX_SIZE 128
#define CLICKS_POLL_TIMEOUT 5000

thrd_t clicks_thrd;

struct wgt** clicks_wgts;
size_t clicks_wgt_count;

size_t read_param_name(
	const char* payload, size_t payload_size, size_t i, char* buf, size_t bufsize
) {
	int next_state = 0;
	while (next_state == 0 && i < payload_size) {
		switch (payload[i]) {
			case '[':
			case ',':
			case ' ':
			case '\t':
			case '{':
			case '"':
				i++;
				break;
			default:
				next_state = 1;
		}
	}

	int no_param = 0;
	if (i >= payload_size) {
		no_param = 1;
	} else if (payload[i] == '\0') {
		no_param = 1;
	}

	if (no_param == 1) {
		fprintf(stderr, "WARN: received invalid input\n");
		buf[0] = '\0';
		return i;
	}

	int j = 0;
	while (
		i < payload_size && j < bufsize - 1 &&
		payload[i] != '"' && payload[i] != '\0'
	) {
		buf[j++] = payload[i++];
	}
	buf[j] = '\0';

	return i;
}

size_t read_param_val(
	const char* payload, size_t payload_size, size_t i, char* buf, size_t bufsize
) {
	while (i < payload_size && payload[i] != ':' && payload[i] != '\0') {
		i++;
	}
	i++;

	if (i >= payload_size || payload[i-1] == '\0') {
		fprintf(stderr, "WARN: no parameter value in input\n");
		buf[0] = '\0';
		return i;
	}

	int next_state = 0;
	while (
		i < payload_size && payload[i] != '\0' && next_state == 0
	) {
		switch(payload[i]) {
			case ' ':
			case '\t':
			case '"':
				i++;
				break;
			default:
				next_state = 1;
		}
	}

	int j = 0;
	next_state = 0;
	while (
		i < payload_size && payload[i] != '\0' &&
		j < bufsize && next_state == 0
	) {
		switch(payload[i]) {
			case ',':
			case '}':
			case '"':
				next_state = 1;
				break;
			default:
				buf[j++] = payload[i++];
		}
	}
	buf[j] = '\0';

	if (j == 0) {
		fprintf(stderr, "WARN: no parameter value in input\n");
		buf[0] = '\0';
	}

	return i;
}

size_t skip_value(const char* payload, size_t payload_size, size_t i) {
	int done = 0;
	while (i < payload_size && payload[i] != '\0' && done == 0) {
		switch(payload[i]) {
			case ',':
			case '}':
				done = 1;
		}
		i++;
	}
	return i;
}

int clicks_listen(void* args) {
	char payload[CLICKS_PAYLOAD_MAX_SIZE];
	char buf[CLICKS_BUF_SIZE];

	struct pollfd stdinfd = {
		0, POLLIN, 0
	};

	while(sig_term == 0) {
		int ready = poll(&stdinfd, 1, CLICKS_POLL_TIMEOUT);
		if (ready < 0) {
			fprintf(stderr, "ERROR: clicks thread failed to poll stdin\n");
			return 1;
		} else if (ready == 0) {
			continue;
		}

		ssize_t payloadsize = read(
			stdinfd.fd, payload, CLICKS_PAYLOAD_MAX_SIZE-1
		);
		if (payloadsize < 0) {
			fprintf(stderr, "ERROR: clicks thread failed to read stdin\n");
			return 1;
		} else if (payloadsize == CLICKS_PAYLOAD_MAX_SIZE-1) {
			fprintf(stderr, "WARN: click payload too large\n");
		}
		payload[payloadsize] = '\0';

		int left_button = 1;
		size_t wgt_idx = clicks_wgt_count;
		size_t i = 0;
		while (
			i < payloadsize &&
			payload[i] != '\0' &&
			wgt_idx == clicks_wgt_count &&
			left_button == 1
		) {
			i = read_param_name(
				payload, payloadsize, i, buf, CLICKS_BUF_SIZE
			);

			if (strcmp(buf, "name") == 0) {
				i = read_param_val(
					payload, payloadsize, i, buf, CLICKS_BUF_SIZE
				);
				wgt_idx = strtol(buf, NULL, 10);
			} else if (strcmp(buf, "button") == 0) {
				i = read_param_val(
					payload, payloadsize, i, buf, CLICKS_BUF_SIZE
				);
				if (buf[0] != '1') {
					left_button = 0;
				}
			} else {
				i = skip_value(payload, payloadsize, i);
			}
		}

		if (wgt_idx < clicks_wgt_count && left_button == 1) {
			if (clicks_wgts[wgt_idx]->on_click != NULL) {
				clicks_wgts[wgt_idx]->on_click();
			}
		}
	}

	return 0;
}

void clicks_start_thread(struct wgt** wgts, size_t wgt_count) {
	clicks_wgts = wgts;
	clicks_wgt_count = wgt_count;

	int s = thrd_create(&clicks_thrd, clicks_listen, NULL);
	if (s != thrd_success) {
		fprintf(stderr, "FATAL: failed to spawn clicks thread\n");
		exit(1);
	}
}
