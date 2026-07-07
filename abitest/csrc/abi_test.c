/* C-side ABI fixtures (built by clang) for the Rust bare-metal ABI tests.
 * Rust calls these; the tests assert the results, verifying that rustc's
 * s1c33 calling convention matches clang's at runtime. */
#include <stdarg.h>

struct P   { int x; int y; };
struct One { int v; };
struct B   { unsigned char b; };

int sum_p(struct P p)     { return p.x + p.y; }   /* multi-field  -> stack       */
int one_val(struct One o) { return o.v; }          /* single 32-bit -> register   */
int b_val(struct B s)     { return s.b; }           /* single 8-bit -> hi-bit pack */

/* variadic: sum `count` int args (caller-side variadic ABI). */
int sum_va(int count, ...) {
    va_list ap; int i, acc = 0;
    va_start(ap, count);
    for (i = 0; i < count; i++) acc += va_arg(ap, int);
    va_end(ap);
    return acc;
}
