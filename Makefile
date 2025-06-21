BUILDDIR=build
CC=gcc
CFLAGS=-Wall -Werror -pedantic -std=c23 -O2 -I src/.
WIDGETSDIR=src/widgets
WIDGET_SRCS=$(wildcard $(WIDGETSDIR)/*.c)
WIDGET_OBJS=$(patsubst %.c,$(BUILDDIR)/%.o,$(WIDGET_SRCS))

.PHONY: clean default

default: $(BUILDDIR)/main

clean:
	rm -rf $(BUILDDIR)

$(BUILDDIR)/$(WIDGETSDIR)/%.o: $(WIDGETSDIR)/%.c $(WIDGETSDIR)/%.h $(WIDGETSDIR)/widget.h
	mkdir -p $(BUILDDIR)/$(WIDGETSDIR)
	$(CC) $(CFLAGS) -c -o $@ $<

$(BUILDDIR)/main: src/main.c $(WIDGET_OBJS)
	$(CC) $(CFLAGS) -o $@ $^
