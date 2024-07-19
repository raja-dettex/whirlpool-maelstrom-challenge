

use std::{fmt::format, io::{BufRead, StdoutLock, Write}};

use anyhow::{bail, Context, Ok};
use serde::{de::DeserializeOwned, Serialize, Deserialize};
use whirlpool_malestorm_challenge::{main_loop, Body, Event, Message, Node};


#[derive(Serialize , Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
pub enum Payload { 
    Generate, 
    GenerateOk {
        #[serde(rename="id")] 
        guid : String
    },
}





#[derive(Debug)]
pub struct UniqueIdsNode { 
    node : String,
    id : usize
}

impl Node<(), Payload> for UniqueIdsNode 
{ 
    fn from_init(_state: (), init: whirlpool_malestorm_challenge::Init , _tx_sender : std::sync::mpsc::Sender<Event<Payload, ()>>) -> anyhow::Result<Self> where Self:Sized {
      Ok(UniqueIdsNode{node : init.node_id, id: 1})  
    } 
    fn step(&mut self, input : Event<Payload, ()>, output: &mut StdoutLock) -> anyhow::Result<()>{ 
        let Event::Message(input_msg) = input else { 
            panic!("event injection happened here");
        };
        let mut reply = input_msg.in_reply(Some(&mut self.id));
        
        match reply.body.payload {
            Payload::Generate {  } => { 
                let guid = format!("{}-{}", self.node, self.id);
                reply.body.payload = Payload::GenerateOk { guid };
                serde_json::to_writer(&mut *output, &reply).context("failed to write echo message")?;
                output.write_all(b"\n").context("failed to write new line")?; 
                self.id += 1;
                eprintln!("init response sent , state : {:?}", self);
            },
            Payload::GenerateOk { .. } => {}
        
        }
        Ok(())
    }
}


fn main() -> anyhow::Result<()> {
    eprintln!("node started");
    //let state = UniqueIdsNode { id : 0};
    main_loop::<_, UniqueIdsNode, _, _>(()).context("failed")
}
