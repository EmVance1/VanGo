#include <stdio.h>

#ifdef _WIN32
#include <Windows.h>

#define VANGO_BENCH_WARMUP 100
#define vango_bench(runs, code) do { \
        LARGE_INTEGER _vango_bench_freq, _vango_bench_t1, _vango_bench_t2; \
        QueryPerformanceFrequency(&_vango_bench_freq); \
        double _vango_bench_t_total = 0; \
        for (size_t _vango_i = 0; _vango_i < VANGO_BENCH_WARMUP; _vango_i++) { \
            do code while (0); \
        } \
        for (size_t _vango_i = 0; _vango_i < (runs); _vango_i++) { \
            QueryPerformanceCounter(&_vango_bench_t1); \
            do code while (0); \
            QueryPerformanceCounter(&_vango_bench_t2); \
            _vango_bench_t_total += (double)(_vango_bench_t2.QuadPart - _vango_bench_t1.QuadPart) / _vango_bench_freq.QuadPart; \
        } \
        double _vango_bench_result = _vango_bench_t_total / (runs); \
        if (_vango_bench_result >= 0.1) { \
            printf("benchmark in '%s' took an average of %f seconds over %d runs\n", __func__, _vango_bench_result, runs); \
        } else if (_vango_bench_result >= 0.0001) { \
            printf("benchmark in '%s' took an average of %f milliseconds over %d runs\n", __func__, _vango_bench_result * 1000, runs); \
        } else if (_vango_bench_result >= 0.0000001) { \
            printf("benchmark in '%s' took an average of %f microseconds over %d runs\n", __func__, _vango_bench_result * 1000 * 1000, runs); \
        } \
    } while (0)

#else
#include <time.h>

double _vango_posix_get_clock();

#define VANGO_BENCH_WARMUP 100
#define vango_bench(runs, code) do { \
        double _vango_bench_t_total = 0; \
        for (size_t _vango_i = 0; _vango_i < VANGO_BENCH_WARMUP; _vango_i++) { \
            do code while (0); \
        } \
        for (size_t _vango_i = 0; _vango_i < runs; _vango_i++) { \
            const double _vango_bench_t1 = _vango_posix_get_clock(); \
            do code while (0); \
            const double _vango_bench_t2 = _vango_posix_get_clock(); \
            _vango_bench_t_total += (double)(_vango_bench_t2 - _vango_bench_t1) * 1000; \
        } \
        double _vango_bench_result = _vango_bench_t_total / (runs); \
        if (_vango_bench_result >= 0.1) { \
            printf("benchmark in '%s' took an average of %f seconds over %d runs\n", __func__, _vango_bench_result, runs); \
        } else if (_vango_bench_result >= 0.0001) { \
            printf("benchmark in '%s' took an average of %f milliseconds over %d runs\n", __func__, _vango_bench_result * 1000, runs); \
        } else if (_vango_bench_result >= 0.0000001) { \
            printf("benchmark in '%s' took an average of %f microseconds over %d runs\n", __func__, _vango_bench_result * 1000 * 1000, runs); \
        } \
    } while (0)

#ifdef VANGO_TEST_ROOT
double _vango_posix_get_clock() {
    struct timespec ts; \
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return ts.tv_sec + ts.tv_nsec * 1e-9;
}
#endif

#endif

