use std::collections::{hash_map, HashMap, HashSet};
use std::fmt;
use std::net::SocketAddr;

use rand::{self, Rng};

use {LogIndex, Term};
use messages_capnp::{
    append_entries_request,
    append_entries_response,
    client_request,
    request_vote_request,
    request_vote_response,
};
use state_machine::StateMachine;
use store::Store;

const HEARTBEAT_MIN: u64 = 150;
const HEARTBEAT_MAX: u64 = 300;

/// Replicas can be in one of three state:
///
/// * `Follower` - which replicates AppendEntries requests and votes for it's leader.
/// * `Leader` - which leads the cluster by serving incoming requests, ensuring
///              data is replicated, and issuing heartbeats.
/// * `Candidate` -  which campaigns in an election and may become a `Leader`
///                  (if it gets enough votes) or a `Follower`, if it hears from
///                  a `Leader`.
#[derive(Clone, Debug)]
enum ReplicaState {
    Follower,
    Candidate(CandidateState),
    Leader(LeaderState),
}

#[derive(Clone, Debug)]
struct LeaderState {
    last_index: LogIndex,
    next_index: HashMap<SocketAddr, LogIndex>,
    match_index: HashMap<SocketAddr, LogIndex>,
}

impl LeaderState {

    /// Returns a new `LeaderState` struct.
    ///
    /// # Arguments
    ///
    /// * `last_index` - The index of the leader's most recent log entry at the
    ///                  time of election.
    pub fn new(last_index: LogIndex) -> LeaderState {
        LeaderState {
            last_index: last_index,
            next_index: HashMap::new(),
            match_index: HashMap::new(),
        }
    }

    /// Returns the next log entry index of the follower node.
    pub fn next_index(&mut self, node: SocketAddr) -> LogIndex {
        match self.next_index.entry(node) {
            hash_map::Entry::Occupied(entry) => *entry.get(),
            hash_map::Entry::Vacant(entry) => *entry.insert(self.last_index + 1),
        }
    }

    /// Sets the next log entry index of the follower node.
    pub fn set_next_index(&mut self, node: SocketAddr, index: LogIndex) {
        self.next_index.insert(node, index);
    }

    /// Returns the index of the highest log entry known to be replicated on
    /// the follower node.
    pub fn match_index(&self, node: SocketAddr) -> LogIndex {
        *self.match_index.get(&node).unwrap_or(&LogIndex(0))
    }

    /// Sets the index of the highest log entry known to be replicated on the
    /// follower node.
    pub fn set_match_index(&mut self, node: SocketAddr, index: LogIndex) {
        self.match_index.insert(node, index);
    }

    /// Counts the number of follower nodes containing the given log index.
    pub fn count_match_indexes(&self, index: LogIndex) -> usize {
        self.match_index.values().filter(|&&i| i >= index).count()
    }
}

#[derive(Clone, Debug)]
struct CandidateState {
    granted_votes: usize,
}

/// A replica of a Raft distributed state machine. A Raft replica controls a client state machine,
/// to which it applies commands in a globally consistent order.
pub struct Replica<S, M> {
    addr: SocketAddr,
    peers: HashSet<SocketAddr>,

    store: S,
    state_machine: M,

    state: ReplicaState,

    /// Volatile State
    commit_index: LogIndex,
    last_applied: LogIndex,
    should_campaign: bool,
}

impl <S, M> Replica<S, M> where S: Store, M: StateMachine {

    pub fn new(addr: SocketAddr,
               peers: HashSet<SocketAddr>,
               store: S,
               state_machine: M)
               -> Replica<S, M> {
        Replica {
            addr: addr,
            peers: peers,
            state: ReplicaState::Follower,
            store: store,
            state_machine: state_machine,
            commit_index: LogIndex(0),
            last_applied: LogIndex(0),
            should_campaign: true,
        }
    }

