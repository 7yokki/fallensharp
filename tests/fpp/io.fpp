import system.exec

fn main():
    name string => stdin("Ad: ")
    stdout("Selam ")
    stdout(name)
    stdout("\n")
    out string => system.exec("echo merhaba", output=True)
    stdout(out)
    return 0
