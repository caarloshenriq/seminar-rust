# Task 1: Warm up 

> Bitcoin is structured as a peer-to-peer network architecture on top of the internet.
> The term peer-to-peer, or P2P, means that the full nodes that participate in the network are peers to each other, that they can all perform the same functions, and that there are no “special” nodes.
> The network nodes interconnect in a mesh network with a “flat” topology.
> There is no server, no centralized service, and no hierarchy within the network.
> Nodes in a P2P network both provide and consume services at the same time.[^1]

[^1]: A. Antonopoulos and D. Harding; Mastering Bitcoin.

The predominant design of applications on the internet is based on the client-server architecture we all learned to build and love.
In this kind of design, servers and clients have stark different functions and capabilities.
Usually, the server is way more capable then the clients, having access to privileged data and controling authentication and access control.
Even in applications that involve connecting different clients (e.g. messaging systems), these connections are mediated by the server; i.e., both clients establish connections with the server and not among them.

In a P2P architecture, on the other hand, the participants are called *nodes* and they connect and talk directly to each other.
Because there's no privileged entity on the system, nodes are usually way more sophisticated than clients.
One of the tasks they have to perform is to manage the connections with other nodes (its peers).
It is common to include discovery mechanisms so that one node can get acquainted to new nodes.

As you will see in Task 2, the Bitcoin protocol provides such a mechanism by specifying special messages that are used to ask for and announce known peers.
In this way, even if my node is connected to a single peer, it will eventually gather the necessary information[^2] to connect to other peers and improve its reachability in the network.
But it rests the question: if I'm not already connected to a peer, how can I find addresses to connect to the network?
We call this process *bootstraping*[^3] into the network.

[^2]: mainly IP addresses for TCP connections, but other transport mechanisms are also used.

[^3]: https://en.wikipedia.org/wiki/Bootstrapping#Etymology

## Bitcoin DNS Seeders

In regard to what the Bitcoin protocol requires, it is true that there are no special nodes in the Bitcoin network.
But the practical implementation of the network does rely on specialized nodes that provide services to application clients and to the full nodes themselves.
Advertising Bitcoin peers addresses is such a service that helps in the bootstraping process of new nodes.
In the onset of Bitcoin, getting an initial set of node addresses to connect was done in user foruns and on IRC.
Given the dynamic nature of a P2P network in which nodes can come and go at any moment, as the network grew this process was automatized by DNS Seeders.

