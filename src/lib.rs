use core::panic;
use std::{io::{ BufRead, Read, StdoutLock, Write}, sync::Arc};

use anyhow::{ Context, Ok,};
use serde::{de::DeserializeOwned, Deserialize, Serialize};



#[derive(Serialize , Deserialize, Debug, Clone)]
pub struct Message<Payload> { 
    pub src: String,
    #[serde(rename="dest")]
    pub dst: String,
    pub body: Body<Payload>
}


#[derive(Serialize , Deserialize, Debug, Clone)]
pub struct Body<Payload> { 
    #[serde(rename = "msg_id")]
    pub id : Option<usize>,
    #[serde(rename = "in_reply_to")]
    pub in_reply_to : Option<usize>, 
    #[serde(flatten)]
    pub payload : Payload
}


#[derive(Serialize , Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
pub enum InitPayload { 
    Init(Init),
    InitOk
}




#[derive(Serialize , Deserialize, Debug, Clone)]
pub struct Init { 
    pub node_id : String,
    pub node_ids : Vec<String>
}



impl<Payload> Message<Payload> { 
    pub fn in_reply(self , id: Option<&mut usize>) -> Self { 
        Self { 
            src : self.dst,
            dst : self.src,
            body : Body { id: id.map(|id| { 
                let mid = *id; 
                *id += 1;
                mid
            }), 
                in_reply_to: self.body.id, payload: 
                self.body.payload }
        }
    }
}




pub trait Node<S,Payload> { 
    fn from_init(state: S, init: Init) -> anyhow::Result<Self> where Self:Sized;
    fn step(&mut self, input : Message<Payload>, output : &mut StdoutLock) -> anyhow::Result<()>;
}

pub fn main_loop<S, N , P>(init_state :  S) -> anyhow::Result<()>
where
    P : DeserializeOwned,
    N : Node<S, P>, 
{ 
        let stdin = std::io::stdin().lock();
        let mut stdout = std::io::stdout().lock();
        let reader = std::io::BufReader::new(stdin);
        let mut lines = reader.lines();
        let init_str = lines.next().expect("there shoud be an init msg").expect("first message should always be init");
        let init_msg: Message<InitPayload> = serde_json::from_str(&init_str)?;
        let InitPayload::Init(init) = init_msg.clone().body.payload else { 
            panic!("the first should always contain init message");
        };
        let mut node = N::from_init(init_state, init)?;
        let reply = Message { 
            src: init_msg.dst,
            dst: init_msg.src,
            body : Body { 
                id: Some(0),
                in_reply_to : init_msg.body.id,
                payload : InitPayload::InitOk
            }
        };
        serde_json::to_writer(&mut stdout, &reply).context("failed to write init message")?;
        stdout.write_all(b"\n").context("failed to write new line")?;
        for line in lines { 
            let input_str = line.context("failed to deserilalize input")?;
            
            let input: Message<P> = serde_json::from_str(&input_str).context("failed to deseralize input string")?;
            node.step(input, &mut stdout).context("Node step function failed")?;
        }
        Ok(())
}