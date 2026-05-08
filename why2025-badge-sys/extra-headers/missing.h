// These defines are referenced in symbols.yml, but not in the headers (or the
// repository). They are probably exported from espidf, but I couldn't find
// headers there either. For now I just guessed/googled the signatures

/** The signature seems to be like this. idk why it's not in our headers */
extern int diprintf(int a, const char *b, ...);
/** The signature seems to be like lgamma_r. I have no idea what it does */
extern double gamma_r(double, int *);
/** The signature seems to be like lgammaf_r. I have no idea what it does */
extern float gammaf_r(float, int *);
/** Probably in symbols, because it's in newlib. idk why it is not in our
 * headers */
extern int sig2str(int signum, char *str);
/** Probably in symbols, because it's in newlib. idk why it is not in our
 * headers */
extern int str2sig(const char *restrict str, int *restrict pnum);
// The upstream manifest lists `_ctype_`, but the public header only exposes it as the macro alias
// `_ctype_b + _CTYPE_OFFSET`. Keep bindgen on `_ctype_b` rather than inventing a standalone host
// declaration for `_ctype_`. If we ever need exact exported names here, do it through selective
// forwarding wrappers, not a blanket libc override.

// Missing for some reason, but included in symbols.yml
extern char *gets(char *s);
extern int open(const char *pathname, int flags, ...);