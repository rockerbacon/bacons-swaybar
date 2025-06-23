BUILDDIR=build
CC=gcc
CFLAGS=-D_DEFAULT_SOURCE -Wall -Werror -pedantic -std=c23 -O2 -I src/.
WIDGETSDIR=src/widgets
WIDGET_SRCS=$(wildcard $(WIDGETSDIR)/*.c)
WIDGET_OBJS=$(patsubst %.c,$(BUILDDIR)/%.o,$(WIDGET_SRCS))

.PHONY: clean default install uninstall

default: $(BUILDDIR)/main

clean:
	rm -rf $(BUILDDIR)

$(BUILDDIR)/$(WIDGETSDIR)/%.o: $(WIDGETSDIR)/%.c $(WIDGETSDIR)/%.h $(WIDGETSDIR)/widget.h
	mkdir -p $(BUILDDIR)/$(WIDGETSDIR)
	$(CC) $(CFLAGS) -c -o $@ $<

$(BUILDDIR)/%.o: src/%.c src/%.h
	mkdir -p $(BUILDDIR)
	$(CC) $(CFLAGS) -c -o $@ $<

$(BUILDDIR)/main: src/main.c $(BUILDDIR)/clicks.o $(BUILDDIR)/sighandler.o $(BUILDDIR)/sway.o $(BUILDDIR)/sysfs.o $(WIDGET_OBJS)
	$(CC) $(CFLAGS) -o $@ $^

install: $(BUILDDIR)/main
	mkdir -p ${HOME}/.local/bin
	rm -f ${HOME}/.local/bin/bacons-swaybar
	cp $< ${HOME}/.local/bin/bacons-swaybar

uninstall:
	rm -f ${HOME}/.local/bin/bacons-swaybar
