use std::io::Write;
use std::sync::{Arc, Mutex};
use std::thread::{JoinHandle, spawn};
use mio::*;
use mio::tcp::{TcpListener, TcpStream};
use mio::util::Slab;
use bytes::{Buf, RingBuf};
// use ::utils::config::Config;
// use ::store::table::TableManager;
// use super::local_client::Process;
use super::queue::{BlockingQueueRef, BlockingQueue};


const SERVER : Token = Token(0);
type TaskQueueRef = BlockingQueueRef<(String, ConnRef)>;

struct SqlServer {
    listener : TcpListener,
    conn_list : Slab<ConnRef>,
    req_que : TaskQueueRef,
    worker : JoinHandle<()>,
}

impl SqlServer {
    fn new(listener : TcpListener) -> Self {
        let q = BlockingQueueRef::new(BlockingQueue::new(64));
        let q_clone = q.clone();
        let worker = spawn(|| {
            consume_task_loop(q_clone);
        });
        SqlServer{
            listener : listener,
            conn_list : Slab::new_starting_at(Token(1), 1024),
            req_que : q,
            worker : worker,
        }
    }
}

impl Handler for SqlServer {
    type Timeout = ();
    type Message = (Token, State);

    fn ready(&mut self, event_loop : &mut EventLoop<SqlServer>, token : Token, events : EventSet) {
        match token {
            SERVER => {
                assert!(events.is_readable());
                println!("server accept");
                match self.listener.accept() {
                    Ok(Some((socket, _))) => {
                        println!("accepted a new client socket");
                        let token = self.conn_list
                            .insert_with(|token| Arc::new(Mutex::new(Connection::new(
                                socket, token, event_loop.channel()
                                ))))
                            .unwrap();
                        event_loop.register(
                            self.conn_list[token].lock().unwrap().get_socket(),
                            token,
                            EventSet::readable(),
                            PollOpt::level()).unwrap();
                    }
                    Ok(None) => { println!("the server socket wasn't actually ready"); },
                    Err(e) => {
                        println!("encountered error while accepting connection; err={:?}", e);
                        event_loop.shutdown();
                    }
                }
            }
            _ => {
                let mut closed = false;
                {
                    let clone = self.conn_list[token].clone();
                    let mut conn = self.conn_list[token].lock().unwrap();
                    conn.dispatch(event_loop, events);
                    match conn.get_state() {
                        State::Ready => {
                            let sql = conn.get_sql();
                            self.req_que.push_back((sql, clone));
                        }
                        State::Closed => closed = true,
                        _ => (),
                    }
                }
                if closed {
                    is_match!(self.conn_list.remove(token), Some(..));
                }
            }
        }
    }

    fn notify(&mut self, event_loop: &mut EventLoop<SqlServer>, msg: (Token, State)) {
        let (token, state) = msg;
        let mut conn = self.conn_list[token].lock().unwrap();
        match state {
            State::Writing => conn.transition_to_writing(event_loop),
            State::Finished => conn.transition_to_finished(),
            _ => panic!("invalid state {:?}", state),
        }
    }
}

type ConnRef = Arc<Mutex<Connection>>;

#[derive(Debug)]
pub struct Connection {
    socket : TcpStream,
    token : Token,
    sender : Sender<(Token, State)>,
    state : State,
    read_buf : Vec<u8>,
    write_buf : RingBuf,
}

impl Connection {
    fn new(socket: TcpStream, token: Token, sender : Sender<(Token, State)>) -> Connection {
        Connection {
            socket : socket,
            token : token,
            sender : sender,
            state : State::Reading,
            read_buf : Vec::new(),
            write_buf : RingBuf::new(64),
        }
    }
}

impl Connection {
    fn get_output_buf(&mut self) -> &mut RingBuf {
        &mut self.write_buf
    }

    fn get_state(&self) -> State {
        self.state.clone()
    }

    fn get_socket(&self) -> &TcpStream {
        &self.socket
    }

    fn get_sql(&self) -> String {
        assert_eq!(self.get_state(), State::Ready);
        let end = self.read_buf.iter().position(|b| *b == b'\n').unwrap();
        String::from_utf8(Vec::from(&self.read_buf[..end])).unwrap()
    }

