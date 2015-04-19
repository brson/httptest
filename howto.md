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

# 3. Routes

Next I want to POST some JSON, but before I do that I need a proper
URL to post to, so I guess I need to learn how to set up routes.

I look at the Iron [docs](http://ironframework.io/doc/iron/) and don't
see anything obvious in the main text, but there's a crate called
[router](http://ironframework.io/doc/router/index.html) that might be
interesting.

The module docs are "`Router` provides fast and flexible routing for
Iron", but not much else. How do I use a `Router`?! After I lot of
sleuthing I discover [an
example](https://github.com/iron/router/blob/master/examples/simple.rs).
OK, let's try to adapt that to our evolving experiment.

I add `router = "*"` to my `Cargo.toml` `[dependencies]` section,
and begin writing. The following is what I come up with
before getting stuck reading the POST data.

```rust
#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

extern crate iron;
extern crate router;
extern crate serde;

use iron::prelude::*;
use iron::status;
use router::Router;
use serde::json;

#[derive(Serialize, Deserialize)]
struct Greeting {
    msg: String
}

fn main() {
    let mut router = Router::new();

    router.get("/", hello_world);
    router.post("/set", set_greeting);

    fn hello_world(_: &mut Request) -> IronResult<Response> {
        let greeting = Greeting { msg: "Hello, World".to_string() };
        let payload = json::to_string(&greeting).unwrap();
        Ok(Response::with((status::Ok, payload)))
    }

    // Receive a message by POST and play it back.
    fn set_greeting(_: &mut Request) -> IronResult<Response> {
        let payload = unimplemented!(); // How to I get the POST body as string.
        let request: Greeting = json::from_str(payload).unwrap();
        let greeting = Greeting { msg: request.msg };
        let payload = json::to_string(&greeting).unwrap();
        Ok(Response::with((status::Ok, payload)))
    }

    Iron::new(router).http("localhost:3000").unwrap();
}
```

This uses `Router` to control handler dispatch. It builds and still
responds to `curl http://localhost:3000`, but the handler for the
`/set` route is yet unimplemented.

Now to read the POST body into a string. The [docs for
`Request`](http://ironframework.io/doc/iron/request/struct.Request.html)
say the field `body` is an iterator, so we just need to collect
that iterator into a string.

I first try `let payload = request.body.read_to_string();` because I
know it used to work.

It does not work.

```sh
brian@brian-ThinkPad-X1-Carbon-3rd:~/dev/httptest⟫ cargo build
Compiling httptest v0.1.0 (file:///opt/dev/httptest)
src/main.rs:30:36: 30:52 error: type `iron::request::Body<'_, '_>` does not implement any method in scope named `read_to_string`
src/main.rs:30         let payload = request.body.read_to_string();
                                                  ^~~~~~~~~~~~~~~~
src/main.rs:30:36: 30:52 help: methods from traits can only be called if the trait is in scope; the following trait is implemented but not in scope, perhaps add a `use` for it:
src/main.rs:30:36: 30:52 help: candidate #1: use `std::io::Read`
error: aborting due to previous error
Could not compile `httptest`.

To learn more, run the command again with --verbose.
```

I throw my hands up in disgust. 'Why does this method no longer exist?
The Rust team is always playing tricks on us!' Then I notice the
compiler has - at some length - explained that the method does exist
and that I should import `std::io::Read`.

I add the import and discover that `read_to_string` behaves
differently than I thought.

```sh
101 brian@brian-ThinkPad-X1-Carbon-3rd:~/dev/httptest⟫ cargo build
Compiling httptest v0.1.0 (file:///opt/dev/httptest)
src/main.rs:31:36: 31:52 error: this function takes 1 parameter but 0 parameters were supplied [E0061]
src/main.rs:31         let payload = request.body.read_to_string();
                                                  ^~~~~~~~~~~~~~~~
```

Ok, yeah the signature of `Read::read_to_string` is now `fn
read_to_string(&mut self, buf: &mut String) -> Result<usize, Error>`,
so that the buffer is supplied and errors handled. Rewrite the
`set_greeting` method.

```rust
    // Receive a message by POST and play it back.
    fn set_greeting(request: &mut Request) -> IronResult<Response> {
        let mut payload = String::new();
        request.body.read_to_string(&mut payload).unwrap();
        let request: Greeting = json::from_str(&payload).unwrap();
        let greeting = Greeting { msg: request.msg };
        let payload = json::to_string(&greeting).unwrap();
        Ok(Response::with((status::Ok, payload)))
    }
```

Let's run this and give it a curl.

```sh
brian@brian-ThinkPad-X1-Carbon-3rd:~⟫ curl -X POST -d '{"msg":"Just trust the Rust"}' http://localhost:3000/set
{"msg":"Just trust the Rust"}
```

Oh, Rust. You're just too bad.




[Crater]: https://github.com/brson/taskcluster-crater
[Rust]: https://github.com/brson/taskcluster-crater
[Iron]: http://ironframework.io/
[Hyper]: http://hyperium.github.io/hyper/hyper/index.html
[rustc-serialize]: https://crates.io/crates/rustc-serialize
[serde]: https://crates.io/crates/serde
[serde_macros]: https://crates.io/crates/serde_macros