mod debugger_command;
mod http_handlers;
mod breakpoint_map;

use std::thread;
use bus;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use serde_json;
use iron::prelude::*;
use router::Router;
use websocket::{Server as WsServer, Message as WsMessage, Sender as WsSender, Receiver as WsReceiver};

use super::Debugger;
use memory::{Memory, ADDRESSABLE_MEMORY};
use cpu::registers::Registers;
use cpu::disassembler::{InstructionDecoder, Instruction};
use self::debugger_command::{DebuggerCommand, BreakReason};
use self::http_handlers::{ToggleBreakpointHandler, ContinueHandler, StepHandler};
use self::breakpoint_map::BreakpointMap;

const DEBUGGER_HTTP_ADDR: &'static str = "127.0.0.1:9975";
const DEBUGGER_WS_ADDR: &'static str = "127.0.0.1:9976";

pub struct HttpDebugger {
    ws_sender: Option<Arc<Mutex<bus::Bus<DebuggerCommand>>>>,
    breakpoints: Arc<Mutex<BreakpointMap>>,
    cpu_thread_handle: thread::Thread,
    is_stepping: Arc<AtomicBool>,
    num_clients: Arc<AtomicUsize>
}

impl HttpDebugger {
    pub fn new() -> Self {
        HttpDebugger {
            breakpoints: Arc::new(Mutex::new(BreakpointMap::new())),
            ws_sender: None,
            cpu_thread_handle: thread::current(),
            is_stepping: Arc::new(AtomicBool::new(true)),
            num_clients: Arc::new(AtomicUsize::new(0))
        }
    }

    pub fn start(&mut self) -> Result<(), String> {
        if self.ws_sender.is_some() {
            panic!("Start already called.");
        }

        self.start_http_server_thread()?;
        self.start_websocket_thread()?;
        Ok(())
    }

    fn start_websocket_thread(&mut self) -> Result<(), String> {
        info!("Starting web socket server at {}", DEBUGGER_WS_ADDR);
        let bus = Arc::new(Mutex::new(bus::Bus::new(0)));

        self.ws_sender = Some(bus.clone());

        let mut ws_server = WsServer::bind(DEBUGGER_WS_ADDR).map_err(|e| e.to_string())?;
        let num_clients = self.num_clients.clone();
        let cpu_thread_handle = self.cpu_thread_handle.clone();

        thread::Builder::new().name("Websocket listener".into()).spawn(move || {

            info!("Listening for client connections...");

            loop {
                let bus = &mut (*bus.lock().unwrap());
                let client_rx = Arc::new(Mutex::new(bus.add_rx()));
                let num_clients = num_clients.clone();
                let cpu_thread_handle = cpu_thread_handle.clone();
                let connection = ws_server.accept().unwrap();
                let request = connection.read_request().unwrap();
                request.validate().unwrap();
                let response = request.accept();
                let (mut sender, mut receiver) = response.send().unwrap().split();
                let old_num_clients = num_clients.fetch_add(1, Ordering::SeqCst);
                let cur_client_num = old_num_clients + 1;
                thread::Builder::new().name(format!("Debugger client WS listener thread #{}", cur_client_num).into()).spawn(move || {
                    info!("Client #{} thread started", cur_client_num);
                    thread::Builder::new().name(format!("Debugger client #{} message listener", cur_client_num).into()).spawn(move || {
                        info!("Client #{} receiver thread started", cur_client_num);
                        let mut client_rx = &mut (*client_rx.lock().unwrap());
                        while let Ok(debugger_msg) = client_rx.recv() {
                            info!("Client #{} received message", cur_client_num);
                            let message: WsMessage = WsMessage::text(serde_json::to_string(&debugger_msg)
                                .unwrap());
                            if let Err(err) = sender.send_message(&message) {
                                warn!("Client #{} receiver error: {}", cur_client_num, err);
                                break;
                            }
                        }
                        info!("client #{} receiver thread exiting", cur_client_num)
                    }).unwrap();

                    for message in receiver.incoming_messages() {
                        let message: Result<WsMessage, _> = message;
                        if message.is_err() {
                            num_clients.fetch_sub(1, Ordering::SeqCst);
                            info!("Client #{} disconnected!", cur_client_num);
                            break;
                        } else {
                            info!("not err");
                        }
                    }

                    info!("Client #{} thread exiting", cur_client_num)
                }).unwrap();

                if old_num_clients == 0 {
                    info!("A debugger client has connection.  Unparking CPU thread.");
                    cpu_thread_handle.unpark();
                }
            }
        }).unwrap();

        Ok(())
    }

    fn start_http_server_thread(&self) -> Result<(), String> {
        info!("Starting http debugger at {}", DEBUGGER_HTTP_ADDR);
        let cpu_thread = self.cpu_thread_handle.clone();
        let breakpoints = self.breakpoints.clone();
        let is_stepping = self.is_stepping.clone();

        thread::spawn(move || {
            let mut router = Router::new();
            router.get("/step", StepHandler::new(cpu_thread.clone()), "step");
            router.get("/continue",
                       ContinueHandler::new(cpu_thread.clone(), is_stepping),
                       "continue");
            router.get("/toggle_breakpoint/:addr",
                       ToggleBreakpointHandler::new(breakpoints.clone()),
                       "toggle_breakpoint");
            Iron::new(router).http(DEBUGGER_HTTP_ADDR).unwrap();
        });

        Ok(())
    }

    fn should_break(&self, pc: u16) -> bool {
        let breakpoints = &(*self.breakpoints.lock().unwrap());
        if breakpoints.is_set(pc) {
            self.is_stepping.compare_and_swap(false, true, Ordering::Relaxed);
            true
        } else {
            false
        }
    }
}

impl Default for HttpDebugger {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Serialize, Clone)]
pub struct CpuSnapshot {
    instructions: Vec<Instruction>,
    registers: Registers,
    cycles: u64,
}

impl<M: Memory> Debugger<M> for HttpDebugger {
    fn on_step(&mut self, mem: &M, registers: &Registers, cycles: u64) {
        if let Some(ref sender) = self.ws_sender {
            let is_stepping = self.is_stepping.load(Ordering::Relaxed);
            if is_stepping || self.should_break(registers.pc) {
                {
                    let mut buf = Vec::with_capacity(ADDRESSABLE_MEMORY);
                    mem.dump(&mut buf);
                    let decoder = InstructionDecoder::new(&buf, 0x400);
                    let instructions = decoder.skip_while(|instr| instr.offset < registers.pc)
                        .take(100)
                        .collect::<Vec<Instruction>>();
                    let snapshot = CpuSnapshot {
                        instructions: instructions,
                        registers: registers.clone(),
                        cycles: cycles,
                    };

                    let is_breakpoint = self.should_break(registers.pc);
                    let break_reason = if is_breakpoint {
                        BreakReason::Breakpoint
                    } else {
                        BreakReason::Step
                    };

                    let mut sender = sender.lock().unwrap();
                    info!("broadcasting break!");
                    sender.broadcast(DebuggerCommand::Break(break_reason, snapshot));
                }
                info!("Breaking!  CPU thread paused.");
                thread::park();
                info!("CPU thread unparked!");
            }

            if self.num_clients.load(Ordering::Relaxed) == 0 {
                info!("No debugger clients connected. CPU thread paused.");
                thread::park();
            }
        }
    }
}
