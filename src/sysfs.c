#include <fcntl.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

#include <sysfs.h>

void sfs_param_init(const char* path, struct sfs_param* param) {
	param->path = path;
	param->fd = open(param->path, O_RDONLY);
	if (param->fd < 0) {
		fprintf(stderr, "FATAL: cannot read %s", param->path);
		exit(1);
	}
}

void sfs_param_destroy(struct sfs_param* param) {
	close(param->fd);
}

int sfs_read_int(struct sfs_param* param) {
	lseek(param->fd, 0, SEEK_SET);
	if (read(param->fd, &param->vstr, SFS_PARAM_MAX_VAL_SIZE) == -1) {
		fprintf(stderr, "ERROR: failed to read int from %s", param->path);
		return 0;
	}
	param->vint = strtol(param->vstr, NULL, 10);
	return param->vint;
}

char sfs_read_char(struct sfs_param* param) {
	lseek(param->fd, 0, SEEK_SET);
	if (read(param->fd, &param->vchar, 1) == -1) {
		fprintf(stderr, "ERROR: failed to read char from %s", param->path);
		return '\0';
	}
	return param->vchar;
}
