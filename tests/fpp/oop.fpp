class Point:
    x int
    y int

    fn Point(a int, b int):
        this.x = a
        this.y = b

    fn sum() -> int:
        return this.x + this.y

    fn move(dx int):
        this.x += dx
end

class Counter:
    n int

    fn new():
        this.n = 0

    fn inc(step int):
        self.n += step

    fn value() -> int:
        return self.n

fn main():
    p Point(3, 4)
    printf("p=%d\n", p.sum())
    p.move(10)
    printf("p2=%d\n", p.sum())
    c Counter()
    c.inc(5)
    c.inc(7)
    printf("c=%d\n", c.value())
    return 0
