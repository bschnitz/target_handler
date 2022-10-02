## Target Handler

Target Handler provides a derive macro to easily create a handler `struct` for
an `enum` with multiple variants. The idea is to transform or avoid a match
construct into a trait with a handler function for each variant the enum has.

The target handler was implemented to simplify or beautify the handling of a
[clap](https://docs.rs/clap/latest/clap/) Subcommands enum as seen in [this
example](https://github.com/clap-rs/clap/blob/master/examples/git-derive.rs).
But it may be useful for other purposes or as a hint on how to implement some
design patterns, like `Command` or `Message Broker` in rust (though this is
neither of them and much simpler).

## Example

``` rust
use target_handler::Target;

// The enum for which a handler shall be implemented. The macro will create a
// trait 'MessageHandler', which must be implemented by the handler struct.
// After implementing the trait, an arbitrary message can be handled using the
// 'deliver' method. If the 'returns' attribute is set, all handlers must
// implement the specified return type, as will the 'deliver' method.
#[derive(Target, Debug)]
#[handler(returns = "Result<(), String>", trait_name = "MessageHandler", method = "deliver")]
enum Message {
    Mail {
        // the fields of each variant will be provided to the corresponding
        // handler method in the trait. The names of the handler methods equal
        // the names of the variants, except for that they are lowercase.
        from: String,
        to: String
    },
    Leaflet {
        composer: Option<String>,
        content: String,
    },
    Ping,
    Exception
}

#[derive(Debug)]
struct Legman;

// The trait enforces an implementation of a handler method for every enum
// variant and delivers in turn a default implementation for the summarized
// handler method.
impl MessageHandler for Legman {
    // The parameters for each method mirror the properties of the struct, if
    // the coresponding variant is a struct. Currently this crate only allows
    // the enum to have either struct or unit variants.
    fn mail(&self, from: String, to: String) -> Result<(), String> {
        println!("Hello {to}. I got a message from {from} for You!");
        Ok(())
    }

    fn leaflet(&self, composer: Option<String>, content: String) -> Result<(), String> {
        match composer {
            Some(composer) => println!("This pamphlet was composed by {composer}."),
            None           => println!("I have an anonymous pamphlet here.")
        }
        println!("Hear my words!");
        println!("{content}");
        Ok(())
    }

    // a unit variant will have no parameters but a reference to self
    fn ping(&self) -> Result<(), String> {
        println!("Pong.");
        Ok(())
    }

    fn exception(&self) -> Result<(), String> {
        Err("I don't know what to do with this message!".to_string())
    }
}

fn main() {
    let legman = Legman;
    let make_delivery = || -> Result<(), String> {
        // the handler methods (specified in the derive macros attributes as
        // 'deliver') can get an arbitrary variant of the enum and delivers it
        // to the corresponding handler method defined when implementing the
        // trait.
        legman.deliver(Message::Mail {
            from: "Bob".to_string(),
            to: "Alice".to_string()
        })?;
        println!("");
        legman.deliver(Message::Leaflet {
            composer: None,
            content: "Tomatoes are delicious!".to_string()
        })?;
        println!("");
        legman.deliver(Message::Ping)?;
        println!("");
        legman.deliver(Message::Exception)
    };

    // the specified return type allows for simplified error handling
    if let Err(err) = make_delivery() {
        println!("An error occeured during delivery:");
        println!("{err}");
    };
}
```
