use std::io::{Write, ErrorKind};
use std::sync::{Arc, Mutex};
use std::rc::Rc;
use std::cell::RefCell;
use std::thread::{JoinHandle, spawn};
use std::mem::transmute;
use std::ptr::read;
use std::slice;
use mio::*;
use mio::tcp::{TcpListener, TcpStream};
use mio::util::Slab;
use bytes::Buf;
use rustc_serialize::json::encode;
use ::utils::config::Config;
use ::utils::pointer::to_cstring;
use ::store::table::{TableManager, AttrType};
use ::store::tuple::TupleData;
use super::queue::{BlockingQueueRef, BlockingQueue};
use super::handler::{sql_handler, ResultHandler, process_table_command};
use super::buf::Buffer;


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
    type Message = SenderMsg;

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
                        self.conn_list[token].lock().unwrap().init_reading_state(event_loop);
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
                    self.conn_list[token].lock().unwrap().deregister_all(event_loop);
                    is_match!(self.conn_list.remove(token), Some(..));
                }
            }
        }
    }

    fn notify(&mut self, event_loop: &mut EventLoop<SqlServer>, msg: SenderMsg) {
        let (token, curr_state, req_state) = msg;
        let mut conn = self.conn_list[token].lock().unwrap();
        match (curr_state, req_state) {
            (State::Writing, State::Writing) => conn.ensure_write_registered(event_loop),
            (State::Writing, State::Finished) => conn.transition_to_finished(event_loop),
            other => panic!("invalid request {:?}", other),
        }
    }
}

type ConnRef = Arc<Mutex<Connection>>;
type SenderMsg = (Token, State, State);

#[derive(Debug)]
pub struct Connection {
    socket : TcpStream,
    token : Token,
    sender : Sender<SenderMsg>,
    state : State,
    read_buf : Vec<u8>,
    write_buf : Buffer,
    event_added : bool,
}

impl Connection {
    fn new(socket: TcpStream, token: Token, sender : Sender<SenderMsg>) -> Connection {
        Connection {
            socket : socket,
            token : token,
            sender : sender,
            state : State::Reading,
            read_buf : Vec::new(),
            write_buf : Buffer::new(64),
            event_added : false,
        }
    }
}

impl Connection {
    fn write_buffer(&mut self, data : &[u8]) {
        check_ok!(self.write_buf.write(data));
        self.ensure_write_registered_in_loop();
    }

    fn get_state(&self) -> State {
        self.state.clone()
    }

    fn get_socket(&self) -> &TcpStream {
        &self.socket
    }

    fn get_sql(&self) -> String {
        assert_eq!(self.get_state(), State::Ready);
        let end = match self.read_buf.iter().position(|b| *b == b'\r') {
            Some(i) => i,
            None => self.read_buf.iter().position(|b| *b == b'\n').unwrap(),
        };
        String::from_utf8(Vec::from(&self.read_buf[..end])).unwrap()
    }

