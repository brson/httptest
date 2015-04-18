% Making a web thing

# 1. Preparing to serve some JSON

So I'm working on this project [Crater] for doing [Rust] regression testing. Looking into the feasibility of writing parts in Rust. I need a HTTP server that talks to PostgreSQL and AMQP and servs JSON, and an HTTP client that talks JSON. I don't see a lot of docs on how to do this so here.

I don't know anything about using [Iron] or [Hyper]. I more-or-less know how to use Cargo.

I start by asking Cargo to give me a new executable project called 'httptest'. Passing `--bin` says
to create a source file in `src/main.rs` that will be compiled to an application.

```sh
$ cargo new httptest --bin
```

I'll use [Iron] for the server, so following the instructions in their docs add the following
to my `Cargo.toml` file.

```toml
[dependencies]
iron = "*"
```

Then into `main.rs` I just copy their example.

```rust
extern crate iron;

use iron::prelude::*;
use iron::status;

fn main() {
    fn hello_world(_: &mut Request) -> IronResult<Response> {
        Ok(Response::with((status::Ok, "Hello World!")))
    }

    Iron::new(hello_world).http("localhost:3000").unwrap();
    println!("On 3000");
}
```

And type `cargo build`.

```sh
brian@brian-ThinkPad-X1-Carbon-3rd:~/dev/httptest⟫ cargo build
Compiling hyper v0.3.13
Compiling iron v0.1.16
Compiling httptest v0.1.0 (file:///opt/dev/httptest)
```

Pure success so far. Damn, Rust is smooth. Let's try running the server with `cargo run`.

```sh
brian@brian-ThinkPad-X1-Carbon-3rd:~/dev/httptest⟫ cargo run
Running `target/debug/httptest`
```

I sit here waiting a while expecting it to print "On 3000" but it never does. Cargo
must be capturing output. Let's see if we're serving something.

```sh
brian@brian-ThinkPad-X1-Carbon-3rd:~⟫ curl http://localhost:3000
Hello World!
```

Oh, that's super cool. We know how to build a web server now. Good stopping point.

# 2. Serving a struct as JSON

Is [rustc-serialize] still the easiest way to convert to and from
JSON? Maybe I should use [serde], but then you really want
[serd_macros], but that only works on Rust nightlies.  Should I just
use nightlies? Nobody else is going to need to use this.

Fortune favors the bold. Let's go with serde and nightlies. Now my Cargo.toml
'dependencies' section looks like the following.

```toml
[dependencies]
iron = "*"
serde = "*"
serde_macros = "*"
```

Based on the [serde README](https://github.com/erickt/rust-serde) I update `main.rs`
to look like this:

```rust
#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

extern crate iron;
extern crate serde;

use iron::prelude::*;
use iron::status;
use serde::json;

#[derive(Serialize, Deserialize)]
struct Greeting {
    msg: String
}

fn main() {
    fn hello_world(_: &mut Request) -> IronResult<Response> {
        let greeting = Greeting { msg: "Hello, World".to_string() };
        let payload = json::to_string(&greeting).unwrap();
        Ok(Response::with(status::Ok, payload))
    }

    Iron::new(hello_world).http("localhost:3000").unwrap();
    println!("On 3000");
}
```

And then run `cargo build`.

```sh
brian@brian-ThinkPad-X1-Carbon-3rd:~/dev/httptest⟫ cargo build
Updating registry `https://github.com/rust-lang/crates.io-index`
Downloading quasi_macros v0.1.9
Downloading aster v0.2.0
Downloading serde_macros v0.3.1
Downloading num v0.1.22
Downloading quasi v0.1.9
Downloading rand v0.3.7
Downloading serde v0.3.1
Compiling quasi v0.1.9
Compiling aster v0.2.0
Compiling rand v0.3.7
Compiling num v0.1.22
Compiling quasi_macros v0.1.9
Compiling serde v0.3.1
Compiling serde_macros v0.3.1
Compiling httptest v0.1.0 (file:///opt/dev/httptest)
src/main.rs:20:12: 20:47 error: this function takes 1 parameter but 2 parameters were supplied [E0061]
src/main.rs:20         Ok(Response::with(status::Ok, payload))
                          ^~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
			  error: aborting due to previous error
			  Could not compile `httptest`.

To learn more, run the command again with --verbose.
```

Lot's of new dependencies now. But an error. [`Response::with`](http://ironframework.io/doc/iron/response/struct.Response.html#method.with) has this definition:

**fn with<M: Modifier<Response>>(m: M) -> Response**

I don't know what a `Modifier` is. [Some
docs](http://ironframework.io/doc/iron/modifiers/index.html) don't
help. I don't know what to do here but I notice that the original
example passed a tuple to `Response::with` whereas my update treated
`Response::with` as taking two parameters. It seems a tuple is a
`Modifier`.

Add the tuple to `Ok(Response::with((status::Ok, payload)))`, execute `cargo run`, curl some JSON.

```
brian@brian-ThinkPad-X1-Carbon-3rd:~⟫ curl http://localhost:3000
{"msg":"Hello, World"}
```

Blam! We're sending JSON. Time for another break.





[Crater]: https://github.com/brson/taskcluster-crater
[Rust]: https://github.com/brson/taskcluster-crater
[Iron]: http://ironframework.io/
[Hyper]: http://hyperium.github.io/hyper/hyper/index.html
[rustc-serialize]: https://crates.io/crates/rustc-serialize
[serde]: https://crates.io/crates/serde
[serde_macros]: https://crates.io/crates/serde_macros