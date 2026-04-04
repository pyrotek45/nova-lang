#!/usr/bin/env python3
"""
Head-to-head Python vs Nova micro-benchmarks.
Each benchmark is designed to be structurally identical to bench_compare.nv.
"""
import time

def now_ms():
    return time.perf_counter_ns() // 1_000_000

results = []

def bench(name, fn):
    t0 = now_ms()
    result = fn()
    t1 = now_ms()
    ms = t1 - t0
    results.append((name, ms))
    print(f"  {name:<40s} {ms:>6d} ms")
    return result

# ═══════════════════════════════════════════════════════════════
# 1. Integer addition loop (1M iterations)
# ═══════════════════════════════════════════════════════════════
def bench_int_add():
    s = 0
    for i in range(1_000_000):
        s = s + i
    assert s == 499999500000
    return s

bench("1  [int_add_1M]", bench_int_add)

# ═══════════════════════════════════════════════════════════════
# 2. Float multiply-accumulate (500k iterations)
# ═══════════════════════════════════════════════════════════════
def bench_float_mul():
    s = 0.0
    for i in range(500_000):
        s = s + float(i) * 0.001
    assert s > 0.0
    return s

bench("2  [float_mul_500k]", bench_float_mul)

# ═══════════════════════════════════════════════════════════════
# 3. Recursive Fibonacci (fib 30)
# ═══════════════════════════════════════════════════════════════
def fib(n):
    if n <= 1:
        return n
    return fib(n - 1) + fib(n - 2)

def bench_fib():
    r = fib(30)
    assert r == 832040
    return r

bench("3  [fib_recursive_30]", bench_fib)

# ═══════════════════════════════════════════════════════════════
# 4. While-loop countdown (2M)
# ═══════════════════════════════════════════════════════════════
def bench_while():
    n = 2_000_000
    s = 0
    while n > 0:
        s = s + n
        n = n - 1
    assert s == 2000001000000
    return s

bench("4  [while_countdown_2M]", bench_while)

# ═══════════════════════════════════════════════════════════════
# 5. Function call overhead (500k calls)
# ═══════════════════════════════════════════════════════════════
def add_one(x):
    return x + 1

def bench_fn_call():
    s = 0
    for i in range(500_000):
        s = add_one(s)
    assert s == 500_000
    return s

bench("5  [fn_call_500k]", bench_fn_call)

# ═══════════════════════════════════════════════════════════════
# 6. List push + iterate (100k push, then sum)
# ═══════════════════════════════════════════════════════════════
def bench_list_ops():
    lst = []
    for i in range(100_000):
        lst.append(i)
    s = 0
    for v in lst:
        s = s + v
    assert s == 4999950000
    return s

bench("6  [list_push_iter_100k]", bench_list_ops)

# ═══════════════════════════════════════════════════════════════
# 7. String concatenation (20k iterations)
# ═══════════════════════════════════════════════════════════════
def bench_string_concat():
    s = ""
    for i in range(20_000):
        s = s + "a"
    assert len(s) == 20_000
    return len(s)

bench("7  [string_concat_20k]", bench_string_concat)

# ═══════════════════════════════════════════════════════════════
# 8. Closure / captured variable (200k calls)
# ═══════════════════════════════════════════════════════════════
def bench_closure():
    offset = 10
    def add_offset(x):
        return x + offset
    s = 0
    for i in range(200_000):
        s = add_offset(s)
    assert s == 2_000_000
    return s

bench("8  [closure_capture_200k]", bench_closure)

# ═══════════════════════════════════════════════════════════════
# 9. Dict (struct) field read/write (200k iterations)
# ═══════════════════════════════════════════════════════════════
def bench_dict_field():
    class Point:
        __slots__ = ('x', 'y')
        def __init__(self, x, y):
            self.x = x
            self.y = y
    p = Point(0, 0)
    for i in range(200_000):
        p.x = p.x + 1
        p.y = p.y + 2
    assert p.x == 200_000
    assert p.y == 400_000
    return p.x

bench("9  [struct_field_rw_200k]", bench_dict_field)

# ═══════════════════════════════════════════════════════════════
# 10. List comprehension / map+filter (10k base, 100 reps)
# ═══════════════════════════════════════════════════════════════
def bench_comprehension():
    total = 0
    for _ in range(100):
        base = list(range(10_000))
        mapped = [x * 2 for x in base]
        filtered = [x for x in mapped if x % 4 == 0]
        total = total + len(filtered)
    assert total == 500_000
    return total

bench("10 [comprehension_100x10k]", bench_comprehension)

# ═══════════════════════════════════════════════════════════════
# 11. Nested loop (matrix multiply 60x60)
# ═══════════════════════════════════════════════════════════════
def bench_matmul():
    N = 60
    A = [0] * (N * N)
    B = [0] * (N * N)
    C = [0] * (N * N)
    for i in range(N):
        for j in range(N):
            A[i * N + j] = i + j
            B[i * N + j] = i - j
    for i in range(N):
        for j in range(N):
            s = 0
            for k in range(N):
                s = s + A[i * N + k] * B[k * N + j]
            C[i * N + j] = s
    return C[0]

bench("11 [matmul_60x60]", bench_matmul)

# ═══════════════════════════════════════════════════════════════
# 12. Sieve of Eratosthenes (50k)
# ═══════════════════════════════════════════════════════════════
def bench_sieve():
    N = 50_000
    is_prime = [True] * N
    is_prime[0] = False
    is_prime[1] = False
    for i in range(2, N):
        if is_prime[i]:
            j = i * i
            while j < N:
                is_prime[j] = False
                j = j + i
    count = sum(1 for x in is_prime if x)
    assert count == 5133
    return count

bench("12 [sieve_50k]", bench_sieve)

# ═══════════════════════════════════════════════════════════════
print()
print("=" * 56)
print("  Python 3.12 Benchmark Results")
print("=" * 56)
total = sum(ms for _, ms in results)
for name, ms in results:
    print(f"  {name:<40s} {ms:>6d} ms")
print(f"  {'TOTAL':<40s} {total:>6d} ms")
