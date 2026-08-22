// The notcurses arm of the vitui comparative suite.
//
// Contract: ../../ARM-CONTRACT.md.  Scenes: ../../SCENES.md.  Everything this
// file decides that those two documents did not is written down in NOTES.md,
// including the places where the scene definitions did not survive contact with
// notcurses.  Read NOTES.md before changing anything here; several of the odder
// decisions below are the only way this arm runs at all.
//
// Targeted against notcurses 3.0.17.  Every call site marked UNVERIFIED is one
// this file's author could not compile on the machine it was written on; see
// NOTES.md, "written but unbuilt".

#define _POSIX_C_SOURCE 200809L
#define _DEFAULT_SOURCE

#include <errno.h>
#include <fcntl.h>
#include <langinfo.h>
#include <locale.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/wait.h>
#include <time.h>
#include <unistd.h>

#include <notcurses/notcurses.h>
#if defined(__has_include)
#  if __has_include(<notcurses/version.h>)
#    include <notcurses/version.h>
#  endif
#endif

// The arm uses `modifiers`/NCKEY_MOD_CTRL and ncinput_ctrl_p(), both of which
// exist from 3.0.7 onward.  Refuse anything older loudly rather than compiling
// against a header that silently means something else.
#if defined(NOTCURSES_VERNUM_ORDERED) && defined(NOTCURSES_VERSION_COMPARABLE)
#  if NOTCURSES_VERNUM_ORDERED < NOTCURSES_VERSION_COMPARABLE(3, 0, 7)
#    error "this arm needs notcurses >= 3.0.7"
#  endif
#endif

// ---------------------------------------------------------------------------
// exit statuses
// ---------------------------------------------------------------------------

// These are the sysexits.h values, spelled out here so the file has no
// dependency on a header that is not on every platform.
#define ARM_OK              0
#define ARM_USAGE          64  // EX_USAGE: bad arguments
#define ARM_CANNOT_EXPRESS 69  // EX_UNAVAILABLE: the scene is beyond this arm.
                               // Paired with the exact bytes "cannot express\n"
                               // on stderr.  The harness must render this as a
                               // "cannot express" cell, never as a blank and
                               // never as a missing row.
#define ARM_SOFTWARE       70  // EX_SOFTWARE: a notcurses call failed
#define ARM_ENV            71  // EX_OSERR: the environment cannot host this arm
                               // (no UTF-8 locale; a controlling terminal that
                               // could not be shed -- see shed_controlling_tty)

// ---------------------------------------------------------------------------
// scene content
// ---------------------------------------------------------------------------

#define SCENE_COLS_DEFAULT 120
#define SCENE_ROWS_DEFAULT  40
#define FRAMES_DEFAULT     120
#define CPU_SECONDS_DEFAULT 10

// Scene 1's title.  The dash is U+2014 EM DASH, so this string is 23 bytes and
// 21 columns wide, and the caret therefore sits at column 21.
static const char SCENE1_TITLE[] = "vitui compare \xe2\x80\x94 caret";
#define SCENE1_TITLE_COLS 21

// Scene 2's lorem: exactly 64 characters, all ASCII.  SCENES.md does not say
// which 64 characters, so this arm picks these and NOTES.md says so; the byte
// counts only depend on the length.
static const char LOREM64[] =
  "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do.";

// Scene 5's spinner, one step per frame.
static const char* const SPINNER[10] = {
  "\xe2\xa0\x8b", "\xe2\xa0\x99", "\xe2\xa0\xb9", "\xe2\xa0\xb8", "\xe2\xa0\xbc",
  "\xe2\xa0\xb4", "\xe2\xa0\xa6", "\xe2\xa0\xa7", "\xe2\xa0\x87", "\xe2\xa0\x8f",
};

// Scene 3's highlighted list index at frame 0, and hence the screen row the
// highlight never leaves.
#define HIGHLIGHT_ROW 12

// "Reverse video" is not expressible in notcurses: it has no NCSTYLE_REVERSE
// (notcurses.h: "if you want reverse video, try ncchannels_reverse()"), and
// ncchannels_reverse() on a pair of *default* channels swaps default for
// default and changes nothing.  So the reversed cells name concrete colours.
// See NOTES.md, "reverse video".
#define REVERSE_FG_R 0x00
#define REVERSE_FG_G 0x00
#define REVERSE_FG_B 0x00
#define REVERSE_BG_R 0xff
#define REVERSE_BG_G 0xff
#define REVERSE_BG_B 0xff

typedef enum {
  SCENE_CARET,
  SCENE_STATUS_LINE,
  SCENE_LIST_SCROLL,
  SCENE_FULL_REPAINT,
  SCENE_MODAL_OVER_LIST,
  SCENE_MODAL_OVER_LIST_BLEND, // not a suite scene; see NOTES.md
  SCENE_LATENCY,
  SCENE_CPU,
} scene_e;

