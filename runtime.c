#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>

void std_io_print(int64_t val) {
    printf("%ld\n", val);
}

void std_io_print_str(const char* ptr) {
    printf("%s\n", ptr);
}

void __demir_assert_fail(const char* msg) {
    fprintf(stderr, "ASSERT FAILED: %s\n", msg);
    exit(1);
}
