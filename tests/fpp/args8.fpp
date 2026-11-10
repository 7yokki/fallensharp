fn sum8(a int, b int, c int, d int, e int, f int, g int, h int) -> int:
    return a + b + c + d + e + f + g + h

fn main():
    x int => 1 + fncall.sum8(1, 2, 3, 4, 5, 6, 7, 8) + 2
    if x equal 39:
        stdout("nested ok\n")
    else:
        stdout("nested fail\n")
    end
    return 0