    fn dispatch(&mut self, event_loop: &mut EventLoop<SqlServer>, events: EventSet) {
        println!("connection state: {:?}", self.state);

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
                match e.kind() {
                    ErrorKind::ConnectionReset => {
                        println!("{:?}", e);
                        self.state = State::Closed;
                    },
                    _ => panic!("got an error trying to read; err={:?}", e),
                }
            }
        }
    }

    fn write(&mut self, event_loop: &mut EventLoop<SqlServer>) {
        match self.socket.try_write_buf(&mut self.write_buf) {
            Ok(Some(n)) => {
                println!("write {:?} bytes", n);
                if !self.write_buf.has_remaining() {
                    self.deregister_all(event_loop);
                }
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

    fn init_reading_state(&mut self, event_loop: &mut EventLoop<SqlServer>) {
        self.state = State::Reading;
        self.register_read(event_loop);
    }

    fn try_transition_to_ready(&mut self, event_loop: &mut EventLoop<SqlServer>) {
        assert_eq!(self.state, State::Reading);
        if let Some(..) = self.read_buf.iter().position(|b| *b == b'\n') {
            println!("change to ready");
            self.write_buf.reset();
            self.state = State::Ready;
            self.deregister_all(event_loop);
        }
    }

    fn change_to_finished_in_loop(&self) {
        assert_eq!(self.state, State::Writing);
        check_ok!(self.sender.send((self.token, State::Writing, State::Finished)));
    }

    fn register_write_in_loop(&self) {
        assert_eq!(self.state, State::Writing);
        check_ok!(self.sender.send((self.token, State::Writing, State::Writing)));
    }

    fn transition_to_writing(&mut self) {
        assert_eq!(self.state, State::Ready);
        println!("change to writing");
        self.state = State::Writing;
    }

    fn transition_to_finished(&mut self, event_loop : &mut EventLoop<SqlServer>) {
        assert_eq!(self.state, State::Writing);
        println!("change to finished");
        self.state = State::Finished;
        // this will trigger a writable event then call try_transition_to_reading
        // should not delete this!
        check_ok!(self.write_buf.write(b"\r\n"));
        if !self.event_added {
            self.register_write(event_loop);
        }
    }

    fn try_transition_to_reading(&mut self, event_loop : &mut EventLoop<SqlServer>) {
        if self.state == State::Finished && !self.write_buf.has_remaining() {
            println!("change to reading");
            self.read_buf.clear();
            self.state = State::Reading;
            self.register_read(event_loop);
        }
    }

    fn ensure_write_registered_in_loop(&mut self) {
        if !self.event_added {
            self.register_write_in_loop();
        }
    }

    fn ensure_write_registered(&mut self, event_loop : &mut EventLoop<SqlServer>) {
        if !self.event_added {
            self.register_write(event_loop);
        }
    }

    fn register_read(&mut self, event_loop : &mut EventLoop<SqlServer>) {
        assert!(!self.event_added);
        check_ok!(
            event_loop.register(self.get_socket(), self.token, EventSet::readable(), PollOpt::level())
        );
        self.event_added = true;
    }

    fn register_write(&mut self, event_loop : &mut EventLoop<SqlServer>) {
        assert!(!self.event_added);
        check_ok!(
            event_loop.register(self.get_socket(), self.token, EventSet::writable(), PollOpt::level())
        );
        self.event_added = true;
    }

    fn deregister_all(&mut self, event_loop : &mut EventLoop<SqlServer>) {
        assert!(self.event_added);
        check_ok!(
            event_loop.deregister(self.get_socket())
        );
        self.event_added = false;
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
    let config = Config::from_cwd_config();
    let mut manager = Rc::new(RefCell::new(TableManager::from_json_file(&config)));
    loop {
        let (sql, conn) = req_que.pop_front();
        conn.lock().unwrap().transition_to_writing();
        if let Ok(out) = process_table_command(&sql, &manager) {
            let mut c = conn.lock().unwrap();
            c.write_buffer(to_cstring(out).as_bytes());
            c.change_to_finished_in_loop();
        } else {
            println!("processing {:?}", sql);
            let mut process = Process::new(conn);
            sql_handler(&sql, &mut process, &mut manager);
        }
    }
}

#[derive(Debug)]
struct Process {
    attr_desc : Vec<AttrType>,
    attr_index : Vec<usize>,
    conn : ConnRef,
    header_sended : bool,
}

impl Process {
    pub fn new(conn : ConnRef) -> Process {
        Process{
            attr_desc : Vec::new(),
            attr_index : Vec::new(),
            conn : conn,
            header_sended : false,
        }
    }
    fn send_header(&mut self) {
        assert!(!self.header_sended);
        let json_header = encode(&self.attr_desc).unwrap();
        let json_len = json_header.len() as u32;
        let cstring = to_cstring(json_header);
        let len_bytes : [u8; 4] = unsafe { transmute(json_len.to_le()) };
        {
            let mut c = self.conn.lock().unwrap();
            c.write_buffer(&len_bytes);
            c.write_buffer(cstring.as_bytes());
        }
        self.header_sended = true;
    }
}

impl ResultHandler for Process {
    fn handle_non_query_finished(&mut self) {
        let non_query_header_tag : [u8; 4] = [0, 0, 0, 0];
        let mut c = self.conn.lock().unwrap();
        c.write_buffer(&non_query_header_tag);
        c.change_to_finished_in_loop();
    }
    fn handle_error(&mut self, err_msg : String) {
        let cstring = to_cstring(err_msg);
        let error_header_tag : [u8; 4] = [0, 0, 0, 0];
        let mut c = self.conn.lock().unwrap();
        c.write_buffer(&error_header_tag);
        c.write_buffer(cstring.as_bytes());
        c.change_to_finished_in_loop();
    }
    fn handle_tuple_data(&mut self, tuple_data : Option<TupleData>) {
        if !self.header_sended {
            self.send_header();
        }
        match tuple_data {
            Some(data) => {
                assert_eq!(self.attr_desc.len(), data.len());
                let mut c = self.conn.lock().unwrap();
                for (attr, p) in self.attr_desc.iter().zip(data.iter()) {
                    match attr {
                        &AttrType::Int | &AttrType::Float => {
                            let bytes = unsafe{read::<[u8; 4]>(*p as *const [u8; 4])};
                            c.write_buffer(&bytes);
                        }
                        &AttrType::Char{len} => {
                            let bytes : &[u8] = unsafe{ slice::from_raw_parts(*p as *const u8, len) };
                            c.write_buffer(bytes);
                        }
                    };
                }
            }
            None => self.conn.lock().unwrap().change_to_finished_in_loop(),
        }
    }
    fn set_tuple_info(&mut self, attr_desc : Vec<AttrType>, attr_index : Vec<usize>) {
        self.attr_desc = attr_desc;
        self.attr_index = attr_index;
    }
}

pub fn run_server() {
    let config = Config::from_cwd_config();
    let port = config.get_int("port");
    let addr = format!("0.0.0.0:{}", port).parse().unwrap();
    let listener = TcpListener::bind(&addr).unwrap();
    let mut event_loop = EventLoop::new().unwrap();

    event_loop.register(&listener, SERVER, EventSet::readable(),
                        PollOpt::level()).unwrap();
    let mut sqlserver = SqlServer::new(listener);
    event_loop.run(&mut sqlserver).unwrap();
}
