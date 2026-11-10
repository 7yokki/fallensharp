fn mix(a int, b double) -> double:
    return a + b

fn main():
    x double => 1.5 + 2.5 * 2.0
    print_double(x)
    y double => fncall.mix(2, 3.5)
    print_double(y)
    if 2 less 2.5:
        stdout("cmp ok\n")
    end
    printf("%.1f\n", x)
    return 0
