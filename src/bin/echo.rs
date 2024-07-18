// use std::io::{self, BufRead, StdoutLock, Write};

// use anyhow::{bail, Context, Ok};
// use serde::{Deserialize, Serialize};



// #[derive(Serialize , Deserialize, Debug, Clone)]
// pub struct Message<Payload> { 
//     pub src: String,
//     #[serde(rename="dest")]
//     pub dst: String,
//     pub body: Body<Payload>
// }


// #[derive(Serialize , Deserialize, Debug, Clone)]
// pub struct Body<Payload> { 
//     #[serde(rename = "msg_id")]
//     id : Option<usize>,
//     #[serde(rename = "in_reply_to")]
//     in_reply_to : Option<usize>, 
//     #[serde(flatten)]
//     payload : Payload
// }


use std::io::{BufRead, StdoutLock, Write};

use anyhow::{Context, bail};
use serde::{de::DeserializeOwned, Serialize, Deserialize};
#[derive(Serialize , Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
pub enum Payload { 
    Echo { echo : String},
    EchoOk { echo : String },
    Init {node_id : String, node_ids : Vec<String>},
    InitOk
}
use whirlpool_malestorm_challenge::{Message , Body};


pub trait Node<Payload> { 
    fn step(&mut self, input : Message<Payload>, output : &mut StdoutLock) -> anyhow::Result<()>;
}

pub fn main_loop<S, Payload>(mut state_var :  S) -> anyhow::Result<()>
where S : Node<Payload>, 
    Payload : DeserializeOwned { 
        let stdin = std::io::stdin().lock();
        let mut stdout = std::io::stdout().lock();
        let reader = std::io::BufReader::new(stdin);
        for line in reader.lines() { 
            let input_str = line.context("failed to deserilalize input")?;
            let input: Message<Payload> = serde_json::from_str(&input_str).context("failed to deseralize input string")?;
            state_var.step(input, &mut stdout).context("Node step function failed")?;
        }
        Ok(())
}

#[derive(Debug)]
pub struct EachNode { 
    id : usize
}

impl Node<Payload> for EachNode 
{ 
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
            Payload::EchoOk { ref echo } => { eprintln!("Received EchoOk with echo: {}", echo); },
            Payload::InitOk => bail!("init ok")
        }
        Ok(())
    }
}


fn main() -> anyhow::Result<()> {
    eprintln!("node started");
    let state = EachNode { id : 0};
    main_loop(state).context("failed")
}
