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

# Set up project
* Install [rust](https://www.rust-lang.org/tools/install), which comes with [Cargo](https://doc.rust-lang.org/cargo/)
* Start TCP server at port 6379: `./spawn_redis_server.sh`
* Connect to TCP server: `nc localhost 6379`
* Run redis commands: PING, ECHO, GET, SET(support PX)
* Run tests: `cargo test --features integration_test`
  * Run only unit tests: `cargo test --features init_redis_test --lib`
  * Run only integration tests: `cargo test --features init_redis_test --test '*'`

**Sample commands**

PING
```
*1\r\n$4\r\n
ping\r\n
```
Output:
```
+PONG
```

ECHO
```
*2\r\n$4\r\n
echo\r\n$11\r\n
hello world\r\n
```
Output:
```
$11
hello world
```

SET
```
*3\r\n$3\r\nSET\r\n
$5\r\nhello\r\n
$5\r\nworld\r\n
```

Output:
```
$2
OK
```

SET with 5 seconds expiry
```
*5\r\n$3\r\nSET\r\n
$5\r\nhello\r\n
$5\r\nworld\r\n
$2\r\npx\r\n
$4\r\n5000\r\n
```
Output:
```
$2
OK
```

GET
```
*2\r\n$3\r\nget\r\n
$5\r\nhello\r\n
```

Output:
```
$5
world
```

GET key that is not found
```
*2\r\n$3\r\nget\r\n
$6\r\nrandom\r\n
```

Output:
```
$-1
```


# TODO
- [ ] Better separation of concern: Move parse and respond out of ClientInput
- [ ] Improve the parsing of user input. Currently, the program only parses an of array of bulk strings, and ignores the bytes part(i.e. $4 in $4\r\nping\r\n) in the bulk string request
- [ ] Replace threads with event loop