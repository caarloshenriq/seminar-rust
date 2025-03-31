# Task 2: Bitcoin client

The Bitcoin P2P protocol defines a set of messages that can be exchanged between peers.
A good reference is the [Bitcoin Wiki](https://en.bitcoin.it/wiki/Protocol_documentation).
However, contrary to other systems in which the implementations should comply to a certain specification (i.e., a mismatching behavior between the specification document and a piece of software that implements it is considered a bug in the software artifact), in the Bitcoin network the [Bitcoin Core](https://github.com/bitcoin/bitcoin) client serves as a reference implementation.
Off course there is some effort in formally specifying certain behaviors by means of [Bitcoin Improvement Proposals](https://github.com/bitcoin/bips) (BIPS), but it is correct to say that, for the most part, the correct behavior in the network is whatever Bitcoin Core does in a certain situation, i.e., the software implementation is the specification.
This is quite unfortunate because it forces us to study the Bitcoin Core source code in case things don't work as expected.
To encourage you to do so, we included references to relevant parts of the Bitcoin Core source code when appropriate[^1].
Also, that's why developing well designed and tested libraries is so important for the Bitcoin ecosystem.

[^1]: You came to learn Rust in a Bitcoin context, and since Rust tries to fit in the same niche as C++, studying a complex C++ code base will be beneficial.

## P2P protocol v1

Bitcoin is a permissionless network whose purpose is to reach consensus over public data.
Since all data relayed in the Bitcoin P2P network is inherently public, and the protocol lacks a notion of cryptographic identities, peers talk to each other over unencrypted and unauthenticated connections.
Furthermore, the protocol is agnostic to what transport mechanism is used to send and receive bytes.
The original Satoshi client used IPv4 TCP streams.
Current versions of Bitcoin Core[^2] support IPv6 TCP, Tor and I2P streams.

[^2]: v28 by the time of the writing.

The network messages follow a common structure[^3]:

[^3]: See the [CMessageHeader class](https://github.com/bitcoin/bitcoin/blob/dfb7d58108daf3728f69292b9e6dba437bb79cc7/src/protocol.h#L28).

- **magic**: 4 bytes indicating message origin network, and used to seek to next message when stream state is unknown;
- **command**: 12 bytes with a NULL padded ASCII string identifying the packet content;
- **payload_size**: 4 bytes indicating the length of payload in number of bytes (can be zero for messages with no payload);
- **checksum**: first 4 bytes of sha256(sha256(payload));
- **payload**: actual data, if any.

<!-- 
TODO: insert reference for the Bitcoin Core code where this is implemented.

CMessageHeader: https://github.com/bitcoin/bitcoin/blob/dfb7d58108daf3728f69292b9e6dba437bb79cc7/src/protocol.h#L28

ProcessMessage: https://github.com/bitcoin/bitcoin/blob/dbc450c1b59b24421ba93f3e21faa8c673c0df4c/src/net_processing.cpp#L3715

Handshake: https://github.com/bitcoin/bitcoin/blob/dbc450c1b59b24421ba93f3e21faa8c673c0df4c/src/net_processing.cpp#L3726


-->

Almost all integers are encoded in little endian, i.e., least significant bytes are sent first.
Only IP or port numbers are encoded big endian.
For this task, you'll need to deal with `version`, `verack`, `ping`, `pong`, `getaddr`, and `addr` messages.
All others[^4] can be safely ignored.

[^4]: All known messages are in the [`NetMsgType` namespace](https://github.com/bitcoin/bitcoin/blob/dfb7d58108daf3728f69292b9e6dba437bb79cc7/src/protocol.h#L60).

To initiate a connection, a Bitcoin client opens a TCP socket to the remote host and sends a `version`[^5] message.
In response, the remote node will send us back their `version` and a `verack` message in case they accept our connection.
The handshake finalizes by sending a `verack` signaling to the remote node we are accepting messages from it[^6].

[^5]: See [service flags](https://github.com/bitcoin/bitcoin/blob/dfb7d58108daf3728f69292b9e6dba437bb79cc7/src/protocol.h#L309)

[^6]: The handshake process can involve other messages depending on what our client signals in its initial `version` message. 
See the [ProcessMessage](https://github.com/bitcoin/bitcoin/blob/dbc450c1b59b24421ba93f3e21faa8c673c0df4c/src/net_processing.cpp#L3715) function.


Now it's time to implement something.

## Task

Write a Bitcoin client that connects to a public node using TCP.

1. The IP address of the public node can be hardcoded for now (we are changing that later).
You can gather one using `dig seed.bitcoin.sipa.be`.

2. Your client should perform the correct Bitcoin P2P protocol handshake.

3.  It is common for highly connected nodes to disconnect right after the handshake.
You can check it by [`peek`](https://doc.rust-lang.org/std/net/struct.TcpStream.html#method.peek)ing the `TcpStream`.

- Your client should respond to `ping` messages otherwise the remote node will disconnect.
- To start receiving addresses of other nodes, send a `getaddr` message.
You should receive `addr` messages from time to time.
- Print sent and received messages to the terminal using `println!()` (we are upgrading this later).
Ignore received `inv` messages.`

The [bitcoin crate](https://github.com/rust-bitcoin/rust-bitcoin) provides a `p2p` module which can (de)serialize network messages.
Use it in your favor.

## Useful tools

1. [`dig`](https://linux.die.net/man/1/dig) – DNS lookup utility
2. [`nc`](https://linux.die.net/man/1/nc) – arbitrary TCP and UDP connections and listens.
## References

1. https://en.bitcoin.it/wiki/Protocol_documentation
2. https://developer.bitcoin.org/devguide/index.html

