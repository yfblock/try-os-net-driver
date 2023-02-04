import socket
import sys
import time

sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
addr = ('localhost', int(sys.argv[1]))
buf = "this is a ping!".encode('utf-8')
sock.bind(addr)


print("pinging...", file=sys.stderr)
while True:
        buf, raddr = sock.recvfrom(4096)
        print("receive: " + buf.decode("utf-8"))
        sock.sendto(buf, raddr)
        time.sleep(1)
