[WIP] rust-reactive-log
=================

This readme is mostly the design spec at this point.  Most of this isn't implemented yet.

Local transactional log inspired by Acid-state and Kafka.  Delivery is strictly FIFO.

Consumption styles:
* per-consumer exactly once
* global exactly once
* nontransactional high-throughput (at least once)
* all above styles expose both blocking and nonblocking interfaces

Consumers can either use a simple iterator or a rich processing function that supports these macros:
* ```commit!()``` completes processing of an element.  This is
* ```retry!()``` restarts the processing attempt without incrementing the count of processed elements.
* ```break!()``` causes the producer function to return.

Caveats
* only one process may produce at any time.  but feel free to create a proxy that funnels writes from multiple producers!
* only one process may globally transactionally consume at a time
* only one process may transactionally consume for a particular consumer ID at a time
* none of these caveats are enforced by this library!

```rust
extern crate reactive-log;
use reactive-log::{Producer, Consumer, ProducerOptions, ConsumerOptions, SyncPolicy, Whence};

fn producer() {
  let prod_opts = ProducerOptions {
      log_dir:                      "/var/log/myapp/",
      sync_policy:                  SyncPolicy::Periodic(Duration::seconds(1)),
      file_roll_size:               67_108_864,
      blocking_minimum_retention:   None,
      max_total_bytes:              536_870_912,
      max_file_age:                 None,
  };
  let prod = Producer::new(prod_opts).unwrap();
  ...
  for data in input_stream.iter() {
    prod.append(data.bytes());
  }
}

fn simple_nontransactional_consumer() {
  let whence = Whence::Latest;
  let consumer_opts = ConsumerOptions {
      log_dir:    "/var/log/myapp/",
      style:      ConsumerStyle::NonTxConsumer(whence),
  };

  let consumer = Consumer::new(consumer_opts).unwrap();
  for msg in consumer.iter() {
    process(msg);
  }
}

fn per_client_transactional_consumer() {
  let consumer_id = "spacejam's post consumer";
  let consumer_opts = ConsumerOptions {
      log_dir:    "/var/log/myapp/",
      style:      ConsumerStyle::ClientTxConsumer(consumer_id),
  };

  let consumer = Consumer::new(consumer_opts).unwrap();

  // try to consume 5, but if we hit the end of the log don't wait to return (nonblocking + limit)
  let max_messages_to_consume = Some(5);
  consumer.nonblocking_process(max_messages_to_consume, |data| {
      match process(data) {
          Ok(processed) => match external_persist(processed) {
              Ok(_) => commit!(),
              Err(e) => {
                  report(e);
                  retry!();
              },
          },
          Err(e) => {
              report(e);
              skip!();
          },
      }
  })
}

fn global_transactional_consumer() {
  let consumer_opts = ConsumerOptions {
      log_dir:    "/var/log/myapp/",
      style:      ConsumerStyle::GlobalTxConsumer(),
  };

  let consumer = Consumer::new(consumer_opts).unwrap();

  // consume forever (blocking + no limit)
  let max_messages_to_consume = None;
  consumer.blocking_process(max_messages_to_consume, |data| {
      match process(data) {
          Ok(processed) => match external_persist(processed) {
              Ok(_) => commit!(),
              Err(e) => {
                  report(e);
                  retry!();
              },
          },
          Err(e) => {
              report(e);
              skip!();
          },
      }
  })
}

fn retrying_nontransactional_consumer() {
  // subscribe to new messages, starting at the current last element
  // position choices: Oldest, Latest, Position(offset)
  let whence = Whence::Latest;
  let consumer_opts = ConsumerOptions {
      log_dir:    "/var/log/myapp/",
      style:      ConsumerStyle::NonTxConsumer(whence),
  };

  let consumer = Consumer::new(consumer_opts).unwrap();

  // consume forever (blocking + no limit)
  let max_messages_to_consume = None;
  consumer.blocking_process(max_messages_to_consume, |data| {
      match process(data) {
          Ok(processed) => match external_persist(processed) {
              Ok(_) => commit!(),
              Err(e) => {
                  report(e);
                  retry!();
              },
          },
          Err(e) => {
              report(e);
              skip!();
          },
      }
  })
}
```
