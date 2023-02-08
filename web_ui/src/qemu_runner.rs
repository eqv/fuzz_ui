
use actix::Actor;
use actix::prelude::*;

use app_state::AppState;

use queue_state::InputMeta;
use qemu_trace_loader::QemuTracer;
use snafu::{ErrorCompat};

pub struct QemuRunner{
    fuzz: AppState,
    qemu: QemuTracer,
}

impl QemuRunner{
    pub fn new(fuzz: AppState) -> Self{
        let qemu = QemuTracer::new(&fuzz.get().config);
        return Self{fuzz, qemu};
    }
}

impl Actor for QemuRunner {
    type Context = Context<Self>;
}

#[derive(Message)]
pub struct TraceInput{
    pub meta: InputMeta,
    pub raw_data: Vec<u8>,
}

impl Handler<TraceInput> for QemuRunner {
    type Result = ();

    fn handle(&mut self, msg: TraceInput, _: &mut Context<Self>) -> Self::Result {
        
        match self.qemu.perform_raw_trace(&msg.raw_data){
            Ok(raw_trace) => {
                self.fuzz.get_mut().insert_input_from_raw(msg.meta, raw_trace);
                self.fuzz.get_mut().cfg.perform_recursive_disassembly();
            },
            Err(e) =>  {         
                eprintln!("An error occurred when running qemu on client supplied raw dat: {}", e);
                if let Some(backtrace) = ErrorCompat::backtrace(&e) {
                    println!("{}", backtrace);
                }
            }
        }

    }
}