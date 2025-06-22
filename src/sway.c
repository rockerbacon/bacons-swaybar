#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

#include <sway.h>

void sway_exec(char* const file, char* const argv[]) {
	size_t argc = 0;
	if (argv != NULL) {
		while(argv[argc] != NULL) argc++;
	}

	char** exec_args = malloc((argc+4)*sizeof(char**));

	exec_args[0] = "swaymsg";
	exec_args[1] = "exec";
	exec_args[2] = file;
	for (size_t i = 0; i < argc; i++) {
		exec_args[i+3] = argv[i];
	}
	exec_args[argc+3] = NULL;

	if (fork() == 0) {
		execvp("swaymsg", exec_args);
	}
}
