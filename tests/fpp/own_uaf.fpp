fn main():
    p ptr int => alloc(8)
    *p = 42
    free(p)
    x int => *p
    return 0
