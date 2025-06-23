#pragma once

#include <signal.h>

extern volatile sig_atomic_t sig_term;
extern int sig_usr1;

void sig_register_hndls(void);
void sig_clear(void);
