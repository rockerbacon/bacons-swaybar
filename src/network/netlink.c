#define _GNU_SOURCE

#include <errno.h>
#include <fcntl.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#include <linux/netlink.h>
#include <linux/rtnetlink.h>
#include <sys/socket.h>
#include <sys/uio.h>

const unsigned IPADD = RTM_NEWADDR;
const unsigned IPRMV = RTM_DELADDR;
const size_t BUFSIZE = 256;

struct ifmodmsg {
	unsigned devidx;
	char modop;
	unsigned ipv4;
};

struct nlsock {
	int fd;
	struct msghdr bufh;
	struct sockaddr_nl addr;
	struct iovec bufv;
	char *buf;
};

/**
 * Opens a new nlsock
 *
 * @returns
 *     a newly allocated socket if successful
 *     null if an error occurs, in which case errno should be set
 */
struct nlsock* nlsock_open() {
	int fd = socket(AF_NETLINK, SOCK_RAW, NETLINK_ROUTE);

	if (fd < 0) {
		return NULL;
	}

	struct nlsock* sock = (struct nlsock*)malloc(
		sizeof(struct nlsock)+BUFSIZE
	);
	memset(sock, 0, sizeof(struct nlsock)+BUFSIZE);

	sock->fd = fd;

	sock->addr.nl_family = AF_NETLINK;
	sock->addr.nl_groups = RTMGRP_IPV4_IFADDR;
	sock->addr.nl_pid = gettid() << 16 | getpid();

	sock->bufh.msg_name = &sock->addr;
	sock->bufh.msg_namelen = sizeof(struct sockaddr_nl);
	sock->bufh.msg_iov = &sock->bufv;
	sock->bufh.msg_iovlen = 1;

	sock->bufv.iov_base = (char*)sock + sizeof(struct nlsock);
	sock->bufv.iov_len = BUFSIZE;

	if (
		bind(
			sock->fd, (struct sockaddr*)&sock->addr, sizeof(struct sockaddr_nl)
		) < 0
	) {
		goto panic;
	}

	if (fcntl(sock->fd, F_SETFL, O_NONBLOCK) < 0) {
		goto panic;
	}

	return sock;

panic:
	close(sock->fd);
	free(sock);
	return NULL;
}

/**
 * Closes an open nlsock
 *
 * @returns
 *     0 if successful
 *     -1 if an error occurs, in which case errno should be set
 */
int nlsock_close(struct nlsock* sock) {
	if (close(sock->fd) < 0) {
		return -1;
	}

	free(sock);
	return 0;
}

/**
 * Reads buffered messages,
 * storing them in vec.
 *
 * @param bufv the buffer to read from
 * @param vec the vector on which to store messages
 * @param veclen the number of messages that vec can hold
 *
 * @returns
 *     the number of messages received, if the operation is successful
 *     -1 if a system error occurs, in which case errno should be set
 *     -2 if vec is not large enough to hold all received messages
 *     -3 if any message includes a non-IPv4 address
 *     -4 if any message includes more than one address for
 *     a single interface
 */
int readmsgs(
	struct iovec *bufv,
	size_t buflen,
	struct ifmodmsg *vec,
	size_t veclen
) {
	struct nlmsghdr *it = (struct nlmsghdr*)bufv;
	int msgs = 0;

	while (NLMSG_OK(it, buflen)) {
		if (it->nlmsg_type == NLMSG_ERROR) {
			return -1;
		} else if (msgs == veclen) {
			return -2;
		} else if (it->nlmsg_type == IPADD || it->nlmsg_type == IPRMV) {
			struct ifaddrmsg *addr = NLMSG_DATA(it);

			struct rtattr *rta = (struct rtattr *)(addr + 1);
			int rtalen = rta->rta_len;

			if (rta->rta_type != IFA_ADDRESS) {
				return -3;
			}

			vec[msgs].devidx = addr->ifa_index;
			vec[msgs].modop = it->nlmsg_type;
			vec[msgs].ipv4 = *((unsigned*)RTA_DATA(rta));

			RTA_NEXT(rta, rtalen);
			if (RTA_OK(rta, rtalen)) {
				return -4;
			}

			msgs++;
		}

		NLMSG_NEXT(it, buflen);
	}

	return msgs;
}

/**
 * Receives messages from the socket,
 * storing them in vec.
 *
 * @param sock the socket to read from
 * @param vec the vector on which to store the messages
 * @param veclen the number of messages that vec can hold
 *
 * @returns the same values as readmsgs()
 */
int nlsock_recv(
	struct nlsock *sock,
	struct ifmodmsg *vec,
	size_t veclen
) {
	ssize_t buflen = recvmsg(sock->fd, &sock->bufh, 0);

	if (buflen < 0) {
		if (errno == EWOULDBLOCK) {
			return 0;
		} else {
			return -1;
		}
	}

	return readmsgs(sock->bufv.iov_base, buflen, vec, veclen);
}
