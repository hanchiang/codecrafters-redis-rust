This is a starting point for Rust solutions to the
["Build Your Own Redis" Challenge](https://codecrafters.io/challenges/redis).

In this challenge, you'll build a toy Redis clone that's capable of handling
basic commands like `PING`, `SET` and `GET`. Along the way we'll learn about
event loops, the Redis protocol and more.

**Note**: If you're viewing this repo on GitHub, head over to
[codecrafters.io](https://codecrafters.io) to signup for early access.

# Passing the first stage

The entry point for your Redis implementation is in `src/main.rs`. Study and
uncomment the relevant code, and push your changes to pass the first stage:

```sh
git add .
git commit -m "pass 1st stage" # any msg
git push origin master
```

That's all!

# Stage 2 & beyond

Note: This section is for stages 2 and beyond.

1. Ensure you have `cargo (1.54)` installed locally
1. Run `./spawn_redis_server.sh` to run your Redis server, which is implemented
   in `src/main.rs`. This command compiles your Rust project, so it might be
   slow the first time you run it. Subsequent runs will be fast.
1. Commit your changes and run `git push origin master` to submit your solution
   to CodeCrafters. Test output will be streamed to your terminal.

# Project structure
* `src/`: The source code
  * `main.rs`: The binary crate
  * `lib.rs` The library crate
* `tests/`: Integration tests
  * `mod.rs`: Module exporters
  * `*.rs`: Test files
* `examples/`: Sample commands for client to send to the server

# Set up project
* Install [rust](https://www.rust-lang.org/tools/install), which comes with [Cargo](https://doc.rust-lang.org/cargo/)
* Start TCP server at port 6379: `./spawn_redis_server.sh` or `cargo run`
* Connect to TCP server: `nc localhost 6379`
* Enter redis commands: PING, ECHO, GET, SET(supports expiry with PX)
  * Alternatively, use the commands in `examples/`, e.g. `nc localhost 6379 < examples/ping.txt` 
* Run tests: `cargo test --features init_redis_test`
  * Run only unit tests: `cargo test --features init_redis_test --lib`
  * Run only integration tests: `cargo test --features init_redis_test --test '*'`

**Sample commands**

PING:
```
*1
$4
ping

```
or  
`nc localhost 6379 < examples/ping.txt`

Output:
```
+PONG
```

ECHO:
```
*2
$4
echo
$11
hello world

```

or  
`nc localhost 6379 < examples/echo.txt`

Output:
```
$11
hello world
```

SET
```
*3
$3
SET
$5
hello
$5
world

```

or  
`nc localhost 6379 < examples/set.txt`

Output:
```
$2
OK
```

SET with 5 seconds expiry
```
*5
$3
SET
$5
hello
$5
world
$2
px
$4
5000

```


or  
`nc localhost 6379 < examples/set_with_expiry.txt`

Output:
```
$2
OK
```

GET
```
*2
$3
get
$5
hello

```

or  
`nc localhost 6379 < examples/get.txt`

Output:
```
$5
world
```

GET a key that is not found
```
*2
$3
get
$6
random

```

or  
`nc localhost 6379 < examples/get_key_not_found.txt`

Output:
```
$-1
```


# TODO
- [ ] Better separation of concern: Move parse, respond out of ClientInput
- [x] Improve the parsing of user input. Currently, the program only parses an of array of bulk strings, and ignores the bytes part(i.e. $4 in $4\r\nping\r\n) in the bulk string request
- [ ] Validate parsed input
- [ ] Replace threads with event loop