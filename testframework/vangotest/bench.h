#include <chrono>


#define VANGO_BENCH_WARMUP 100
#define vango_bench(runs, func) do { \
        int64_t _vango_bench_t_total = 0; \
        for (size_t _vango_i = 0; _vango_i < VANGO_BENCH_WARMUP; _vango_i++) { \
            func(); \
        } \
        for (size_t _vango_i = 0; _vango_i < runs; _vango_i++) { \
            const auto _vango_bench_t1 = std::chrono::high_resolution_clock::now(); \
            func(); \
            const auto _vango_bench_t2 = std::chrono::high_resolution_clock::now(); \
            _vango_bench_t_total += std::chrono::duration_cast<std::chrono::microseconds>(_vango_bench_t2 - _vango_bench_t1).count(); \
        } \
        printf("benchmark in '%s' took an average of %lld microseconds over %d runs\n", __func__, _vango_bench_t_total / runs, runs); \
    } while (0)

