# Polina Shell - GUI Shell in Rust

**Polina Shell** is a GUI shell written in Rust, created for the educational project _Конфигурационное управление (часть 1/1) [I.25-26]_.

## Build and Run
```
cargo build
cargo run
```
## How to run scripts? 
```
cargo build
./target/debug/polina-shell --storage ./storage --startapp ./storage/home.pl 
```

### Args
- `storage` - VFS file system
- `startapp` - pre-prepared shell script

## Dev info
- variant `1`
- group `IKBO-65-24`