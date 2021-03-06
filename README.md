# Raft #

> Note: This project is **incomplete** and the code is of **pre-alpha** quality. **Only some specific, basic functionality is available, and I need your help defining an API!**

[![Build Status](https://travis-ci.org/Hoverbear/raft.svg)](https://travis-ci.org/Hoverbear/raft)

**[Blog Updates](http://www.hoverbear.org/tag/raft/)**

## Problem and Importance ##

When building a distributed system one principal goal is often to build in *fault-tolerance*. That is, if one particular node in a network goes down, or if there is a network partition, the entire cluster does not fall over. The cluster of nodes taking part in a distributed consensus protocol must come to agreement regarding values, and once that decision is reached, that choice is final.

Distributed Consensus Algorithms often take the form of a replicated state machine and log. Each state machine accepts inputs from its log, and represents the value(s) to be replicated, for example, a hash table. They allow a collection of machines to work as a coherent group that can survive the failures of some of its members.

Two well known Distributed Consensus Algorithms are Paxos and Raft. Paxos is used in systems like [Chubby](http://research.google.com/archive/chubby.html) by Google, and Raft is used in things like [`etcd`](https://github.com/coreos/etcd/tree/master/raft). Raft is generally seen as a more understandable and simpler to implement than Paxos, and was chosen for this project for this reason.

This project is appropriate for this class as it involves a number of peers distributed over a network with no consistent leader attempting to reliably replicate a log (and accompanying state machine). The vast majority of the code will be related to Remote Procedure Calls and the handling of network connections, particularly UDP based connections. There are a number of interesting failure modes to explore, including network partitions, failed hosts, and a variety of I/O errors.

## What's Been Done ##

There are numerous Raft implementations in a variety of languages, the most popular are in Go. Given this algorithm's foundational role in building distributed systems and  networks, it would be useful to have this algorithm easily available to users of the Rust programming language as well.

## Approach ##

I propose a simple implementation of Raft with Leader Election and Log Replication. If time permits, Membership Changes and Log Compaction will be included, however they are not required. The implementation language will be [Rust](http://rust-lang.org/) for a number of reasons: personal interest, strong typing, data safety, lack of garbage collector, and acceptable FFI interfaces.

My deliverables will be a functioning MIT licensed library with simple bindings, adequate test coverage, and example code.

## Schedule ##

* Jan 21-Feb 4: Basic structure and scaffold implementation.
* Feb 4-18: Refinement and materialization of protocol.
* Feb 18-Mar 4: Testing and example building.
* Mar 4-Onward: Release to community and improve based on feedback.

## Compiling ##

> For Linux or Mac, with root! Windows users are, unfortunately, on their own.

You will need the [Rust](http://rust-lang.org/) compiler:

```bash
curl -L https://static.rust-lang.org/rustup.sh | sudo sh
```

This should install `cargo` and `rustc`. Next, you'll need `capnp` to build the
`messages.canpnp` file . It is suggested to use the [git method](https://capnproto.org/install.html#installation-unix)

```bash
git clone https://github.com/sandstorm-io/capnproto.git
cd capnproto/c++
./setup-autotools.sh
autoreconf -i
./configure
make -j6 check
sudo make install
```

Finally, clone the repository and build it:

```bash
git clone git@github.com:Hoverbear/raft.git && \
cd raft && \
cargo build
```

> Note this is a library, so building won't necessarily produce anything useful for you.

## Documentation ##

* [Raft Crate Documentation](https://hoverbear.github.io/raft/raft/)
* [The Raft site](https://raftconsensus.github.io/)
* [The Secret Lives of Data - Raft](http://thesecretlivesofdata.com/raft/)
* [Raft Paper](http://ramcloud.stanford.edu/raft.pdf)
* [Raft Dissertation](https://github.com/ongardie/dissertation#readme)

## Testing ##

You can run `raft`'s full bank of tests with all debug output like so:

```bash
RUST_LOG=raft=debug cargo test -- --nocapture
```

> Due to the nature of this library's pre-alpha state, some tests may occasionally fail. I'm working on this.

For something more terse use `cargo test`.

The `tests/lib.rs` file currently hosts a working example of using the library. *Before you say anything, yes, I know it's not nice. Let's talk about how to make that much nicer!*

## Project Website ##

I'll keep track of my progress both on my blog through the [Raft](http://www.hoverbear.org/tag/raft/) tag, and via Github.