// ---------------------------------------------------------------------------
// getting notcurses to run with no tty at all
// ---------------------------------------------------------------------------
//
// notcurses does not merely prefer a tty.  When the FILE* it is handed is not
// one, get_tty_fd() (src/lib/fd.c) falls back to open("/dev/tty"), and if that
// succeeds notcurses will
//
//   * take the geometry from TIOCGWINSZ on that tty rather than from
//     COLUMNS/LINES, which breaks the fixed 120x40 the contract guarantees,
//   * send its capability queries to whatever terminal the developer happens to
//     be sitting in, making the emitted bytes a function of that terminal,
//   * put that terminal into cbreak mode and scribble on it, and
//   * block in interrogate_terminfo() -> handle_responses() ->
//     inputlayer_get_responses() waiting for a Device Attributes reply.  On a
//     pipe-only run inside a terminal the reply does come back, so this hangs
//     only intermittently -- which is worse.
//
// With no controlling terminal, ttyfd stays -1 and every one of those paths is
// skipped: no queries are sent, no responses are awaited, and
// update_term_dimensions() reads tcache->default_rows/default_cols, which
// get_default_geometry() (src/lib/termdesc.c) fills from the LINES and COLUMNS
// environment variables.  That is exactly the contract's size source, by
// accident rather than by design, and it is the configuration this arm requires.
//
// So: shed the controlling terminal before touching notcurses.  setsid() alone
// is enough when we are not a process-group leader (the usual case under a
// harness that spawns us with posix_spawn/subprocess); when we are one (the
// usual case under an interactive shell with job control) we fork and let the
// child do it, and the parent becomes a transparent status relay.
//
// TIOCNOTTY was considered and rejected: its behaviour for a non-session-leader
// differs between kernels, and it is not in POSIX.

static int
have_controlling_tty(void){
  int fd = open("/dev/tty", O_RDWR | O_NOCTTY | O_CLOEXEC);
  if(fd < 0){
    return 0;
  }
  close(fd);
  return 1;
}

// Returns 0 in the process that should go on to render.  Never returns in a
// parent that forked (it _exit()s with the child's status).  Returns -1 if the
// controlling terminal could not be shed.
static int
shed_controlling_tty(void){
  if(!have_controlling_tty()){
    return 0;
  }
  if(setsid() < 0){
    // We are a process-group leader.  Fork; the child is not.
    fflush(NULL); // nothing has been written yet, but do not rely on that
    pid_t kid = fork();
    if(kid < 0){
      return -1;
    }
    if(kid > 0){
      int wstatus = 0;
      while(waitpid(kid, &wstatus, 0) < 0){
        if(errno != EINTR){
          _exit(ARM_SOFTWARE);
        }
      }
      if(WIFEXITED(wstatus)){
        _exit(WEXITSTATUS(wstatus));
      }
      if(WIFSIGNALED(wstatus)){
        _exit(128 + WTERMSIG(wstatus));
      }
      _exit(ARM_SOFTWARE);
    }
    if(setsid() < 0){
      return -1;
    }
  }
  if(have_controlling_tty()){
    return -1;
  }
  return 0;
}

// ---------------------------------------------------------------------------
// locale
// ---------------------------------------------------------------------------
//
// notcurses refuses to start on a locale that is neither UTF-8 nor ASCII, and
// with an ASCII locale it will not render scene 1's em dash or scene 5's braille
// spinner.  Insist on UTF-8 and say so plainly if there is none.

static int
insist_on_utf8(void){
  static const char* const candidates[] = { "", "C.UTF-8", "en_US.UTF-8", NULL };
  for(size_t i = 0 ; candidates[i] ; ++i){
    if(setlocale(LC_ALL, candidates[i]) == NULL){
      continue;
    }
    const char* cs = nl_langinfo(CODESET);
    if(cs && (strcmp(cs, "UTF-8") == 0 || strcmp(cs, "utf8") == 0 ||
              strcmp(cs, "UTF8") == 0 || strcmp(cs, "utf-8") == 0)){
      return 0;
    }
  }
  return -1;
}

// ---------------------------------------------------------------------------
// colour helpers
// ---------------------------------------------------------------------------

static void
set_default_colours(struct ncplane* n){
  ncplane_set_styles(n, NCSTYLE_NONE);
  ncplane_set_fg_default(n);
  ncplane_set_bg_default(n);
}

static void
set_reverse_colours(struct ncplane* n){
  ncplane_set_styles(n, NCSTYLE_NONE);
  ncplane_set_fg_rgb8(n, REVERSE_FG_R, REVERSE_FG_G, REVERSE_FG_B);
  ncplane_set_bg_rgb8(n, REVERSE_BG_R, REVERSE_BG_G, REVERSE_BG_B);
}

