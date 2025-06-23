#pragma once

#include <time.h>

#include <widgets/widget.h>

extern struct wgt wgt_clock;

void clk_sync_interval(struct timespec*);
time_t clk_sec(void);
long clk_nsec(void);
void clk_upd(void);