    /// Apply an append entries request to the Raft replica.
    pub fn append_entries_request(&mut self,
                                  from: SocketAddr,
                                  request: append_entries_request::Reader,
                                  mut response: append_entries_response::Builder) {
        assert!(self.peers.contains(&from), "Received append entries request from unknown node {}.", from);
        debug!("{:?}: AppendEntriesRequest from Replica({})", self, from);

        let leader_term = Term(request.get_term());
        let current_term = self.store.current_term().unwrap();

        if leader_term < current_term {
            response.set_term(current_term.into());
            response.set_stale_term(());
            return;
        }

        match self.state {
            ReplicaState::Follower => {
                if current_term < leader_term {
                    self.store.set_current_term(leader_term).unwrap();
                    response.set_term(leader_term.into());
                } else {
                    response.set_term(current_term.into());
                }

                let prev_log_index = LogIndex(request.get_prev_log_index());
                let prev_log_term = Term(request.get_prev_log_term());

                let local_latest_index = self.store.latest_log_index().unwrap();
                if local_latest_index < prev_log_index {
                    response.set_inconsistent_prev_entry(());
                } else {
                    let (existing_term, _) = self.store.entry(prev_log_index).unwrap();
                    if existing_term != prev_log_term {
                        self.store.truncate_entries(prev_log_index).unwrap();
                        response.set_inconsistent_prev_entry(());
                    } else {
                        let entries = request.get_entries().unwrap();
                        let num_entries = entries.len();
                        if num_entries > 0 {
                            let mut entries_vec = Vec::with_capacity(num_entries as usize);
                            for i in 0..num_entries {
                                entries_vec.push((leader_term, entries.get(i).unwrap()));
                            }
                            self.store.append_entries(prev_log_index + 1, &entries_vec).unwrap();
                        }
                        response.set_success(());
                    }
                }
            },
            ReplicaState::Candidate(..) => {
                // recognize the new leader, return to follower state, and apply the entries
                self.transition_to_follower(leader_term);
                return self.append_entries_request(from, request, response)
            },
            ReplicaState::Leader(..) => {
                if leader_term == current_term {
                    // The single leader-per-term invariant is broken; there is a bug in the Raft
                    // implementation.
                    panic!("ID {}: peer leader {} with matching term {:?} detected.",
                           self.addr, from, current_term);
                }

                // recognize the new leader, return to follower state, and apply the entries
                self.transition_to_follower(leader_term);
                return self.append_entries_request(from, request, response)
            },
        }
    }

    /// Apply an append entries response to the Raft replica.
    ///
    /// The provided message may be initialized with a message to send back to the original
    /// responder.
    ///
    /// # Returns
    ///
    /// Returns `true` if the passed in message should be sent to the responder.
    pub fn append_entries_response(&mut self,
                                   from: SocketAddr,
                                   response: append_entries_response::Reader,
                                   message: append_entries_request::Builder) -> bool {
        assert!(self.peers.contains(&from), "Received append entries response from unknown node {}.", from);
        debug!("{:?}: AppendEntriesResponse from Replica({})", self, from);

        // TODO
        unimplemented!();
    }

    /// Apply a request vote request to the Raft replica.
    pub fn request_vote_request(&mut self,
                                candidate: SocketAddr,
                                request: request_vote_request::Reader,
                                mut response: request_vote_response::Builder) {
        assert!(self.peers.contains(&candidate), "Received request vote request from unknown node {}.", candidate);
        debug!("{:?}: RequestVoteRequest from Replica({})", self, candidate);

        let candidate_term = Term(request.get_term());
        let candidate_index = LogIndex(request.get_last_log_index());
        let local_term = self.store.current_term().unwrap();
        let local_index = self.store.latest_log_index().unwrap();

        if candidate_term > local_term {
            self.store.set_current_term(candidate_term).unwrap();
            response.set_term(candidate_term.into());
        } else {
            response.set_term(local_term.into());
        }

        if candidate_term < local_term {
            response.set_stale_term(());
        } else if candidate_index < local_index {
            response.set_inconsistent_log(());
        } else {
            match self.store.voted_for().unwrap() {
                None => {
                    self.store.set_voted_for(candidate).unwrap();
                    response.set_granted(());
                    self.should_campaign = false;
                },
                Some(voted_for) if voted_for == candidate => {
                    response.set_granted(());
                    self.should_campaign = false;
                },
                _ => {
                    response.set_already_voted(());
                },
            }
        }
    }

