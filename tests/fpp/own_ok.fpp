fn main():
    p ptr int => alloc(8)
    *p = 42
    x int => *p
    if x equal 42:
        stdout("owned ok\n")
    end
    free(p)
    return 0
