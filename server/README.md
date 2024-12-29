You must specify the ip config for the server. In a .env file at the root of the project, put
`ADDRESS=<ip address>:<port number>`

You can send a TCP request using Netcat:
`nc hostname port`

if we want to do http then we need to implement http headers into the request:
```
let status = "HTTP/1.1 200 OK";
let body ="<p>Hello World</p>";
let length = body.len();
let response = format!("{status}\r\nContent-Length: {length}\r\n\r\n{body}");
```
Is an example of how to send an http response. 
https://doc.rust-lang.org/book/ch20-01-single-threaded.html