    /// Apply a request vote response to the Raft replica.
    ///
    /// # Return
    ///
    /// Returns `true` if the provided AppendEntriesRequest should be sent to every peer cluster
    /// member.
    pub fn request_vote_response(&mut self,
                                 from: SocketAddr,
                                 response: request_vote_response::Reader,
                                 message: append_entries_request::Builder)
                                 -> bool {
        assert!(self.peers.contains(&from), "Received request vote response from unknown node {}.", from);
        debug!("{:?}: RequestVoteResponse from Replica({})", self, from);

        let local_term = self.store.current_term().unwrap();
        let voter_term = Term::from(response.get_term());

        let majority = self.majority();
        let mut transition_to_leader = false;

        if local_term < voter_term {
            // Respondent has a higher term number. The election is compromised; abandon it and
            // update the local term number. We will stay in candidate state, since we don't
            // know of a valid leader to follow, but any further responses we receive from this
            // election cycle will be ignored because the term will be outdated.
            self.store.set_current_term(voter_term).unwrap();
            if let ReplicaState::Candidate(ref mut candidate_state) = self.state {
                candidate_state.granted_votes = 0;
            }
        } else if local_term > voter_term {
            // Ignore this message; it came from a previous election cycle.
        } else if let ReplicaState::Candidate(ref mut state) = self.state {
            if let Ok(request_vote_response::Granted(_)) = response.which() {
                state.granted_votes += 1;
                if state.granted_votes >= majority {
                    transition_to_leader = true;
                }
            }
        }

        if transition_to_leader {
            self.transition_to_leader(message);
        }
        transition_to_leader
    }

    /// Apply a client request to the Raft replica.
    pub fn client_request(&mut self, from: SocketAddr, _request: client_request::Reader) {
        debug!("{:?}: ClientRequest from Client({})", self, from);
        unimplemented!();
    }

    /// Trigger a timeout on the Raft replica.
    ///
    /// The provided RequestVoteRequest builder may be initialized with a message to send to each
    /// cluster peer.
    ///
    /// # Return
    ///
    /// A new timeout period, and whether the RequestVote message should be sent to each cluster
    /// peer.
    pub fn timeout(&mut self, message: request_vote_request::Builder) -> (u64, bool) {
        debug!("{:?}: Timeout", self);
        let send_message = if self.should_campaign && !self.is_leader() {
            if self.peers.is_empty() {
                // Solitary replica special case; jump straight to leader status
                assert!(self.is_follower());
                assert!(self.store.voted_for().unwrap().is_none());
                self.store.inc_current_term().unwrap();
                self.store.set_voted_for(self.addr).unwrap();
                let latest_log_index = self.store.latest_log_index().unwrap();
                self.state = ReplicaState::Leader(LeaderState::new(latest_log_index));
                false
            } else {
                self.transition_to_candidate(message);
                true
            }
        } else { false };

        self.should_campaign = true;
        let timeout = rand::thread_rng().gen_range::<u64>(HEARTBEAT_MIN, HEARTBEAT_MAX);

        (timeout, send_message)
    }

    /// Transition this Replica to Leader state.
    ///
    /// The provided AppendEntriesRequest builder will be initialized with a message to send to each
    /// cluster peer.
    fn transition_to_leader(&mut self, mut message: append_entries_request::Builder) {
        info!("{:?}: Transition to Leader", self);
        let current_term = self.store.current_term().unwrap();
        let latest_log_index = self.store.latest_log_index().unwrap();
        let latest_log_term = self.store.latest_log_term().unwrap();
        self.state = ReplicaState::Leader(LeaderState::new(latest_log_index));

        message.set_term(current_term.into());
        message.set_prev_log_index(latest_log_index.into());
        message.set_prev_log_term(latest_log_term.into());
        message.set_leader_commit(self.commit_index.into());
    }

    /// Transition this Replica to Candidate state.
    ///
    /// The provided RequestVoteRequest message will be initialized with a message to send to each
    /// cluster peer.
    fn transition_to_candidate(&mut self, mut message: request_vote_request::Builder) {
        info!("{:?}: Transition to Candidate", self);
        self.store.inc_current_term().unwrap();
        self.store.set_voted_for(self.addr).unwrap();
        self.state = ReplicaState::Candidate(CandidateState { granted_votes: 1 });

        let current_term = self.store.current_term().unwrap();
        let latest_index = self.store.latest_log_index().unwrap();
        let latest_term = self.store.latest_log_term().unwrap();

        message.set_term(current_term.into());
        message.set_last_log_index(latest_index.into());
        message.set_last_log_term(latest_term.into());
    }

    /// Transition to follower state with the provided term. The `voted_for` field will be reset.
    fn transition_to_follower(&mut self, term: Term) {
        info!("{:?}: Transition to Follower", self);
        self.store.set_current_term(term).unwrap();
        self.state = ReplicaState::Follower;
    }

    /// Returns `true` if the replica is in the Leader state.
    ///
    /// public for testing.
    fn is_leader(&self) -> bool {
        if let ReplicaState::Leader(..) = self.state { true } else { false }
    }

