#include <stdio.h>
#include <stdint.h>

void std_io_print(int64_t val) {
    printf("%ld\n", val);
}

void std_io_print_str(const char* ptr) {
    printf("%s\n", ptr);
}
