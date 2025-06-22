#pragma once

#define SFS_PARAM_MAX_PATH_SIZE 128
#define SFS_PARAM_MAX_VAL_SIZE 16

struct sfs_param {
	int fd;
	char path[SFS_PARAM_MAX_PATH_SIZE];
	union {
		char vstr[SFS_PARAM_MAX_VAL_SIZE];
		int vint;
		char vchar;
	};
};

void sfs_param_init(const char*, struct sfs_param*);
void sfs_param_destroy(struct sfs_param*);

int sfs_read_int(struct sfs_param*);
char sfs_read_char(struct sfs_param*);
