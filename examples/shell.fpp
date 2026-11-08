import system.exec

fn main():
    stdout("Fallen++ shell\n")
    loop:
        cmd string => stdin(">> ")
        if cmd equal 0:
            break
        end
        out string => system.exec(cmd, output=True)
        stdout(out)
    do
    return 0