// Write |s| at (y, 0) and pad with spaces out to |cols| in whatever channels the
// plane currently carries.  Used for the full-width status line.
static int
put_row_padded(struct ncplane* n, int y, const char* s, unsigned cols){
  char buf[1024];
  if((size_t)cols + 1 > sizeof(buf)){
    return -1;
  }
  size_t len = strlen(s);
  if(len > (size_t)cols){
    len = cols;
  }
  memcpy(buf, s, len);
  memset(buf + len, ' ', cols - len);
  buf[cols] = '\0';
  return ncplane_putstr_yx(n, y, 0, buf) < 0 ? -1 : 0;
}

// ---------------------------------------------------------------------------
// the status line, shared by scene 2 and the latency scene
// ---------------------------------------------------------------------------
//
// SCENES.md gives the row as a literal:
//
//     " frame 0     elapsed 0.00s    cpu 12%    3 tasks"
//
// and then says the cpu and tasks fields never change.  Those two statements
// only hold together if the frame number sits in a fixed-width field -- with a
// literal substitution "frame 119" would shove the later fields two columns
// right, which is both a change to them and a fifteenfold change to the
// measurement, because the whole 120-cell row would differ every frame instead
// of two or three cells.  So field one is nine columns wide, which reproduces
// the quoted row exactly at frame 0.  NOTES.md, "the status line's first field".
static void
format_status(char* out, size_t outlen, const char* field1, double elapsed){
  snprintf(out, outlen, " %-9s   elapsed %.2fs    cpu 12%%    3 tasks",
           field1, elapsed);
}

static int
draw_status_row(struct ncplane* n, unsigned rows, unsigned cols,
                const char* field1, double elapsed){
  char status[256];
  format_status(status, sizeof(status), field1, elapsed);
  set_reverse_colours(n);
  return put_row_padded(n, (int)rows - 1, status, cols);
}

// Rows 0..rows-2 of scene 2, which never change after frame 0.
static int
draw_status_body(struct ncplane* n, unsigned rows, unsigned cols){
  set_default_colours(n);
  char buf[256];
  for(unsigned y = 0 ; y + 1 < rows ; ++y){
    snprintf(buf, sizeof(buf), "line %02u  %s", y, LOREM64);
    // A put that runs off the right edge of a non-scrolling plane is an error.
    if(cols < sizeof(buf) && (size_t)cols < strlen(buf)){
      buf[cols] = '\0';
    }
    if(ncplane_putstr_yx(n, (int)y, 0, buf) < 0){
      return -1;
    }
  }
  return 0;
}

// ---------------------------------------------------------------------------
// scene 3's rows, shared with the layered demonstration scene
// ---------------------------------------------------------------------------

// Draw the 40 visible rows for a window whose top row is list index |top|, with
// the highlight on screen row HIGHLIGHT_ROW.
//
// This is the plain redraw: forty rows written every frame, with notcurses'
// own damage comparison deciding what reaches the wire.  notcurses also has a
// genuine scroll idiom -- ncplane_set_scrolling() plus ncplane_scrollup(), which
// src/lib/render.c honours through scroll_lastframe() and rasterize_scrolls() --
// and it would cut this scene's wire cost by most of an order of magnitude.
// This arm does not use it, so **notcurses' list-scroll figure is an upper
// bound, not its floor**, and the report has to say so or it understates
// notcurses exactly the way a missing row would overstate it.  NOTES.md,
// "list-scroll is an upper bound".
static int
draw_list(struct ncplane* n, unsigned rows, unsigned cols, unsigned top){
  char buf[128];
  for(unsigned y = 0 ; y < rows ; ++y){
    unsigned idx = top + y;
    // SCENES.md's template is `NNNNN  ` + `item-NNNNN---------`, which is 26
    // columns, not the 20-character label the prose claims; the template wins
    // here.  NOTES.md, "the list label is 19 characters, not 20".
    snprintf(buf, sizeof(buf), "%05u  item-%05u---------", idx, idx);
    if(cols < sizeof(buf) && (size_t)cols < strlen(buf)){
      buf[cols] = '\0';
    }
    if(y == HIGHLIGHT_ROW){
      set_reverse_colours(n);
    }else{
      set_default_colours(n);
    }
    if(ncplane_putstr_yx(n, (int)y, 0, buf) < 0){
      return -1;
    }
  }
  return 0;
}

// ---------------------------------------------------------------------------
// the scenes
// ---------------------------------------------------------------------------

