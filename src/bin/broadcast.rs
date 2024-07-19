

use std::{collections::HashMap, fmt::format, io::{BufRead, StdoutLock, Write}};

use anyhow::{bail, Context, Ok};
use serde::{de::DeserializeOwned, Serialize, Deserialize};
use whirlpool_malestorm_challenge::{Message , Body, Node, main_loop};


#[derive(Serialize , Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
pub enum Payload { 
    Broadcast { message : usize},
    BroadcastOk,
    Read,
    ReadOk { messages : Vec<usize>},
    Topology { topology : HashMap<String, Vec<String>>},
    TopologyOk
}





#[derive(Debug)]
pub struct BroadcastNode { 
    node : String,
    id : usize,
    messages : Vec<usize>
}

impl Node<(), Payload> for BroadcastNode 
{ 
    fn from_init(_state: (), init: whirlpool_malestorm_challenge::Init) -> anyhow::Result<Self> where Self:Sized {
      Ok(BroadcastNode{node : init.node_id, id: 1, messages : Vec::new()})  
    } 
    fn step(&mut self, input : Message<Payload>, output: &mut StdoutLock) -> anyhow::Result<()>{ 
        let mut reply = input.in_reply(Some(&mut self.id));
        match reply.body.payload { 
            Payload::Broadcast { message } => { 
                self.messages.push(message);
                reply.body.payload = Payload::BroadcastOk;
                serde_json::to_writer(&mut *output, &reply).context("serialize broadcast ok")?;
                output.write_all(b"\n");
            }, 
            Payload::Read => { 
                reply.body.payload = Payload::ReadOk { messages: self.messages.clone() };
                serde_json::to_writer(&mut *output, &reply).context("serialize broadcast ok")?;
                output.write_all(b"\n");
            },
            Payload::Topology { .. } => { 
                reply.body.payload = Payload::TopologyOk;
                serde_json::to_writer(&mut *output, &reply).context("serialize broadcast ok")?;
                output.write_all(b"\n");
            }, 
            Payload::BroadcastOk | Payload::ReadOk { .. } | Payload::TopologyOk=>{}
        }
        Ok(())
    }
}


fn main() -> anyhow::Result<()> {
    eprintln!("node started");
    //let state = UniqueIdsNode { id : 0};
    main_loop::<_, BroadcastNode, _>(()).context("failed")
}
    