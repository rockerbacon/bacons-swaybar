#include <fcntl.h>
#include <stdio.h>
#include <sys/wait.h>
#include <unistd.h>

#include <sway.h>
#include <widgets/notifications.h>

#define ICN_NTF_ENABLED 0x1F514
#define ICN_NTF_DISABLED 0x1F515

void ntf_display(FILE* out) {
	int pid = fork();
	if (pid == 0) {
		close(1);
		int fd = open("/dev/null", O_WRONLY);
		if (fd < 0) {
			fprintf(stderr, "WARN: could not redirect stdout to /dev/null\n");
		} else {
			dup2(fd, 1);
		}
		execlp("notifications-enabled", "notifications-enabled");
	} else {
		int wstatus;
		waitpid(pid, &wstatus, 0);
		if (WEXITSTATUS(wstatus) == 0) {
			fprintf(out, "%lc", ICN_NTF_ENABLED);
		} else {
			fprintf(out, "%lc", ICN_NTF_DISABLED);
		}
	}
}

void ntf_on_click(void) {
	sway_exec("notification-center", NULL);
}

struct wgt wgt_notifications = {
	ntf_display,
	NULL,
	ntf_on_click,
};