static int
scene_caret(struct notcurses* nc, struct ncplane* n, unsigned rows,
            unsigned cols, uint64_t frames){
  (void)rows;
  (void)cols;
  // The caret is drawn as a cell, not as the terminal's hardware cursor.
  // notcurses cannot hide the hardware cursor here: notcurses_cursor_enable()
  // writes through nc->ttyfp, but notcurses_cursor_disable() writes through
  // tty_emit(cinvis, nc->tcache.ttyfd) (src/lib/render.c), and ttyfd is -1 by
  // construction, so the disable fails and emits nothing.  A drawn cell is also
  // the closer reading of the scene's own rationale -- "one cell of the four
  // thousand eight hundred changes".  NOTES.md, "the caret is a cell".
  set_default_colours(n);
  if(ncplane_putstr_yx(n, 0, 0, SCENE1_TITLE) < 0){
    return -1;
  }
  for(uint64_t f = 0 ; f < frames ; ++f){
    if(f % 2 == 0){
      set_reverse_colours(n);
    }else{
      set_default_colours(n);
    }
    if(ncplane_putchar_yx(n, 0, SCENE1_TITLE_COLS, ' ') < 0){
      return -1;
    }
    if(notcurses_render(nc)){
      return -1;
    }
  }
  return 0;
}

static int
scene_status_line(struct notcurses* nc, struct ncplane* n, unsigned rows,
                  unsigned cols, uint64_t frames){
  if(draw_status_body(n, rows, cols)){
    return -1;
  }
  for(uint64_t f = 0 ; f < frames ; ++f){
    char field1[32];
    snprintf(field1, sizeof(field1), "frame %llu", (unsigned long long)f);
    if(draw_status_row(n, rows, cols, field1, (double)f / 60.0)){
      return -1;
    }
    if(notcurses_render(nc)){
      return -1;
    }
  }
  return 0;
}

static int
scene_list_scroll(struct notcurses* nc, struct ncplane* n, unsigned rows,
                  unsigned cols, uint64_t frames){
  for(uint64_t f = 0 ; f < frames ; ++f){
    if(draw_list(n, rows, cols, (unsigned)f)){
      return -1;
    }
    if(notcurses_render(nc)){
      return -1;
    }
  }
  return 0;
}

static int
scene_full_repaint(struct notcurses* nc, struct ncplane* n, unsigned rows,
                   unsigned cols, uint64_t frames){
  // "the cell at column c, row r takes rgb(c * 2, r * 6, 128)" -- taken as the
  // foreground of the '#', with the background left at the terminal default.
  // SCENES.md does not say which channel; NOTES.md, "which channel scene 4
  // colours".
  ncplane_set_styles(n, NCSTYLE_NONE);
  ncplane_set_bg_default(n);
  for(uint64_t f = 0 ; f < frames ; ++f){
    for(unsigned y = 0 ; y < rows ; ++y){
      for(unsigned x = 0 ; x < cols ; ++x){
        unsigned src = (unsigned)((x + f) % cols);
        ncplane_set_fg_rgb8(n, (src * 2) & 0xff, (y * 6) & 0xff, 128);
        if(ncplane_putchar_yx(n, (int)y, (int)x, '#') < 0){
          return -1;
        }
      }
    }
    if(notcurses_render(nc)){
      return -1;
    }
  }
  return 0;
}

