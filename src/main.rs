use std::env;
extern crate blastoise;


fn main() {
    if let Some(ref opt) = env::args().nth(1) {
        if opt == "-c" {
            let mut client = blastoise::LocalClient;
            println!("starting Blastoise shell");
            client.shell_loop();
        } else {
            println!("invalid option {}, only support `-c`", opt);
        }
    } else {
        blastoise::run_server();
    }
}
