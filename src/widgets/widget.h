#pragma once

#include <stdlib.h>

struct wgt {
	unsigned upd_sec;
	void (*destroy)(void);
	int (*display)(char*, size_t);
	void (*init)(void);
	void (*on_click)(void);
};
