# rusty-ping

A Rust built Ping CLI application that supports Ipv4 and hostname addressing. This application was developed for the Cloudflare systems internship.

## Commandline Usage

```
$ cargo run -- 1.1.1.1 --ttl=128
$ ./target/debug/rusty-ping google.com --ttl=4
```

## Output

This is the output when pinging google.com with a TTL set to 59

```
$ ./target/debug/rusty-ping google.com --ttl=59
Pinging 172.217.12.174 with TTL=59
Reply from 172.217.12.174: bytes=28 time=1ms TTL=56
        Packets Sent = 1, Received = 1, Lost = 0 (0% loss)

Pinging 172.217.12.174 with TTL=59
Reply from 172.217.12.174: bytes=28 time=1ms TTL=56
        Packets Sent = 2, Received = 2, Lost = 0 (0% loss)

Pinging 172.217.12.174 with TTL=59
Reply from 172.217.12.174: bytes=28 time=1ms TTL=56
        Packets Sent = 3, Received = 3, Lost = 0 (0% loss)
```

This is the output when pinging 1.1.1.1 with a TTL set to 2. Notice that because it either timed out or received a time exceeded response, it will increase the TTL by 1 to attempt a connection.

```
$ ./target/debug/rusty-ping 1.1.1.1 --ttl=2
Pinging 1.1.1.1 with TTL=2
Packet Error: Time exceeded.
        Packets Sent = 1, Received = 0, Lost = 1 (100% loss)

Pinging 1.1.1.1 with TTL=3
Packet timed out
        Packets Sent = 2, Received = 0, Lost = 2 (100% loss)

Pinging 1.1.1.1 with TTL=4
Packet timed out
        Packets Sent = 3, Received = 0, Lost = 3 (100% loss)

Pinging 1.1.1.1 with TTL=5
Packet timed out
        Packets Sent = 4, Received = 0, Lost = 4 (100% loss)

Pinging 1.1.1.1 with TTL=6
Packet timed out
        Packets Sent = 5, Received = 0, Lost = 5 (100% loss)

Pinging 1.1.1.1 with TTL=7
Reply from 1.1.1.1: bytes=28 time=1ms TTL=59
        Packets Sent = 6, Received = 1, Lost = 5 (83% loss)

Pinging 1.1.1.1 with TTL=7
Reply from 1.1.1.1: bytes=28 time=1ms TTL=59
        Packets Sent = 7, Received = 2, Lost = 5 (71% loss)
```
