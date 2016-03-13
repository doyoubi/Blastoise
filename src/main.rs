extern crate blastoise;

fn main() {
    let mut client = blastoise::LocalClient;
    println!("starting Blastoise shell");
    client.shell_loop();
}
