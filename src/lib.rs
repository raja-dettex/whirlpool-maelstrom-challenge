use core::panic;
use std::{io::{ BufRead, Read, StdoutLock, Write}, sync::Arc, thread};

use anyhow::{ Context, Ok,};
use std::result::Result::Ok as resultOK;
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
pub enum Event<Payload, InjectedPayload> { 
    Message(Message<Payload>),
    Injected(InjectedPayload),
    EOF
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
    pub fn send(self, output : &mut impl Write) -> anyhow::Result<()> 
    where Payload : Serialize
    { 
        serde_json::to_writer(&mut *output, &self).context("failed to write init message")?;
        output.write_all(b"\n").context("failed to write new line")?;
        Ok(())   
    }
}





pub trait Node<S,Payload, InjectedPayload=()> { 
    fn from_init(state: S, init: Init, sender : std::sync::mpsc::Sender<Event<Payload, InjectedPayload>>) -> anyhow::Result<Self> where Self:Sized;
    fn step(&mut self, input : Event<Payload, InjectedPayload>, output : &mut StdoutLock) -> anyhow::Result<()>;
}

pub fn main_loop<S, N , P, IP>(init_state :  S) -> anyhow::Result<()>
where
    P : DeserializeOwned + Send + std::marker::Sync + 'static,
    N : Node<S, P, IP>, 
    IP : Send + 'static
{ 
        
        let (tx , rx) = std::sync::mpsc::channel::<Event<P, IP>>();
        let mut stdin = std::io::stdin().lock();
        let mut stdin = stdin.lines();
        let mut stdout = std::io::stdout().lock();
        let init_str = stdin.next().expect("there shoud be an init msg").expect("first message should always be init");
        let init_msg: Message<InitPayload> = serde_json::from_str(&init_str)?;
        let InitPayload::Init(init) = init_msg.clone().body.payload else { 
            panic!("the first should always contain init message");
        };
        let mut node = N::from_init(init_state, init, tx.clone())?;
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
        drop(stdin);
        let stdin_tx = tx.clone();
        let jh = thread::spawn(move || -> anyhow::Result<()>{
            let mut stdin = std::io::stdin();
            let mut stdin = stdin.lines(); 
            for line in stdin { 
                let input_str = line.context("failed to parse input message")?;
                let input : Message<P> = serde_json::from_str((&input_str)).context("failed to deserialize input lines")?;
                if let Err(_) = stdin_tx.send(Event::Message(input)) { 
                    return Ok(());
                }
            };
            stdin_tx.send(Event::EOF);
            Ok(())
        });
        for input in rx { 
            node.step(input, &mut stdout);
        }
        jh.join().expect("thread panicked").context("error")?;
        Ok(())
}