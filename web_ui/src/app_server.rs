use std::collections::HashSet;
use actix::Actor;
use actix::prelude::*;

use app_state::AppState;
use websock::IjonWebSocket;


#[allow(dead_code)]
pub struct AppServer{
    fuzz: AppState,
    clients: HashSet::<Addr<IjonWebSocket>>,
}

impl AppServer{
    pub fn new(fuzz: AppState) -> Self{
        let clients = HashSet::new();
        return Self{fuzz, clients};
    }
}

impl Actor for AppServer {
    type Context = Context<Self>;
}


//Handle new connections
#[derive(Message)]
#[rtype(usize)]
pub struct RegisterListener(pub Addr<IjonWebSocket>);
impl Handler<RegisterListener> for AppServer {
    type Result = usize;

    fn handle(&mut self, msg: RegisterListener, _: &mut Context<Self>) -> Self::Result {
        self.clients.insert(msg.0);
        return 0;
    }
}

//Handle dropped connections
#[derive(Message)]
#[rtype(usize)]
pub struct RemoveListener(pub Addr<IjonWebSocket>);
impl Handler<RemoveListener> for AppServer {
    type Result = usize;

    fn handle(&mut self, msg: RemoveListener, _: &mut Context<Self>) -> Self::Result {
        self.clients.remove(&msg.0);
        return 0;
    }
}

//handle incoming text messages
#[derive(Message)]
pub struct Text(pub String);
impl Handler<Text> for AppServer {
    type Result = ();

    fn handle(&mut self, msg: Text, _: &mut Context<Self>) -> Self::Result {
        println!("Someone send {}", msg.0);
        return ();
    }
}

//handle updates from the server
#[derive(Message, Clone)]
pub struct UpdateClients(pub String);
impl Handler<UpdateClients> for AppServer {
    type Result = ();

    fn handle(&mut self, msg: UpdateClients, _: &mut Context<Self>) -> Self::Result {
         for c in self.clients.iter() {
            c.do_send(Text(msg.0.clone()));
        }
    }
}