// The layered scene, reached with notcurses' compositor rather than refused.
//
// This is NOT one of the suite's five scenes and the harness must not run it as
// `modal-over-list`; `--scene modal-over-list` refuses, per SCENES.md.  It
// exists because the ticket's premise -- that notcurses has no operator mixing a
// plane's colours toward another colour -- is wrong, and a claim that specific
// deserves a demonstration rather than a paragraph.  NOTES.md, "scene 5: the
// ticket is wrong".
//
// Mechanism: three planes.  The dim plane sits above the list and below the
// dialog, its base cell has gcluster 0 (so it contributes no glyph and the
// list's glyphs show through) and both its channels are rgb(0,0,0) with
// NCALPHA_BLEND.  ncpile_render_internal() walks the pile top to bottom;
// channels_blend() takes the running mean (r1 * blends + r2) / (blends + 1), so
// the dim plane -- reached first, with blends == 0 -- installs black and leaves
// the target non-opaque, and the list plane -- reached second, with blends == 1
// -- lands at exactly r2 / 2.  Halfway toward black, computed by the compositor,
// with the caller naming no dimmed colour anywhere.
//
// The list here carries explicit colours rather than the terminal defaults that
// scene 3 specifies.  With no tty notcurses never learns the terminal's default
// foreground (there is no OSC 10 reply), so tcache.fg_default stays rgb(0,0,0)
// and dimming a default-coloured cell yields black on black -- correct
// arithmetic, invisible picture.  NOTES.md says so.
static int
scene_modal_blend(struct notcurses* nc, struct ncplane* stdp, unsigned rows,
                  unsigned cols, uint64_t frames){
  // The list, on the standard plane, in explicit colours.
  char buf[128];
  ncplane_set_styles(stdp, NCSTYLE_NONE);
  for(unsigned y = 0 ; y < rows ; ++y){
    snprintf(buf, sizeof(buf), "%05u  item-%05u---------", y, y);
    if(y == HIGHLIGHT_ROW){
      ncplane_set_fg_rgb8(stdp, 0x00, 0x00, 0x00);
      ncplane_set_bg_rgb8(stdp, 0xff, 0xff, 0xff);
    }else{
      ncplane_set_fg_rgb8(stdp, 0xc0, 0xc0, 0xc0);
      ncplane_set_bg_rgb8(stdp, 0x00, 0x00, 0x00);
    }
    if(ncplane_putstr_yx(stdp, (int)y, 0, buf) < 0){
      return -1;
    }
  }

  // The dim plane.
  struct ncplane_options dopts;
  memset(&dopts, 0, sizeof(dopts));
  dopts.y = 0;
  dopts.x = 0;
  dopts.rows = rows;
  dopts.cols = cols;
  dopts.name = "dim";
  struct ncplane* dim = ncplane_create(stdp, &dopts);
  if(dim == NULL){
    return -1;
  }
  {
    uint64_t chans = 0;
    ncchannels_set_fg_rgb8(&chans, 0, 0, 0);
    ncchannels_set_bg_rgb8(&chans, 0, 0, 0);
    ncchannels_set_fg_alpha(&chans, NCALPHA_BLEND);
    ncchannels_set_bg_alpha(&chans, NCALPHA_BLEND);
    // A base cell whose gcluster is 0 contributes colour but no glyph:
    // paint() substitutes the base cell for the colour of an unwritten cell,
    // and leaves crender->p NULL because targc->gcluster comes out 0, so the
    // glyph search continues into the plane below.  NCCELL_TRIVIAL_INITIALIZER
    // is used rather than ncplane_set_base(dim, "", ...) because an empty EGC
    // goes through utf8_egc_len(), and this way there is nothing to be unsure
    // about.
    nccell base = NCCELL_TRIVIAL_INITIALIZER;
    base.channels = chans;
    if(ncplane_set_base_cell(dim, &base) < 0){
      return -1;
    }
    ncplane_erase(dim); // fb all zeroes: every cell falls through to the base
  }

  // The dialog, 40x12, top-left at (14, 40), opaque.
  const unsigned dh = 12, dw = 40;
  const int dy = 14, dx = 40;
  struct ncplane_options mopts;
  memset(&mopts, 0, sizeof(mopts));
  mopts.y = dy;
  mopts.x = dx;
  mopts.rows = dh;
  mopts.cols = dw;
  mopts.name = "dialog";
  struct ncplane* dlg = ncplane_create(stdp, &mopts);
  if(dlg == NULL){
    return -1;
  }
  // ncplane_create() puts the new plane at the top of the pile, so creating
  // dim and then dlg already leaves the order dlg > dim > stdp.  State it
  // anyway rather than relying on creation order.
  ncplane_move_top(dlg);
  if(ncplane_move_below(dim, dlg)){
    return -1;
  }

  ncplane_set_styles(dlg, NCSTYLE_NONE);
  ncplane_set_fg_rgb8(dlg, 0xe0, 0xe0, 0xe0);
  ncplane_set_bg_rgb8(dlg, 0x20, 0x20, 0x30);
  // The border is written as literal box-drawing text rather than through
  // ncplane_box(), whose six-nccell setup is more API than this needs.
  {
    char top[256], mid[256], bot[256];
    size_t p = 0;
    memcpy(top + p, "\xe2\x94\x8c", 3); p += 3;
    for(unsigned i = 0 ; i + 2 < dw ; ++i){ memcpy(top + p, "\xe2\x94\x80", 3); p += 3; }
    memcpy(top + p, "\xe2\x94\x90", 3); p += 3;
    top[p] = '\0';
    p = 0;
    memcpy(bot + p, "\xe2\x94\x94", 3); p += 3;
    for(unsigned i = 0 ; i + 2 < dw ; ++i){ memcpy(bot + p, "\xe2\x94\x80", 3); p += 3; }
    memcpy(bot + p, "\xe2\x94\x98", 3); p += 3;
    bot[p] = '\0';
    p = 0;
    memcpy(mid + p, "\xe2\x94\x82", 3); p += 3;
    for(unsigned i = 0 ; i + 2 < dw ; ++i){ mid[p++] = ' '; }
    memcpy(mid + p, "\xe2\x94\x82", 3); p += 3;
    mid[p] = '\0';
    if(ncplane_putstr_yx(dlg, 0, 0, top) < 0){
      return -1;
    }
    for(unsigned y = 1 ; y + 1 < dh ; ++y){
      if(ncplane_putstr_yx(dlg, (int)y, 0, mid) < 0){
        return -1;
      }
    }
    if(ncplane_putstr_yx(dlg, (int)dh - 1, 0, bot) < 0){
      return -1;
    }
  }
  static const char* const BODY[4] = {
    " scanning 10000 items",
    " 3 tasks queued",
    " dim is NCALPHA_BLEND, not arithmetic",
    " press Esc to cancel",
  };
  for(unsigned i = 0 ; i < 4 ; ++i){
    char line[64];
    // The dialog's interior is dw - 2 columns wide, and a put that runs off a
    // non-scrolling plane is an error, so clip rather than trust the literals.
    snprintf(line, sizeof(line), "%.*s", (int)(dw - 2), BODY[i]);
    if(ncplane_putstr_yx(dlg, (int)(3 + i), 1, line) < 0){
      return -1;
    }
  }

  for(uint64_t f = 0 ; f < frames ; ++f){
    char title[64];
    snprintf(title, sizeof(title), "%s working", SPINNER[f % 10]);
    if(ncplane_putstr_yx(dlg, 1, 2, title) < 0){
      return -1;
    }
    if(notcurses_render(nc)){
      return -1;
    }
  }
  return 0;
}

