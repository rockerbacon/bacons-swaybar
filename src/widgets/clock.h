#pragma once

#include <time.h>

#include <widgets/widget.h>

extern struct wgt wgt_clock;
void clk_sync_interval(struct timespec*);
