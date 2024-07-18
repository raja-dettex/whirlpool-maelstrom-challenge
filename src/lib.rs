use std::io::{ BufRead, StdoutLock};

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
pub struct Init { 
    pub node_id : String,
    pub node_ids : Vec<String>
}

