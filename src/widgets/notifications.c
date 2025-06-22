#include <fcntl.h>
#include <stdio.h>
#include <sys/wait.h>
#include <unistd.h>

#include <sway.h>
#include <widgets/notifications.h>

#define ICN_NTF_ENABLED 0x1F514
#define ICN_NTF_DISABLED 0x1F515

int ntf_display(char* buf, size_t bufsize) {
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
		return 0;
	} else {
		int wstatus;
		waitpid(pid, &wstatus, 0);
		if (WEXITSTATUS(wstatus) == 0) {
			return snprintf(buf, bufsize, "%lc", ICN_NTF_ENABLED);
		} else {
			return snprintf(buf, bufsize, "%lc", ICN_NTF_DISABLED);
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