    fn dispatch(&mut self, event_loop: &mut EventLoop<SqlServer>, events: EventSet) {
        println!("    connection-state={:?}", self.state);

        match self.state {
            State::Reading => {
                assert!(events.is_readable(), "unexpected events; events={:?}", events);
                self.read(event_loop)
            }
            State::Writing | State::Finished => {
                assert!(events.is_writable(), "unexpected events; events={:?}", events);
                self.write(event_loop)
            }
            _ => panic!("invalid state {:?}", self.state),
        }
    }

    fn read(&mut self, event_loop : &mut EventLoop<SqlServer>) {
        assert_eq!(self.state, State::Reading);
        match self.socket.try_read_buf(&mut self.read_buf) {
            Ok(Some(0)) => {
                println!("read 0 bytes from client; buffered={}", self.read_buf.len());
                assert_eq!(self.state, State::Reading);
                self.state = State::Closed;
            }
            Ok(Some(n)) => {
                println!("read {} bytes", n);
                self.try_transition_to_ready(event_loop);
            }
            Ok(None) => {
                println!("nothing read");
            }
            Err(e) => {
                panic!("got an error trying to read; err={:?}", e);
            }
        }
    }

    fn write(&mut self, event_loop: &mut EventLoop<SqlServer>) {
        match self.socket.try_write_buf(&mut self.write_buf) {
            Ok(Some(n)) => {
                println!("write {:?} bytes", n);
                self.try_transition_to_reading(event_loop);
            }
            Ok(None) => {
                println!("nothing write");
            }
            Err(e) => {
                panic!("got an error trying to write; err={:?}", e);
            }
        }
    }

    fn try_transition_to_ready(&mut self, event_loop: &mut EventLoop<SqlServer>) {
        assert_eq!(self.state, State::Reading);
        if let Some(..) = self.read_buf.iter().position(|b| *b == b'\n') {
            println!("change to ready");
            self.write_buf.clear();
            self.state = State::Ready;
            check_ok!(
                event_loop.deregister(self.get_socket())
            );
        }
    }

    fn change_to_writing_in_loop(&self) {
        check_ok!(self.sender.send((self.token, State::Writing)));
    }

    fn change_to_finished_in_loop(&self) {
        check_ok!(self.sender.send((self.token, State::Finished)));
    }

    fn transition_to_writing(&mut self, event_loop : &mut EventLoop<SqlServer>) {
        assert_eq!(self.state, State::Ready);
        println!("change to writing");
        self.state = State::Writing;
        check_ok!(
            event_loop.register(self.get_socket(), self.token, EventSet::writable(), PollOpt::level())
            );
    }

    fn transition_to_finished(&mut self) {
        assert_eq!(self.state, State::Writing);
        println!("change to finished");
        self.state = State::Finished;
        // this will trigger a writable event then call try_transition_to_reading
        // should not delete this!
        check_ok!(self.write_buf.write(b"\n"));
    }

    fn try_transition_to_reading(&mut self, event_loop : &mut EventLoop<SqlServer>) {
        if self.state == State::Finished && !self.write_buf.has_remaining() {
            println!("change to reading");
            self.read_buf.clear();
            self.state = State::Reading;
            check_ok!(
                event_loop.reregister(self.get_socket(), self.token, EventSet::readable(), PollOpt::level())
                );
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
enum State {
    Reading,
    Ready,
    Writing,
    Finished,
    Closed,
}

fn consume_task_loop(req_que : TaskQueueRef) {
    // let config = Config::from_cwd_config();
    // let mut manager = Rc::new(RefCell::new(TableManager::from_json_file(&config)));
    // let mut process = Process::new();
    loop {
        let (_sql, conn) = req_que.pop_front();
        let mut c = conn.lock().unwrap();
        check_ok!(c.get_output_buf().write(b"ok!\n"));
        assert_eq!(c.get_state(), State::Ready);
        c.change_to_writing_in_loop();
        c.change_to_finished_in_loop();
    }
}

pub fn run_server() {
    let addr = "127.0.0.1:8080".parse().unwrap();
    let listener = TcpListener::bind(&addr).unwrap();
    let mut event_loop = EventLoop::new().unwrap();

    event_loop.register(&listener, SERVER, EventSet::readable(),
                        PollOpt::level()).unwrap();
    let mut sqlserver = SqlServer::new(listener);
    event_loop.run(&mut sqlserver).unwrap();
}