A Bitcoin DNS Seeder is a Bitcoin client that actively connects to nodes on the P2P network and retrieve peers addresses, trying to connect to them as well[^4].
The seeder maintains a database of known addresses that can be filtered by known (I have heard about them), active (I have succesfully connected to them), and inactive (I couldn't connect to them).
You can figure out other statuses, e.g., ban an address know to misbehave [^5].
The seeder also acts as a DNS server that provides `A` records in response to a DNS query.
See the example below (your results will probably differ as the reported set of known addresses is randomized).

[^4]: The seeder also suffers from the bootstraping problem as you will see. The first seeders were fed addresses manually and are running since the early ages of the Bitcoin network, keeping their databases up to date. We are going to use an existing seeder to bootstrap ours.

[^5]: See Provoost, S. **Bitcoin: A Work in Progress** chapters 7 (Eclispe attacks) and 8 (Fake nodes).

```bash
❯ dig seed.bitcoin.sipa.be

; <<>> DiG 9.10.6 <<>> seed.bitcoin.sipa.be
;; global options: +cmd
;; Got answer:
;; ->>HEADER<<- opcode: QUERY, status: NOERROR, id: 49894
;; flags: qr rd ra; QUERY: 1, ANSWER: 25, AUTHORITY: 0, ADDITIONAL: 1

;; OPT PSEUDOSECTION:
; EDNS: version: 0, flags:; udp: 512
;; QUESTION SECTION:
;seed.bitcoin.sipa.be.          IN      A

;; ANSWER SECTION:
seed.bitcoin.sipa.be.   2455    IN      A       57.129.38.163
seed.bitcoin.sipa.be.   2455    IN      A       139.177.179.5
seed.bitcoin.sipa.be.   2455    IN      A       80.228.205.180
seed.bitcoin.sipa.be.   2455    IN      A       170.187.185.151
seed.bitcoin.sipa.be.   2455    IN      A       172.236.224.103
seed.bitcoin.sipa.be.   2455    IN      A       172.104.174.254
seed.bitcoin.sipa.be.   2455    IN      A       107.139.249.27
seed.bitcoin.sipa.be.   2455    IN      A       188.192.229.52
seed.bitcoin.sipa.be.   2455    IN      A       172.104.130.244
seed.bitcoin.sipa.be.   2455    IN      A       218.154.213.62
seed.bitcoin.sipa.be.   2455    IN      A       80.181.229.24
seed.bitcoin.sipa.be.   2455    IN      A       123.100.246.115
seed.bitcoin.sipa.be.   2455    IN      A       157.173.107.70
seed.bitcoin.sipa.be.   2455    IN      A       88.10.254.88
seed.bitcoin.sipa.be.   2455    IN      A       98.251.47.34
seed.bitcoin.sipa.be.   2455    IN      A       90.240.33.39
seed.bitcoin.sipa.be.   2455    IN      A       13.126.144.12
seed.bitcoin.sipa.be.   2455    IN      A       34.102.53.81
seed.bitcoin.sipa.be.   2455    IN      A       184.162.157.34
seed.bitcoin.sipa.be.   2455    IN      A       66.163.223.241
seed.bitcoin.sipa.be.   2455    IN      A       123.100.246.236
seed.bitcoin.sipa.be.   2455    IN      A       172.105.193.102
seed.bitcoin.sipa.be.   2455    IN      A       45.33.125.229
seed.bitcoin.sipa.be.   2455    IN      A       203.11.72.160
seed.bitcoin.sipa.be.   2455    IN      A       139.162.150.59

;; Query time: 47 msec
;; SERVER: 8.8.8.8#53(8.8.8.8)
;; WHEN: Mon Mar 24 16:34:02 -03 2025
;; MSG SIZE  rcvd: 449
```

The oldest known implementation, called the [Bitcoin Seeder](https://github.com/sipa/bitcoin-seeder), was developed by Pieter Wuille and [hardcoded into Bitcoin Core](https://github.com/bitcoin/bitcoin/blob/dbc450c1b59b24421ba93f3e21faa8c673c0df4c/src/kernel/chainparams.cpp#L145) in 2011.
There are other implementations around, including a Rust one, but I recommend you don't look for them.
The idea is that you don't get biased when writing your own port in Rust.

## Tasks

### Install and get acquainted with the Rust toolchain

- Study chapters 1, 2, 3, and 7 of the [Rust Book](https://doc.rust-lang.org/stable/book/title-page.html).

### Run the bitcoin seeder

- Compile and run the [Bitcoin Seeder](https://github.com/sipa/bitcoin-seeder).
Your goal is to reimplement this thing, thus get familiar to what it's doing.
- Study it's source code and get a feel for how it is structured.
Try to identify weak points that can be improved in your implementation (e.g. how does the database component is implemented?).

### Get acquainted to DNS

You are going to interact and implement a DNS server in future tasks.
So, it's a good idea to get aquaincted to what's a DNS server, DNS queries and DNS records.
You don't need to understand the whole complexity of modern DNS systems, just the basics of how the protocol works.

- The [Domain Name System Wikipedia entry](https://en.wikipedia.org/wiki/Domain_Name_System) is a good starting point.
- See also the original specifications in [RFC 1034](https://datatracker.ietf.org/doc/html/rfc1034) and [RFC 1035](https://datatracker.ietf.org/doc/html/rfc1035).

### Get acquainted to the Bitcoin P2P protocol

Get best reference is probably the [Bitcoin Wiki](https://en.bitcoin.it/wiki/Protocol_documentation#getaddr).

### A word of warning

These are easy tasks to get you up and running, but don't get fooled: this seminar is designed to be challenging and the next tasks are going to be way more challenging.
