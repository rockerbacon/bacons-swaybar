#include <ifaddrs.h>
#include <net/if.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/types.h>
#include <unistd.h>

#include <sway.h>
#include <sysfs.h>
#include <widgets/network.h>
#include <widgets/widget.h>

#define ICN_LAPTOP 0x1F4BB
#define ICN_GLOBE 0x1F310
#define ICN_QUESTION 0x2753

#define NET_PATH_MAX_SIZE 128

int net_display(char* buf, size_t bufsize) {
	static char path[NET_PATH_MAX_SIZE];
	struct ifaddrs* addrs;

	getifaddrs(&addrs);

	int conn_wireless = 0;
	int conn_wired = 0;
	struct ifaddrs* i = addrs;
	while (conn_wireless == 0 && i != NULL) {
		if (
			((IFF_LOOPBACK | IFF_NOARP) & i->ifa_flags) == 0 &&
			(
				i->ifa_addr->sa_family == AF_INET ||
				i->ifa_addr->sa_family == AF_INET6
			)
		) {
			snprintf(
				path, NET_PATH_MAX_SIZE, "/sys/class/net/%s/wireless", i->ifa_name
			);
			if (access(path, F_OK) == 0) {
				conn_wireless = 1;
			} else {
				conn_wired = 1;
			}
		}

		i = i->ifa_next;
	}

	freeifaddrs(addrs);

	if (conn_wireless) {
		return snprintf(buf, bufsize, "%lc   %lc", ICN_LAPTOP, ICN_GLOBE);
	} else if (conn_wired) {
		return snprintf(buf, bufsize, "%lc - %lc", ICN_LAPTOP, ICN_GLOBE);
	} else {
		return snprintf(buf, bufsize, "%lc x %lc", ICN_LAPTOP, ICN_QUESTION);
	}
}

void net_on_click(void) {
	sway_exec("manage-network", NULL);
}

struct wgt wgt_network = {
	*net_display,
	NULL,
	*net_on_click,
};
