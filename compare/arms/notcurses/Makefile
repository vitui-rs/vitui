# The notcurses arm of the vitui comparative suite.
#
#   make               build ./arm
#   make check-headers say whether this machine can build it, and why not
#   make clean
#
# Links notcurses-core, not notcurses: the arm uses notcurses_core_init() and
# needs nothing from the multimedia half, and notcurses-core drags in neither
# ffmpeg nor OpenImageIO.  On Debian and Ubuntu that is libnotcurses-core-dev;
# see NOTES.md.

CC      ?= cc
PKGCONF ?= pkg-config

# notcurses-core first; fall back to the full notcurses module if only that is
# installed (Homebrew ships both, some distributions only the latter).
NC_MODULE := $(shell $(PKGCONF) --exists notcurses-core 2>/dev/null && echo notcurses-core || echo notcurses)

NC_CFLAGS := $(shell $(PKGCONF) --cflags $(NC_MODULE) 2>/dev/null)
NC_LIBS   := $(shell $(PKGCONF) --libs   $(NC_MODULE) 2>/dev/null)
NC_VERSION := $(shell $(PKGCONF) --modversion $(NC_MODULE) 2>/dev/null)

CFLAGS  ?= -O2 -std=c11 -Wall -Wextra
LDFLAGS ?=

.PHONY: all check-headers clean

all: arm

arm: arm.c
	@if [ -z "$(NC_VERSION)" ]; then \
	  echo "make: cannot build the notcurses arm on this machine." >&2; \
	  echo >&2; \
	  echo "  pkg-config found neither 'notcurses-core' nor 'notcurses'." >&2; \
	  echo "  This arm is written but unbuilt here on purpose -- see NOTES.md." >&2; \
	  echo >&2; \
	  echo "  Debian / Ubuntu:  apt-get install libnotcurses-core-dev" >&2; \
	  echo "  Fedora:           dnf install notcurses-devel" >&2; \
	  echo "  Homebrew:         brew install notcurses   (pulls ffmpeg; ~hundreds of MB)" >&2; \
	  echo >&2; \
	  echo "  Run 'make check-headers' for the same answer with more detail." >&2; \
	  exit 1; \
	fi
	$(CC) $(CFLAGS) $(NC_CFLAGS) -o $@ arm.c $(LDFLAGS) $(NC_LIBS)

# Report whether this machine can build the arm, without building it and
# without installing anything.  Exits 0 if it can, 1 if it cannot.
check-headers:
	@echo "pkg-config:      $$($(PKGCONF) --version 2>/dev/null || echo 'NOT FOUND')"
	@echo "notcurses-core:  $$($(PKGCONF) --modversion notcurses-core 2>/dev/null || echo 'not installed')"
	@echo "notcurses:       $$($(PKGCONF) --modversion notcurses 2>/dev/null || echo 'not installed')"
	@echo "module chosen:   $(NC_MODULE)"
	@echo "cflags:          $(NC_CFLAGS)"
	@echo "libs:            $(NC_LIBS)"
	@if [ -z "$(NC_VERSION)" ]; then \
	  echo; \
	  echo "RESULT: cannot build here -- no notcurses development files."; \
	  echo "        The arm is written against 3.0.17 and unbuilt on this machine;"; \
	  echo "        NOTES.md says why and names the package CI should install."; \
	  exit 1; \
	fi
	@printf '#include <notcurses/notcurses.h>\nint main(void){ return notcurses_version() == 0; }\n' > .hdrprobe.c
	@if $(CC) $(CFLAGS) $(NC_CFLAGS) -fsyntax-only .hdrprobe.c 2>/dev/null; then \
	  echo; \
	  echo "RESULT: buildable -- notcurses $(NC_VERSION) headers compile."; \
	  rm -f .hdrprobe.c; \
	else \
	  echo; \
	  echo "RESULT: pkg-config reports notcurses $(NC_VERSION) but its headers do not compile."; \
	  rm -f .hdrprobe.c; \
	  exit 1; \
	fi

clean:
	rm -f arm .hdrprobe.c
	rm -rf arm.dSYM
