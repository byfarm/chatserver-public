# Mission Chat Server

The server is currently running on the Raspberry Pi. Create an env file in the root of the repo and put `ADDRESS=10.0.0.134:6969` in the file (This is the network location and port of the Raspberry Pi internal to our network). Make sure you are connected to our wifi.

To interact with it, `cd` into the `client` directory and run 

```bash
cargo build --release
```

This will create the binary `client/target/release/client`.

The three commands implemented are 
```bash
client recieve
```
to recieve messages from the server,
```bash
client createuser <username>
```
To create your username. Users are based off their IP address and usernames must be unique.


```bash
client send <message>
```
Sends messages to the server. The error handling is shit right now so don't break it.