    /// Returns `true` if the replica is in the Follower state.
    ///
    /// public for testing.
    pub fn is_follower(&self) -> bool {
        if let ReplicaState::Follower = self.state { true } else { false }
    }

    /// Returns `true` if the replica is in the Candidate state.
    ///
    /// public for testing.
    pub fn is_candidate(&self) -> bool {
        if let ReplicaState::Candidate(..) = self.state { true } else { false }
    }

    /// Returns the address of the replica.
    ///
    /// public for testing.
    pub fn addr(&self) -> &SocketAddr {
        &self.addr
    }

    /// Returns the current term of the replica.
    ///
    /// public for testing.
    pub fn current_term(&self) -> Term {
        self.store.current_term().unwrap()
    }


    /// Get the cluster quorum majority size.
    fn majority(&self) -> usize {
        let peers = self.peers.len();
        let cluster_members = peers.checked_add(1).expect(&format!("unable to support {} cluster members", peers));
        (cluster_members >> 1) + 1
    }
}

impl <S, M> fmt::Debug for Replica<S, M> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "Replica({})", self.addr)
    }
}

#[cfg(test)]
mod test {

    use std::collections::HashSet;
    use std::net::{IpAddr, SocketAddr};
    use std::sync::mpsc;
    use std::str::FromStr;

    use capnp::{MallocMessageBuilder, MessageBuilder};

    use messages_capnp::{
        append_entries_request,
        request_vote_request,
        request_vote_response,
    };
    use replica::Replica;
    use state_machine::ChannelStateMachine;
    use store::MemStore;
    use Term;

    type TestReplica = Replica<MemStore, ChannelStateMachine>;

    fn new_cluster(size: u16) -> Vec<(TestReplica, mpsc::Receiver<Vec<u8>>)> {
        // the actual port does not matter here since they won't be bound
        let addrs: HashSet<SocketAddr> =
            (0..size).map(|port| FromStr::from_str(&format!("127.0.0.1:{}", port)).unwrap()).collect();

        addrs.iter().map(|addr| {
            let mut peers = addrs.clone();
            peers.remove(addr);
            let store = MemStore::new();
            let (state_machine, recv) = ChannelStateMachine::new();
            (Replica::new(addr.clone(), peers, store, state_machine), recv)
        }).collect()
    }

    /// Tests that a single-replica cluster will behave appropriately.
    ///
    /// The single replica should transition straight to the Leader state upon the first timeout.
    #[test]
    fn test_solitary_replica_transition_to_leader() {
        let (mut replica, _) = new_cluster(1).pop().unwrap();
        assert!(replica.is_follower());

        let mut message = MallocMessageBuilder::new_default();
        let request = message.init_root::<request_vote_request::Builder>();

        let (_, send_message) = replica.timeout(request);
        assert!(!send_message);
        assert!(replica.is_leader());
    }

    /// A simple election test of a two-replica cluster.
    #[test]
    fn test_election() {
        let mut replicas = new_cluster(2);
        let (mut replica1, _) = replicas.pop().unwrap();
        let (mut replica2, _) = replicas.pop().unwrap();

        let mut request = MallocMessageBuilder::new_default();
        let mut response = MallocMessageBuilder::new_default();

        // Trigger replica1's timeout, and make sure it transitions to candidate

        let (_, send_message) =
            replica1.timeout(request.init_root::<request_vote_request::Builder>());
        assert!(send_message);
        assert!(replica1.is_candidate());

        // Send replica1's RequestVoteRequest to replica2

        replica2.request_vote_request(replica1.addr().clone(),
                                      request.get_root::<request_vote_request::Builder>().unwrap().as_reader(),
                                      response.init_root::<request_vote_response::Builder>());

        let resp = response.get_root::<request_vote_response::Builder>().unwrap().as_reader();
        assert!(if let request_vote_response::Which::Granted(_) = resp.which().unwrap() { true } else { false });

        // Trigger replica2's timeout, and make sure it does *not* transitition to candidate, since
        // it has already voted in an election during this timeout period.
        let (_, send_message) =
            replica2.timeout(request.init_root::<request_vote_request::Builder>());
        assert!(!send_message);

        // Return success vote to candidate, and make sure it transitions to leader
        let send_message = replica1.request_vote_response(replica2.addr().clone(),
                                                          resp,
                                                          request.init_root::<append_entries_request::Builder>());
        assert!(send_message);
        assert!(replica1.is_leader());
        assert!(replica1.current_term() == Term::from(1));
    }
}