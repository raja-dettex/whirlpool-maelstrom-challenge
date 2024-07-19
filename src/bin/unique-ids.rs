

use std::{fmt::format, io::{BufRead, StdoutLock, Write}};

use anyhow::{bail, Context, Ok};
use serde::{de::DeserializeOwned, Serialize, Deserialize};
use whirlpool_malestorm_challenge::{Message , Body, Node, main_loop};


#[derive(Serialize , Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
pub enum Payload { 
    Generate, 
    GenerateOk {
        #[serde(rename="id")] 
        guid : String
    },
    Echo { echo : String},
    EchoOk { echo : String },
    Init {node_id : String, node_ids : Vec<String>},
    InitOk
}





#[derive(Debug)]
pub struct UniqueIdsNode { 
    node : String,
    id : usize
}

impl Node<(), Payload> for UniqueIdsNode 
{ 
    fn from_init(_state: (), init: whirlpool_malestorm_challenge::Init) -> anyhow::Result<Self> where Self:Sized {
      Ok(UniqueIdsNode{node : init.node_id, id: 1})  
    } 
    fn step(&mut self, input : Message<Payload>, output: &mut StdoutLock) -> anyhow::Result<()>{ 
        match input.body.payload {
            Payload::Init { ref node_id, ref node_ids } => { 
                eprintln!("Handling Init payload: node_id={}, node_ids={:?}", node_id, node_ids);
                let reply = Message { 
                    src: input.dst,
                    dst: input.src,
                    body : Body { id: Some(self.id), in_reply_to: input.body.id, payload: Payload::InitOk  }
                };
                serde_json::to_writer(&mut *output, &reply).context("failed to handle initOk")?;
                output.write_all(b"\n").context("failed to write new line")?;
                //output.flush().context("failed to flush output");
                self.id += 1;
                eprintln!("init response sent , state : {:?}", self);
            },
            Payload::Echo { echo } => { 
                let reply = Message { 
                    src: input.dst,
                    dst: input.src,
                    body : Body { id: Some(self.id), in_reply_to: input.body.id, payload: Payload::EchoOk { echo: "echo_ok".to_string() } }
                };
                serde_json::to_writer(&mut *output, &reply).context("failed to write echo message")?;
                output.write_all(b"\n").context("failed to write new line")?; 
                self.id += 1;
                eprintln!("init response sent , state : {:?}", self);
            },
            Payload::Generate {  } => { 
                let guid = format!("{}-{}", self.node, self.id);
                let reply = Message { 
                    src: input.dst,
                    dst: input.src,
                    body : Body { id: Some(self.id), in_reply_to: input.body.id, payload: Payload::GenerateOk { guid  }  }
                };
                serde_json::to_writer(&mut *output, &reply).context("failed to write echo message")?;
                output.write_all(b"\n").context("failed to write new line")?; 
                self.id += 1;
                eprintln!("init response sent , state : {:?}", self);
            },
            Payload::EchoOk { ref echo } => { eprintln!("Received EchoOk with echo: {}", echo); },
            Payload::InitOk => bail!("init ok"),
            Payload::GenerateOk { .. } => {}
        
        }
        Ok(())
    }
}


fn main() -> anyhow::Result<()> {
    eprintln!("node started");
    //let state = UniqueIdsNode { id : 0};
    main_loop::<_, UniqueIdsNode, _>(()).context("failed")
}