// ---------------------------------------------------------------------------
// latency
// ---------------------------------------------------------------------------

static void
key_name(const ncinput* ni, char* out, size_t outlen){
  switch(ni->id){
    case NCKEY_UP:    snprintf(out, outlen, "Up");    return;
    case NCKEY_DOWN:  snprintf(out, outlen, "Down");  return;
    case NCKEY_LEFT:  snprintf(out, outlen, "Left");  return;
    case NCKEY_RIGHT: snprintf(out, outlen, "Right"); return;
    case NCKEY_ENTER: snprintf(out, outlen, "Enter"); return;
    case NCKEY_ESC:   snprintf(out, outlen, "Esc");   return;
    case NCKEY_TAB:   snprintf(out, outlen, "Tab");   return;
    default: break;
  }
  if(ncinput_ctrl_p(ni) && ni->id >= 'A' && ni->id <= 'Z'){
    // notcurses normalises a 0x01..0x1a byte to the uppercase letter plus
    // NCKEY_MOD_CTRL (src/lib/in.c), so 0x03 arrives as 'C' + ctrl.
    snprintf(out, outlen, "C-%c", (char)('a' + (ni->id - 'A')));
    return;
  }
  if(ni->id < 0x80 && ni->id > 0x20){
    snprintf(out, outlen, "%c", (char)ni->id);
    return;
  }
  snprintf(out, outlen, "0x%x", (unsigned)ni->id);
}

static int
scene_latency(struct notcurses* nc, struct ncplane* n, unsigned rows,
              unsigned cols){
  if(draw_status_body(n, rows, cols)){
    return -1;
  }
  if(draw_status_row(n, rows, cols, "frame 0", 0.0)){
    return -1;
  }
  if(notcurses_render(nc)){
    return -1;
  }
  for(;;){
    ncinput ni;
    memset(&ni, 0, sizeof(ni));
    uint32_t id = notcurses_get_blocking(nc, &ni);
    if(id == (uint32_t)-1 || id == NCKEY_EOF || id == 0){
      break;
    }
    if(id == NCKEY_RESIZE){
      continue;
    }
    if(ni.evtype == NCTYPE_RELEASE){
      continue; // a release is not a keystroke
    }
    char name[32], field1[64];
    key_name(&ni, name, sizeof(name));
    snprintf(field1, sizeof(field1), "key %s", name);
    if(draw_status_row(n, rows, cols, field1, 0.0)){
      return -1;
    }
    // notcurses_render() write(2)s straight to the output fd, so there is
    // nothing left buffered after it returns and no flush to add.
    if(notcurses_render(nc)){
      return -1;
    }
  }
  return 0;
}

// ---------------------------------------------------------------------------
// cpu
// ---------------------------------------------------------------------------

static void
timespec_add_ns(struct timespec* ts, long long ns){
  ts->tv_nsec += ns;
  while(ts->tv_nsec >= 1000000000L){
    ts->tv_nsec -= 1000000000L;
    ts->tv_sec += 1;
  }
}

static double
timespec_diff(const struct timespec* a, const struct timespec* b){
  return (double)(a->tv_sec - b->tv_sec) +
         (double)(a->tv_nsec - b->tv_nsec) / 1e9;
}

// Sleep until the absolute CLOCK_MONOTONIC deadline |until|.  macOS has no
// clock_nanosleep(), so fall back to a relative nanosleep(2) there; the cpu
// scene is the only caller and a few microseconds of drift in its pacing does
// not move the number it reports.
static void
sleep_until(const struct timespec* until){
#if defined(TIMER_ABSTIME) && !defined(__APPLE__)
  while(clock_nanosleep(CLOCK_MONOTONIC, TIMER_ABSTIME, until, NULL) == EINTR){
    // keep waiting out the interval
  }
#else
  struct timespec now;
  if(clock_gettime(CLOCK_MONOTONIC, &now)){
    return;
  }
  double d = timespec_diff(until, &now);
  if(d <= 0){
    return;
  }
  struct timespec rel;
  rel.tv_sec = (time_t)d;
  rel.tv_nsec = (long)((d - (double)rel.tv_sec) * 1e9);
  while(nanosleep(&rel, &rel) < 0 && errno == EINTR){
    // keep waiting out the interval
  }
#endif
}

