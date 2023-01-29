# Net Stack

## C socket

```c
int socket(int domain, int type, int protocol);
```

| Arg | Description |
| --- | --- |
| domain | set signal domain(Local(PF_LOCAL), ipv4(AF_INET), ipv6() etc.) |
| type | socket type (TCP(SOCK_STEAM), UDP(SOCK_DGRAM)) |
| protocol | always be 0 |

## From xv6

the application from xv6.

The sequence of setting the ip and mask of the os is as follows.

1.

The sequence of creating a simple tcp echo server is as follows.

1. `register` a socket
2. set socket info, such as family, addr and port.
3. `bind` a socket through syscall
4. `listen` a socket
5. `accept` a client (Waiting for the connection from the client)
6. enter a loop that receives data from the client and sends it to the client.
