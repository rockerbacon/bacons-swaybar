#pragma once

#include <stdio.h>

struct wgt {
	void (*display)(FILE*);
	void (*init)(void);
	void (*on_click)(void);
};
