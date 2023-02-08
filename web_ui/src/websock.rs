use std::time::{Duration, Instant};

use actix::*;
use actix_web::{ web,Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;

use app_server::{AppServer, RegisterListener, RemoveListener, Text};

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(2);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(5);

pub fn ws_create(r: HttpRequest, stream: web::Payload, server: web::Data<Addr<AppServer>>) -> Result<HttpResponse, Error> {
    let addr = server.get_ref().clone();
    let ws = ws::start(IjonWebSocket::new(addr), &r, stream);
    return ws;
}

pub struct IjonWebSocket {
    hb: Instant,
    state: Addr<AppServer>,
}

impl IjonWebSocket {
    fn new(state: Addr<AppServer>) -> Self {
        IjonWebSocket { hb: Instant::now(),  state}
    }

    fn hb(&self, ctx: &mut <Self as Actor>::Context) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                ctx.stop();
                return;
            }
            ctx.ping("");
        });
    }

    fn text_msg(&self, txt: String, _ctx: &mut <Self as Actor>::Context){
        let _id=self.state.do_send(Text(txt));
    }

}

impl Handler<Text> for IjonWebSocket {
    type Result = ();

    fn handle(&mut self, msg: Text, ctx: &mut <Self as Actor>::Context ) -> Self::Result {
        ctx.text(msg.0);
        return ();
    }
}


impl Actor for IjonWebSocket {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);
        self.state.do_send(RegisterListener(ctx.address()));
    }
    fn stopped(&mut self, ctx: &mut Self::Context) {
        self.state.do_send(RemoveListener(ctx.address()));
    }
}

impl StreamHandler<ws::Message, ws::ProtocolError> for IjonWebSocket {
    fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
        match msg {
            ws::Message::Ping(msg) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            ws::Message::Pong(_) => {
                self.hb = Instant::now();
            }
            ws::Message::Text(text) => {self.text_msg(text.to_owned(), ctx); ctx.text(text)},
            ws::Message::Binary(bin) => ctx.binary(bin),
            ws::Message::Close(_) => {
                ctx.stop();
            }
            ws::Message::Nop => (),
        }
    }
}

