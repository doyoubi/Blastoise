use std::io::{Write, Cursor, SeekFrom, Seek};
use mio::*;
use mio::tcp::{TcpListener, TcpStream};
use mio::util::Slab;
use bytes::Buf;


const SERVER : Token = Token(0);

struct SqlServer {
    listener : TcpListener,
    conn_list : Slab<Connection>,
}

impl SqlServer {
    fn new(listener : TcpListener) -> Self {
        SqlServer{
            listener : listener,
            conn_list : Slab::new_starting_at(Token(1), 1024),
        }
    }
}

impl Handler for SqlServer {
    type Timeout = ();
    type Message = ();

    fn ready(&mut self, event_loop : &mut EventLoop<SqlServer>, token : Token, events : EventSet) {
        match token {
            SERVER => {
                assert!(events.is_readable());
                println!("server accept");
                match self.listener.accept() {
                    Ok(Some((socket, _))) => {
                        println!("accepted a new client socket");
                        let token = self.conn_list
                            .insert_with(|token| Connection::new(socket, token))
                            .unwrap();
                        event_loop.register(
                            &self.conn_list[token].socket,
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
                self.conn_list[token].ready(event_loop, events);
                if self.conn_list[token].is_closed() {
                    let _ = self.conn_list.remove(token);
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct Connection {
    socket: TcpStream,
    token: Token,
    state: State,
    read_buf : Vec<u8>,
    write_buf : Cursor<Vec<u8>>,
}

impl Connection {
    fn new(socket: TcpStream, token: Token) -> Connection {
        Connection {
            socket: socket,
            token: token,
            state: State::Reading,
            read_buf : Vec::new(),
            write_buf : Cursor::new(Vec::new()),
        }
    }
}

impl Connection {
    fn ready(&mut self, event_loop: &mut EventLoop<SqlServer>, events: EventSet) {
        println!("    connection-state={:?}", self.state);

        match self.state {
            State::Reading => {
                assert!(events.is_readable(), "unexpected events; events={:?}", events);
                self.read(event_loop)
            }
            State::Writing => {
                assert!(events.is_writable(), "unexpected events; events={:?}", events);
                self.write(event_loop)
            }
            State::Closed => panic!("should not be here"),
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
                self.try_transition_to_writing(event_loop);
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

    fn try_transition_to_writing(&mut self, event_loop : &mut EventLoop<SqlServer>) {
        if let Some(..) = self.read_buf.iter().position(|b| *b == b'\n') {
            check_ok!(self.write_buf.seek(SeekFrom::Start(0)));
            check_ok!(self.write_buf.write(b"ok!\n"));
            check_ok!(self.write_buf.seek(SeekFrom::Start(0)));
            self.state = State::Writing;
            check_ok!(
                event_loop.reregister(&self.socket, self.token, EventSet::writable(), PollOpt::level())
                );
        }
    }

    fn try_transition_to_reading(&mut self, event_loop : &mut EventLoop<SqlServer>) {
        if !self.write_buf.has_remaining() {
            self.read_buf.clear();
            self.state = State::Reading;
            check_ok!(
                event_loop.reregister(&self.socket, self.token, EventSet::readable(), PollOpt::level())
                );
        }
    }

    fn is_closed(&self) -> bool {
        self.state == State::Closed
    }
}

#[derive(Debug, Eq, PartialEq)]
enum State {
    Reading,
    Writing,
    Closed,
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
