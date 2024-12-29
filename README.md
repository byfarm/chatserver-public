
run 
```bash
docker run --mount type=bind,source="$(pwd)",target=/workspace,consistency=cached -it espressif/idf-rust:esp32_latest /bin/bash
```
then `cd /workspace` inside the container.

run 
```bash
cargo add esp-idf
```
then run
```bash
cargo build
```
to build the project

you need to install espflash to flash it to the esp
```bash
sudo apt-get install libudev-dev
cargo install espflash
```

then run 
```bash
espflash flash target/debug/<binary> --monitor 
```
to flash to the thingy


add a file called `cfg.toml` to the home directry and insert
```toml
[hardware-check]
wifi_ssid = "WIFI Name"
wifi_psk = "WIFI Password"
```

may need to run this on your host machine to allow for editing within the container.
```bash
sudo chmod -R 777 $(pwd)
```

# Mission Chat Server

The server is currently running on the ThinkCenter in the living room, whose ip address (internal to our network) is 10.0.0.143. The TCP server is running on port 8023, in honor of Littleton's zip code, 80123. Suck on that.

To interact with it, `cd` into the `client` directory and run 

```bash
cargo build --release
```

This will create the binary `client/target/release/client`.

The two commands implemented are 
```bash
client recieve
```
to recieve messages from the server and 
```bash
client send <message>
```
to send messages to the server. The error handling is shit right now so don't break it.