static int
scene_cpu(struct notcurses* nc, struct ncplane* n, unsigned rows, unsigned cols,
          double seconds){
  (void)rows;
  (void)cols;
  set_default_colours(n);
  if(ncplane_putstr_yx(n, 0, 0, SCENE1_TITLE) < 0){
    return -1;
  }
  struct timespec start, deadline, now;
  if(clock_gettime(CLOCK_MONOTONIC, &start)){
    return -1;
  }
  deadline = start;
  for(uint64_t f = 0 ; ; ++f){
    if(f % 2 == 0){
      set_reverse_colours(n);
    }else{
      set_default_colours(n);
    }
    if(ncplane_putchar_yx(n, 0, SCENE1_TITLE_COLS, ' ') < 0){
      return -1;
    }
    if(notcurses_render(nc)){
      return -1;
    }
    // The clock is read only to pace; frame f's picture is still a function of
    // f alone, as the contract requires.
    timespec_add_ns(&deadline, 1000000000LL / 60);
    if(clock_gettime(CLOCK_MONOTONIC, &now)){
      return -1;
    }
    if(timespec_diff(&now, &start) >= seconds){
      break;
    }
    if(timespec_diff(&deadline, &now) > 0){
      sleep_until(&deadline);
    }else{
      deadline = now; // we are behind 60 Hz; do not accumulate debt
    }
  }
  return 0;
}

// ---------------------------------------------------------------------------
// main
// ---------------------------------------------------------------------------

static void
usage(FILE* fp){
  fprintf(fp,
    "usage: arm --scene <name> [--frames N] [--seconds S]\n"
    "  scenes: caret status-line list-scroll full-repaint modal-over-list\n"
    "          latency cpu\n"
    "          modal-over-list-blend (not a suite scene; see NOTES.md)\n");
}

static int
parse_scene(const char* s, scene_e* out){
  if(strcmp(s, "caret") == 0){ *out = SCENE_CARET; return 0; }
  if(strcmp(s, "status-line") == 0){ *out = SCENE_STATUS_LINE; return 0; }
  if(strcmp(s, "list-scroll") == 0){ *out = SCENE_LIST_SCROLL; return 0; }
  if(strcmp(s, "full-repaint") == 0){ *out = SCENE_FULL_REPAINT; return 0; }
  if(strcmp(s, "modal-over-list") == 0){ *out = SCENE_MODAL_OVER_LIST; return 0; }
  if(strcmp(s, "modal-over-list-blend") == 0){
    *out = SCENE_MODAL_OVER_LIST_BLEND; return 0;
  }
  if(strcmp(s, "latency") == 0){ *out = SCENE_LATENCY; return 0; }
  if(strcmp(s, "cpu") == 0){ *out = SCENE_CPU; return 0; }
  return -1;
}

