#!/usr/bin/env python3
"""Drive actual less in a fresh controlling PTY and validate search/quit."""
import fcntl,os,pty,select,struct,termios,time
from pathlib import Path
root=Path(__file__).resolve().parent.parent
pid,fd=pty.fork()
if pid==0:
    os.environ.update(TERM='xterm-256color',PAGER='less -R')
    os.chdir(root)
    os.execv(str(root/'target/release/marklight'),['marklight','fixtures/markdown/huge.md','--width','60'])
fcntl.ioctl(fd,termios.TIOCSWINSZ,struct.pack('HHHH',24,80,0,0))
def read(seconds):
    result=b'';deadline=time.monotonic()+seconds
    while time.monotonic()<deadline:
        if select.select([fd],[],[],.05)[0]:
            try:result+=os.read(fd,65536)
            except OSError:break
    return result
try:
    initial=read(.7)
    assert b'Section 0' in initial and b'Section 799' not in initial,'pager did not limit initial view'
    os.write(fd,b'/Section 400\n')
    searched=read(.4)
    assert b'400' in searched,'search did not reach requested heading'
    os.write(fd,b'q')
    read(.2)
    deadline=time.monotonic()+3
    while time.monotonic()<deadline:
        result,status=os.waitpid(pid,os.WNOHANG)
        if result:
            assert os.waitstatus_to_exitcode(status)==0
            print('PASS: real less paging, / search and q exit in PTY')
            break
        time.sleep(.05)
    else:raise AssertionError('q did not quit the pager')
finally:
    os.close(fd)
