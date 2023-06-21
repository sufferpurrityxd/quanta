use {
  async_std::io,
  futures::prelude::*,
  libp2p::{
    core::upgrade,
    identity,
    noise,
    swarm::{NetworkBehaviour, SwarmBuilder, SwarmEvent},
    tcp,
    yamux,
    Multiaddr,
    PeerId,
    Swarm,
    Transport,
  },
  quanta_swap::Storage,
  std::{
    collections::HashMap,
    error::Error,
    str::{from_utf8, FromStr},
    sync::Arc,
  },
  tokio::sync::Mutex,
};

/// MemoryStorage
struct MemoryStorage(Arc<Mutex<HashMap<Vec<u8>, Vec<u8>>>>);

impl Storage for MemoryStorage {
  fn exists(&mut self, key: Vec<u8>) -> bool {
    let guard = futures::executor::block_on(self.0.lock());
    guard.contains_key(&key)
  }

  fn get(&mut self, key: Vec<u8>) -> Option<Vec<u8>> {
    let guard = futures::executor::block_on(self.0.lock());
    guard.get(&key).cloned()
  }
}

impl MemoryStorage {
  /// Create new storage
  pub fn new(hm: Arc<Mutex<HashMap<Vec<u8>, Vec<u8>>>>) -> Self { Self(hm) }
}

#[derive(NetworkBehaviour)]
struct Behaviour {
  quanta_swap: quanta_swap::Behaviour<MemoryStorage>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
  pretty_env_logger::init_timed();

  // Create a random PeerId
  let id_keys = identity::Keypair::generate_ed25519();
  let local_peer_id = PeerId::from(id_keys.public());
  println!("Local peer id: {local_peer_id}");
  let mut hm = Arc::new(Mutex::new(HashMap::new()));
  let storage = MemoryStorage::new(Arc::clone(&hm));
  // Create behaviour
  let behaviour = Behaviour {
    quanta_swap: quanta_swap::Behaviour::new(storage),
  };
  // transport
  let tcp_transport = tcp::async_io::Transport::new(tcp::Config::default().nodelay(true))
    .upgrade(upgrade::Version::V1Lazy)
    .authenticate(noise::Config::new(&id_keys).expect("signing libp2p-noise static keypair"))
    .multiplex(yamux::Config::default())
    .boxed();
  // create swarm
  let mut swarm =
    SwarmBuilder::with_async_std_executor(tcp_transport, behaviour, local_peer_id).build();
  // cargo run /ipv4/0.0.0.0/tcp/{port}
  if let Some(addr) = std::env::args().nth(1) {
    swarm.dial(Multiaddr::from_str(addr.as_str())?)?;
  };
  let mut stdin = io::BufReader::new(io::stdin())
    .lines()
    .fuse();
  swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;
  loop {
    tokio::select! {
        line = stdin.select_next_some() => handle_input_line(
            &mut swarm,
            &mut hm,
            line.expect("Stdin not to close")
        ),
        event = swarm.select_next_some() => {
            match event {
                SwarmEvent::Behaviour(BehaviourEvent::QuantaSwap(quanta_swap::Event::QueryCompleted {
                    search_id,
                    searching,
                    item,
                })) => {
                    let result_str = from_utf8(item.as_slice()).unwrap();
                    let searching_str = from_utf8(searching.as_slice()).unwrap();
                    println!(
                       "Search completed. ID={}, SEARCHING={}, RESULT={}",
                        search_id, searching_str, result_str
                    );
                },
                SwarmEvent::NewListenAddr {address, ..} => {
                    println!("Swarm listen on: {:?}", address);
                },
                _ => {},
            }
        }
    }
  }
}

fn handle_input_line(
  swarm: &mut Swarm<Behaviour>,
  hm: &mut Arc<Mutex<HashMap<Vec<u8>, Vec<u8>>>>,
  line: String,
) {
  let mut args = line.split(' ');

  match args.next() {
    Some("INSERT") => {
      let key = {
        match args.next() {
          Some(key) => key.as_bytes().to_vec(),
          None => {
            eprintln!("Expected key");
            return;
          },
        }
      };
      let value = {
        match args.next() {
          Some(value) => value.as_bytes().to_vec(),
          None => {
            eprintln!("Expected value");
            return;
          },
        }
      };
      let mut guard = futures::executor::block_on(hm.lock());
      guard.insert(key, value);
    },
    Some("SEARCH") => {
      let key = {
        match args.next() {
          Some(key) => key.as_bytes().to_vec(),
          None => {
            eprintln!("Expected key");
            return;
          },
        }
      };
      let search_id = swarm
        .behaviour_mut()
        .quanta_swap
        .search_item_with(key);
      println!("SEARCH_ID={}", search_id);
    },
    Some(_) => {},
    None => {},
  }
}
