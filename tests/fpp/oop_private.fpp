class Secret:
    n int
    private
    fn hidden() -> int:
        return this.n

fn main():
    s Secret()
    s.hidden()
    return 0
