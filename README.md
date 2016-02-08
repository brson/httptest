# Let's make a web service and client in Rust

So I'm working on this project [Crater] for doing [Rust] regression testing. Looking into the feasibility of writing parts in Rust. I need an HTTP server that speaks JSON, along with a corresponding client. I don't see a lot of docs on how to do this so I'm recording my investigation.

I'm going to use [Iron] and [Hyper], neither of which I have experience with.

Each commit in this repo corresponds with a chapter, so follow along if you want.

**Edit: Some inaccuracies within! [Thanks /r/rust!](http://www.reddit.com/r/rust/comments/33k1yn/lets_make_a_web_service_and_client/)**

# 1. Preparing to serve some JSON

I start by asking Cargo to give me a new executable project called 'httptest'. Passing `--bin` says
to create a source file in `src/main.rs` that will be compiled to an application.

```text
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

```text
$ cargo build
Compiling hyper v0.3.13
Compiling iron v0.1.16
Compiling httptest v0.2.0 (file:///opt/dev/httptest)
```

Pure success so far. Damn, Rust is smooth. Let's try running the server with `cargo run`.

```text
$ cargo run
Running `target/debug/httptest`
```

I sit here waiting a while expecting it to print "On 3000" but it never does. Cargo
must be capturing output. Let's see if we're serving something.

```text
$ curl http://localhost:3000
Hello World!
```

Oh, that's super cool. We know how to build a web server now. Good starting point.

# 2. Serving a struct as JSON

Is [rustc-serialize] still the easiest way to convert to and from
JSON? Maybe I should use [serde], but then you really want
[serde_macros], but that only works on Rust nightlies.  Should I just
use nightlies? Nobody else is going to need to use this.

Fortune favors the bold. Let's go with serde and nightlies. Now my Cargo.toml
'dependencies' section looks like the following.

```toml
[dependencies]
iron = "*"
rustc-serialize = "*"
```

Based on the [rustc-serialize docs](https://doc.rust-lang.org/rustc-serialize/rustc_serialize/json/index.html) I update `main.rs`
to look like this:

```rust
extern crate iron;
extern crate rustc_serialize;

use iron::prelude::*;
use iron::status;
use rustc_serialize::json;

#[derive(RustcEncodable)]
struct Greeting {
    msg: String
}

fn main() {
    fn hello_world(_: &mut Request) -> IronResult<Response> {
        let greeting = Greeting { msg: "Hello, World".to_string() };
        let payload = json::encode(&greeting).unwrap();
        Ok(Response::with((status::Ok, payload)))
    }

    Iron::new(hello_world).http("localhost:3000").unwrap();
    println!("On 3000");
}
```

And then run `cargo build`.

```text
$ cargo build
   Updating registry `https://github.com/rust-lang/crates.io-index`
 Downloading unicase v1.1.1
 <... more downloading here ...>
   Compiling lazy_static v0.1.15
 <... more compiling here ...>
   Compiling httptest v0.2.0 (file:///opt/dev/httptest)
src/main.rs:17:12: 17:47 error: this function takes 1 parameter but 2 parameters were supplied [E0061]
src/main.rs:17         Ok(Response::with(status::Ok, payload))
                          ^~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
src/main.rs:17:12: 17:47 help: run `rustc --explain E0061` to see a detailed explanation
error: aborting due to previous error
Could not compile `httptest`.

To learn more, run the command again with --verbose.
```

Lot's of new dependencies now. But an error. [`Response::with`](http://ironframework.io/doc/iron/response/struct.Response.html#method.with) has this definition:

```rust
fn with<M: Modifier<Response>>(m: M) -> Response
```

I don't know what a `Modifier` is. [Some
docs](http://ironframework.io/doc/iron/modifiers/index.html) don't
help. I don't know what to do here but I notice that the original
example passed a tuple to `Response::with` whereas my update treated
`Response::with` as taking two parameters. It seems a tuple is a
`Modifier`.

Add the tuple to `Ok(Response::with((status::Ok, payload)))`, execute `cargo run`, curl some JSON.

```
$ curl http://localhost:3000
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
extern crate iron;
extern crate router;
extern crate rustc_serialize;

use iron::prelude::*;
use iron::status;
use router::Router;
use rustc_serialize::json;

#[derive(RustcEncodable, RustcDecodable)]
struct Greeting {
    msg: String
}

fn main() {
    let mut router = Router::new();

    router.get("/", hello_world);
    router.post("/set", set_greeting);

    fn hello_world(_: &mut Request) -> IronResult<Response> {
        let greeting = Greeting { msg: "Hello, World".to_string() };
        let payload = json::encode(&greeting).unwrap();
        Ok(Response::with((status::Ok, payload)))
    }

    // Receive a message by POST and play it back.
    fn set_greeting(request: &mut Request) -> IronResult<Response> {
        let payload = request.body.read_to_string();
        let request: Greeting = json::decode(payload).unwrap();
        let greeting = Greeting { msg: request.msg };
        let payload = json::encode(&greeting).unwrap();
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

```text
$ cargo build
Compiling httptest v0.2.0 (file:///opt/dev/httptest)
src/main.rs:29:36: 29:52 error: no method named `read_to_string` found for type `iron::request::Body<'_, '_>` in the current scope
src/main.rs:29         let payload = request.body.read_to_string();
                                                  ^~~~~~~~~~~~~~~~
src/main.rs:29:36: 29:52 help: items from traits can only be used if the trait is in scope; the following trait is implemented but not in scope, perhaps add a `use` for it:
src/main.rs:29:36: 29:52 help: candidate #1: use `std::io::Read`
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

```text
101 $ cargo build
Compiling httptest v0.2.0 (file:///opt/dev/httptest)
src/main.rs:30:36: 30:52 error: this function takes 1 parameter but 0 parameters were supplied [E0061]
src/main.rs:30         let payload = request.body.read_to_string();
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
        let request: Greeting = json::decode(&payload).unwrap();
        let greeting = Greeting { msg: request.msg };
        let payload = json::encode(&greeting).unwrap();
        Ok(Response::with((status::Ok, payload)))
    }
```

Let's run this and give it a curl.

```text
$ curl -X POST -d '{"msg":"Just trust the Rust"}' http://localhost:3000/set
{"msg":"Just trust the Rust"}
```

Oh, Rust. You're just too bad.

# 4. Mutation

Hey, I know all those `.unwrap()`s are wrong. I don't care. We're prototyping.

Before we continue on to writing a client, I want to modify this toy example
to store some state on POST to `/set` and report it later. I'll make `greeting`
a local, capture it in some closures, then see how the compiler complains.

Here's my new `main` function, before attempting to compile:

```rust
fn main() {
    let mut greeting = Greeting { msg: "Hello, World".to_string() };

    let mut router = Router::new();

    router.get("/", |r| hello_world(r, &greeting));
    router.post("/set", |r| set_greeting(r, &mut greeting));

    fn hello_world(_: &mut Request, greeting: &Greeting) -> IronResult<Response> {
        let payload = json::encode(&greeting).unwrap();
        Ok(Response::with((status::Ok, payload)))
    }

    // Receive a message by POST and play it back.
    fn set_greeting(request: &mut Request, greeting: &mut Greeting) -> IronResult<Response> {
        let mut payload = String::new();
        request.body.read_to_string(&mut payload).unwrap();
        *greeting = json::decode(&payload).unwrap();
        Ok(Response::with(status::Ok))
    }

    Iron::new(router).http("localhost:3000").unwrap();
}
```

```text
src/main.rs:21:12: 21:51 error: type mismatch resolving `for<'r, 'r, 'r> <[closure@src/main.rs:21:21: 21:50 greeting:_] as core::ops::FnOnce<(&'r mut iron::request::Request<'r, 'r>,)>>::Output == core::result::Result<iron::response::Response, iron::error::IronError>`:
 expected bound lifetime parameter ,
    found concrete lifetime [E0271]
src/main.rs:21     router.get("/", |r| hello_world(r, &greeting));
                          ^~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
src/main.rs:21:12: 21:51 help: run `rustc --explain E0271` to see a detailed explanation
src/main.rs:22:12: 22:60 error: type mismatch resolving `for<'r, 'r, 'r> <[closure@src/main.rs:22:25: 22:59 greeting:_] as core::ops::FnOnce<(&'r mut iron::request::Request<'r, 'r>,)>>::Output == core::result::Result<iron::response::Response, iron::error::IronError>`:
 expected bound lifetime parameter ,
    found concrete lifetime [E0271]
src/main.rs:22     router.post("/set", |r| set_greeting(r, &mut greeting));
                          ^~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
src/main.rs:22:12: 22:60 help: run `rustc --explain E0271` to see a detailed explanation
src/main.rs:21:12: 21:51 error: type mismatch: the type `[closure@src/main.rs:21:21: 21:50 greeting:&Greeting]` implements the trait `core::ops::Fn<(&mut iron::request::Request<'_, '_>,)>`, but the trait `for<'r, 'r, 'r> core::ops::Fn<(&'r mut iron::request::Request<'r, 'r>,)>` is required (expected concrete lifetime, found bound lifetime parameter ) [E0281]
src/main.rs:21     router.get("/", |r| hello_world(r, &greeting));
                          ^~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
src/main.rs:21:12: 21:51 help: run `rustc --explain E0281` to see a detailed explanation
src/main.rs:22:12: 22:60 error: the trait `for<'r, 'r, 'r> core::ops::Fn<(&'r mut iron::request::Request<'r, 'r>,)>` is not implemented for the type `[closure@src/main.rs:22:25: 22:59 greeting:&mut Greeting]` [E0277]
src/main.rs:22     router.post("/set", |r| set_greeting(r, &mut greeting));
                          ^~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
src/main.rs:22:12: 22:60 help: run `rustc --explain E0277` to see a detailed explanation
error: aborting due to 4 previous errors
Could not compile `httptest`.
```

rustc is not happy. But I expected that. I'm throwing types at it just to get a response. Tell me what to do rustc.

Those error messages are confusing, but clearly the closure is the wrong type. The `get` and `post` methods of `Router` take a [`Handler`](http://ironframework.io/doc/iron/middleware/trait.Handler.html), and from the doc page I see there's an impl defined as

```rust
pub trait Handler: Send + Sync + Any {
    fn handle(&self, &mut Request) -> IronResult<Response>;
}
```

That's a mouthful, but `Handler` is defined for `Fn`, not `FnOnce` or
`FnMut`, and it has to be `Send + Sync`. Since it needs to be send,
we're not going to be capturing any references, and since the
environment isn't mutable we have to use interior mutability to mutate
the greeting. So I'm going to use a sendable smart pointer, `Arc`, and
to make it mutable, put a `Mutex` inside it. For that we need to import
both from the standard library like this: `use std::sync::{Mutex, Arc};`.
I'm also going to have to move the captures with `move |r| ...` to avoid
capturing by reference.

Updating my code like so yields the same error messages.

```rust
    let greeting = Arc::new(Mutex::new(Greeting { msg: "Hello, World".to_string() }));
    let greeting_clone = greeting.clone();

    let mut router = Router::new();

    router.get("/", move |r| hello_world(r, &greeting.lock().unwrap()));
    router.post("/set", move |r| set_greeting(r, &mut greeting_clone.lock().unwrap()));
```

rustc doesn't like the lifetimes of my closure. Why? I don't know. I ask reem
in #rust if he knows what to do.

Several hours later reem says

```text
16:02 < reem> brson: Partially hint the type of the request art, rustc has trouble inferring HRTBs
```

HRTB means 'higher-ranked trait bounds', which means roughly 'complicated lifetimes'.

I change those same lines to hint the type of `r: &mut Request` and everything works...

```rust
    let greeting = Arc::new(Mutex::new(Greeting { msg: "Hello, World".to_string() }));
    let greeting_clone = greeting.clone();

    let mut router = Router::new();

    router.get("/", move |r: &mut Request| hello_world(r, &greeting.lock().unwrap()));
    router.post("/set", move |r: &mut Request| set_greeting(r, &mut greeting_clone.lock().unwrap()));
```

It was seemingly a bug in Rust's inferencer. That's lame.

Now it builds again, so we can test with curl.

```text
$ curl http://localhost:3000
{"msg":"Hello, World"}
$ curl -X POST -d '{"msg":"Just trust the Rust"}' http://localhost:3000/set
$ curl http://localhost:3000
{"msg":"Just trust the Rust"}
```

Now we're playing with power.

# 5. The client

We've got a little JSON server going. Now let's write the client. This
time we're going to use [Hyper] directly.

I add it to my `Cargo.toml`: `hyper = "*"`, then create `src/bin/client.rs`:

```rust
extern crate hyper;

fn main() { }
```

Source files in `src/bin/` are automatically built as executables by cargo. Run `cargo build`
and the `target/debug/client` program appears. Good, universe is sane. Now figure out Hyper.

Cribbing off the [Hyper client example](http://hyperium.github.io/hyper/hyper/client/index.html) I come
up with this snippet that just makes a request for "/" and prints the body:

```rust
extern crate hyper;

use hyper::*;
use std::io::Read;

fn main() {
    let client = Client::new();
    let mut res = client.get("http://localhost:3000/").send().unwrap();
    assert_eq!(res.status, hyper::Ok);
    let mut s = String::new();
    res.read_to_string(&mut s).unwrap();
    println!("{}", s);
}
```

But now `cargo run` no longer works.

```text
$ cargo run
`cargo run` requires that a project only have one executable; use the `--bin` option to specify which one to run
```

I must type `cargo run --bin httptest` to start the server. I do so, then `cargo run --bin client` and see

```text
$ cargo run --bin client
Running `target/debug/client`
{"msg":"Hello, World"}
```

Oh, man, I'm a Rust wizard. One last thing I want to do, make the POST
request to set the message. Obvious thing to do is change `client.get`
to
[`client.post`](http://hyperium.github.io/hyper/hyper/client/struct.Client.html#method.post).
This returns a
[`RequestBuilder`](http://hyperium.github.io/hyper/hyper/client/struct.RequestBuilder.html),
so I'm looking for a builder method that sets the payload. How about [`body`](http://hyperium.github.io/hyper/hyper/client/struct.RequestBuilder.html#method.body)?

My new creation:

```rust
extern crate hyper;

use hyper::*;
use std::io::Read;

fn main() {
    let client = Client::new();
    let res = client.post("http://localhost:3000/set").body("Just trust the Rust").send().unwrap();
    assert_eq!(res.status, hyper::Ok);
    let mut res = client.get("http://localhost:3000/").send().unwrap();
    assert_eq!(res.status, hyper::Ok);
    let mut s = String::new();
    res.read_to_string(&mut s).unwrap();
    println!("{}", s);
}
```

But running it is disappointing.

```text
101 $ cargo run --bin client
   Compiling httptest v0.2.0 (file:///Users/danieleesposti/workspace/httptest)
     Running `target/debug/client`
thread '<main>' panicked at 'assertion failed: `(left == right)` (left: `InternalServerError`, right: `Ok`)', src/bin/client.rs:9
```

And simultaneously I see that the server has also errored:

```text
$ cargo run --bin httptest
   Running `target/debug/httptest`
thread '<unnamed>' panicked at 'called `Result::unwrap()` on an `Err` value: ParseError(SyntaxError("invalid syntax", 1, 1))', src/libcore/result.rs:741
```

It's because of my bad error handling! I didn't pass valid JSON to the `/set` route. Fixing the body to be `.body(r#"{ "msg": "Just trust the Rust" }"#)` lets the client succeed:

```text
$ cargo run --bin client
   Compiling httptest v0.2.0 (file:///opt/dev/httptest)
      Running `target/debug/client`
{"msg":"Just trust the Rust"}
```

And just like that we've created a web service and client in Rust. Looks like the future is near. Go build something with [Iron] and [Hyper].



[Crater]: https://github.com/brson/taskcluster-crater
[Rust]: https://github.com/brson/taskcluster-crater
[Iron]: http://ironframework.io/
[Hyper]: http://hyperium.github.io/hyper/hyper/index.html
[rustc-serialize]: https://crates.io/crates/rustc-serialize
[serde]: https://crates.io/crates/serde
[serde_macros]: https://crates.io/crates/serde_macros
