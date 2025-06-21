#pragma once

#include <stdlib.h>

struct wgt {
	int (*display)(char*,size_t);
	void (*init)(void);
	void (*on_click)(void);
};
