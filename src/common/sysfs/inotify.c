#include <stdio.h>
#include <sys/inotify.h>
#include <errno.h>
#include <unistd.h>

#define MAX_NAMELEN 64
#define NEXT(it, bytes) { \
	bytes -= sizeof(struct inotify_event) + it->len; \
	it = (struct inotify_event*)((char*)it + sizeof(struct inotify_event) + it->len); \
}
#define OK(it, bytes, begin, buflen) ( \
	(size_t)it < (size_t)begin + buflen && \
	bytes >= sizeof(struct inotify_event)+it->len \
)

/**
 * Reads inotify messages, storing the raw bytes in buf
 * and populating vec with pointers to the start of each
 * inotify_event structure.
 *
 * @param fd the file descriptor to read from
 * @param buf the buffer on which to store the raw bytes
 * @param buflen the size of the buf in bytes
 * @param vec array for storing pointers to the inotify_event structures
 * @param veclen the number of pointers that vec can hold
 *
 * @returns
 *   the number of messages received when successful
 *   -1 when a system error occurs, in which case errno should be set
 *   -2 when vec cannot hold all messages
 *   -3 when a message would overflow MAX_NAMELEN
 */
ssize_t sysfsinotify_recvmsg(
	int fd,
	void *buf,
	size_t buflen, // in bytes
	void **vec,
	size_t veclen // in number of items
) {
	int msgcount = 0;
	ssize_t bytes = read(fd, buf, buflen);

	if (bytes == EWOULDBLOCK) {
		return 0;
	} else if (bytes < 0) {
		return -1;
	}

	struct inotify_event *it = (struct inotify_event*)buf;
	while (OK(it, bytes, buf, buflen)) {
		if (msgcount == veclen) {
			return -2;
		} else if (it->len >= MAX_NAMELEN) {
			return -3;
		}

		vec[msgcount] = it;
		NEXT(it, bytes);
		msgcount++;
	}

	return msgcount;
}

/**
 * Discards all currently pending inotify messages
 *
 * @param fd the file descriptor to read from
 * @param buf the buffer on which to store the raw bytes
 * @param buflen the size of the buf in bytes
 *
 * @returns
 *   0 when successful
 *   -1 when a system error occurs, in which case errno should be set
 */
ssize_t sysfsinotify_discmsg(int fd, void *buf, size_t buflen) {
	while (read(fd, buf, buflen) > 0) {}

	if (errno != EWOULDBLOCK) {
		return -1;
	}

	return 0;
}
