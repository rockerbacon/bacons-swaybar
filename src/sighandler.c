#include <signal.h>
#include <memory.h>

#include <sighandler.h>

volatile sig_atomic_t sig_term;
int sig_usr1;

void sig_hndl_term(int signum) {
	sig_term = signum;
}

void sig_hndl_usr(int signum) {
	sig_usr1 = signum;
}

void sig_register_hndls(void) {
	struct sigaction sigterm_hndl;
	memset(&sigterm_hndl, 0, sizeof(struct sigaction));
	sigterm_hndl.sa_handler = sig_hndl_term;
	sigterm_hndl.sa_flags = SA_RESETHAND;
	sigaction(SIGINT, &sigterm_hndl, NULL);
	sigaction(SIGTERM, &sigterm_hndl, NULL);

	struct sigaction sigusr_hndl;
	memset(&sigusr_hndl, 0, sizeof(struct sigaction));
	sigusr_hndl.sa_handler = sig_hndl_usr;
	sigaction(SIGUSR1, &sigusr_hndl, NULL);

	sig_term = 0;
	sig_usr1 = 0;
}

void sig_clear(void) {
	sig_usr1 = 0;
}
