#include <stdio.h>
#include <string.h>
#include <threads.h>

#include <clicks.h>
#include <widgets/widget.h>

#define CLICKS_PAYLOAD_MAX_SIZE 128
#define CLICKS_BUF_SIZE 32

thrd_t clicks_thrd;
struct wgt** clicks_wgts;
size_t clicks_wgt_count;
int clicks_listening;

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

	clicks_listening = 1;
	while(clicks_listening == 1) {
		fgets(payload, CLICKS_PAYLOAD_MAX_SIZE, stdin);

		int left_button = 1;
		size_t wgt_idx = clicks_wgt_count;
		size_t i = 0;
		while (
			i < CLICKS_PAYLOAD_MAX_SIZE &&
			payload[i] != '\0' &&
			wgt_idx == clicks_wgt_count &&
			left_button == 1
		) {
			i = read_param_name(
				payload, CLICKS_PAYLOAD_MAX_SIZE, i, buf, CLICKS_BUF_SIZE
			);

			if (strcmp(buf, "name") == 0) {
				i = read_param_val(
					payload, CLICKS_PAYLOAD_MAX_SIZE, i, buf, CLICKS_BUF_SIZE
				);
				wgt_idx = strtol(buf, NULL, 10);
			} else if (strcmp(buf, "button") == 0) {
				i = read_param_val(
					payload, CLICKS_PAYLOAD_MAX_SIZE, i, buf, CLICKS_BUF_SIZE
				);
				if (buf[0] != '1') {
					left_button = 0;
				}
			} else {
				i = skip_value(payload, CLICKS_PAYLOAD_MAX_SIZE, i);
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

void clicks_stop_thread(void) {
	clicks_listening = 0;
	thrd_join(clicks_thrd, NULL);
}
