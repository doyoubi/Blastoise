# Blastoise

Run server:
```bash
cargo run
```

Run client:
```bash
cargo run -- -c

> create table msg(id int not null primary, content char(233));
> insert msg values(1, "hello world");
> insert msg values(2, "hello doyoubi");
> insert msg values(3, "hello Blastoise");
> select * from msg where id % 2 = 1;

Ctrl-C to exit
```

