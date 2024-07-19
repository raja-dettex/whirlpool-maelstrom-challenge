

use core::time;
use std::{collections::{HashMap, HashSet}, fmt::format, io::{BufRead, StdoutLock, Write}, thread};

use anyhow::{bail, Context, Ok};
use rand::Rng;
use serde::{de::DeserializeOwned, Serialize, Deserialize};
use whirlpool_malestorm_challenge::{main_loop, Body, Event, Message, Node};


#[derive(Serialize , Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
pub enum Payload { 
    Broadcast { message : usize},
    BroadcastOk,
    Read,
    ReadOk { messages : HashSet<usize>},
    Topology { topology : HashMap<String, Vec<String>>},
    TopologyOk, 
    Gossip { seen : HashSet<usize>}
}

#[derive(Serialize , Deserialize, Debug, Clone)]
pub enum InjectedPayload { 
    Gossip
}




#[derive(Debug)]
pub struct BroadcastNode { 
    node : String,
    id : usize,
    messages : HashSet<usize>,
    known : HashMap<String, HashSet<usize>>,
    neighbours : Vec<String>,
    msg_communicated : HashMap<usize, HashSet<usize>>
}

impl Node<(), Payload, InjectedPayload> for BroadcastNode 
{ 
    fn from_init(_state: (), init: whirlpool_malestorm_challenge::Init,  tx_sender : std::sync::mpsc::Sender<Event<Payload, InjectedPayload>>) -> anyhow::Result<Self> where Self:Sized {
      thread::spawn(move || { 
        loop { 
            std::thread::sleep(time::Duration::from_millis(300));
            if let Err(_) = tx_sender.send(Event::Injected(InjectedPayload::Gossip)) { 
                break;
            }
        }
      }); 
      Ok(BroadcastNode{
        node : init.node_id, 
        id: 1, 
        messages : HashSet::new(),
        known : init.node_ids.into_iter().map(|nid| (nid, HashSet::new())).collect(),
        neighbours : Vec::new(),
        msg_communicated : HashMap::new()
        
    })  
    } 
    fn step(&mut self, input : Event<Payload, InjectedPayload>, output: &mut StdoutLock) -> anyhow::Result<()>{
        match input {
            Event::Message(input) => { 
                let mut reply = input.in_reply(Some(&mut self.id));
                match reply.body.payload { 
                    Payload::Gossip { seen  } => { 
                        self.known.get_mut(&reply.dst).expect("got gossip from unknown source").extend(seen.iter().copied());
                        self.messages.extend(seen);
                    }
                    Payload::Broadcast { message } => { 
                        self.messages.insert(message);
                        reply.body.payload = Payload::BroadcastOk;
                        reply.send(&mut *output);
                    }, 
                    Payload::Read => { 
                        reply.body.payload = Payload::ReadOk { messages: self.messages.clone() };
                        reply.send(&mut *output);
                    },
                    Payload::Topology { mut topology } => {
                        self.neighbours = topology.remove(&self.node).unwrap_or_else(|| { 
                            panic!("shoud contain topology for node : {}", self.node);
                        });
                        reply.body.payload = Payload::TopologyOk;
                        reply.send(&mut *output);
                    }, 
                    Payload::BroadcastOk | Payload::ReadOk { .. } | Payload::TopologyOk=>{} 
                }
            },
            Event::Injected(payload) => { 
                match payload {
                    InjectedPayload::Gossip => { 
                        for n in &self.neighbours { 
                            let known_to_n = self.known.get(n).expect("msg");
                            let (mut already_known , mut notify_of)  : (HashSet<_>, HashSet<_>) = self.messages.iter().copied()
                            .partition(|m| !known_to_n.contains(m));
                            eprintln!("notified {}/{}", notify_of.len() , self.messages.len());
                            let mut rng = rand::thread_rng();
                            notify_of.extend(already_known.iter().filter(|_| { 
                                rng.gen_ratio(10.min(already_known.len() as u32), already_known.len() as u32)
                            }));
                            let message = Message { 
                                src: self.node.clone(),
                                dst: n.clone(),
                                body : Body { 
                                    id : None, 
                                    in_reply_to : None,
                                    payload : Payload::Gossip { 
                                        seen : notify_of
                                    }
                                }
                            };
                            message.send(&mut *output).with_context(|| format!("gossip to {}", n))?;
                        }
                        
                    }
                }
            },
            Event::EOF => { 
                // handle eof signal
            }
        } 
        
        Ok(())
    }
}


fn main() -> anyhow::Result<()> {
    eprintln!("node started");
    //let state = UniqueIdsNode { id : 0};
    main_loop::<_, BroadcastNode, _, InjectedPayload>(()).context("failed")
}
    