int main(int argc, char** argv){
  scene_e scene = SCENE_CARET;
  int have_scene = 0;
  uint64_t frames = FRAMES_DEFAULT;
  double seconds = CPU_SECONDS_DEFAULT;

  for(int i = 1 ; i < argc ; ++i){
    if(strcmp(argv[i], "--scene") == 0 && i + 1 < argc){
      if(parse_scene(argv[++i], &scene)){
        fprintf(stderr, "unknown scene: %s\n", argv[i]);
        return ARM_USAGE;
      }
      have_scene = 1;
    }else if(strcmp(argv[i], "--frames") == 0 && i + 1 < argc){
      frames = strtoull(argv[++i], NULL, 10);
    }else if(strcmp(argv[i], "--seconds") == 0 && i + 1 < argc){
      seconds = strtod(argv[++i], NULL);
    }else if(strcmp(argv[i], "--help") == 0){
      usage(stdout);
      return ARM_OK;
    }else{
      fprintf(stderr, "unexpected argument: %s\n", argv[i]);
      usage(stderr);
      return ARM_USAGE;
    }
  }
  if(!have_scene){
    usage(stderr);
    return ARM_USAGE;
  }

  // The refusal comes before anything else: no notcurses, no locale, no fork,
  // and above all no output on stdout.  The identity line still goes out so the
  // harness can record the version it refused at.
  if(scene == SCENE_MODAL_OVER_LIST){
    fprintf(stderr, "arm=notcurses version=%s no_color=unsupported alt_screen=no\n",
            notcurses_version());
    fprintf(stderr, "cannot express\n");
    return ARM_CANNOT_EXPRESS;
  }

  if(insist_on_utf8()){
    fprintf(stderr, "no UTF-8 locale available; this arm needs one for the "
                    "em dash in scene 1 and the braille spinner in scene 5\n");
    return ARM_ENV;
  }

  if(shed_controlling_tty()){
    fprintf(stderr, "could not shed the controlling terminal; notcurses would "
                    "open /dev/tty, take its geometry from TIOCGWINSZ instead "
                    "of COLUMNS/LINES, and block awaiting a Device Attributes "
                    "reply -- run this arm with setsid(1)\n");
    return ARM_ENV;
  }

  // TERM alone does not declare 24-bit colour: notcurses' query_rgb()
  // (src/lib/termdesc.c) wants terminfo's RGB/Tc -- which xterm-256color does
  // not carry -- or COLORTERM in {truecolor, 24bit}, and without one it
  // quantizes scene 4's ramp into the 256-colour palette, which is neither the
  // described picture nor a comparable byte count.  Declaring it here, only if
  // the harness has not, is the least bad of the available wrongs; the real fix
  // is for ARM-CONTRACT.md to fix COLORTERM for every arm.  NOTES.md,
  // "COLORTERM is missing from the contract".
  if(getenv("COLORTERM") == NULL){
    setenv("COLORTERM", "truecolor", 0);
  }

  struct notcurses_options opts;
  memset(&opts, 0, sizeof(opts));
  // A zeroed struct means NCLOGLEVEL_PANIC, not SILENT (SILENT is -1), and a
  // panic would land on stderr in breach of the contract's one-line rule.
  opts.loglevel = NCLOGLEVEL_SILENT;
  opts.flags = NCOPTION_SUPPRESS_BANNERS      // the banner goes to stdout
             | NCOPTION_NO_ALTERNATE_SCREEN   // unreachable anyway with no tty
             | NCOPTION_NO_CLEAR_BITMAPS
             | NCOPTION_NO_FONT_CHANGES
             | NCOPTION_NO_WINCH_SIGHANDLER
             | NCOPTION_NO_QUIT_SIGHANDLERS
             | NCOPTION_INHIBIT_SETLOCALE;    // insist_on_utf8() already did it
  if(scene != SCENE_LATENCY){
    // The five picture scenes and cpu never read input; say so, so notcurses
    // does not accumulate it.
    opts.flags |= NCOPTION_DRAIN_INPUT;
  }

  // stdout, deliberately.  notcurses' own ncpile_render_to_buffer() would look
  // like the right call for a pipe, but in 3.0.17 it skips postpaint(), so no
  // cell is ever marked damaged and rasterize_core() emits essentially nothing.
  // The ordinary path is both correct and the one a real notcurses application
  // takes.  NOTES.md, "rendering to a pipe".
  // The identity line goes out before init, so it precedes not just the first
  // frame but notcurses' own prologue on stdout.
  //
  //   no_color=unsupported -- the string NO_COLOR does not occur anywhere in
  //     notcurses 3.0.17.  It is not that notcurses ignores the declaration; it
  //     has no concept of it, which is the contract's third answer.
  //   alt_screen=no -- and not by choice: notcurses_enter_alternate_screen()
  //     returns -1 outright when tcache.ttyfd < 0, which it always is here.
  fprintf(stderr, "arm=notcurses version=%s no_color=unsupported alt_screen=no\n",
          notcurses_version());
  fflush(stderr);

  struct notcurses* nc = notcurses_core_init(&opts, stdout);
  if(nc == NULL){
    fprintf(stderr, "notcurses_core_init failed\n");
    return ARM_SOFTWARE;
  }

  struct ncplane* stdp = notcurses_stdplane(nc);
  unsigned rows, cols;
  ncplane_dim_yx(stdp, &rows, &cols);

  int rc = 0;
  switch(scene){
    case SCENE_CARET:
      rc = scene_caret(nc, stdp, rows, cols, frames);
      break;
    case SCENE_STATUS_LINE:
      rc = scene_status_line(nc, stdp, rows, cols, frames);
      break;
    case SCENE_LIST_SCROLL:
      rc = scene_list_scroll(nc, stdp, rows, cols, frames);
      break;
    case SCENE_FULL_REPAINT:
      rc = scene_full_repaint(nc, stdp, rows, cols, frames);
      break;
    case SCENE_MODAL_OVER_LIST_BLEND:
      rc = scene_modal_blend(nc, stdp, rows, cols, frames);
      break;
    case SCENE_LATENCY:
      rc = scene_latency(nc, stdp, rows, cols);
      break;
    case SCENE_CPU:
      rc = scene_cpu(nc, stdp, rows, cols, seconds);
      break;
    case SCENE_MODAL_OVER_LIST:
      rc = -1; // handled above; unreachable
      break;
  }

  if(notcurses_stop(nc) && rc == 0){
    rc = -1;
  }
  if(rc){
    fprintf(stderr, "scene failed\n");
    return ARM_SOFTWARE;
  }
  return ARM_OK;
}
