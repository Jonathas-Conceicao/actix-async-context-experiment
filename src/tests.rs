use super::*;
use actix::fut::WrapFuture;
use std::time::{Duration, Instant};
use tokio::time::delay_for;

struct BasicActor;

impl Actor for BasicActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        ctx.wait(delay_for(Duration::from_secs(2)).into_actor(self));
    }
}

#[actix_rt::test]
async fn empty_start() {
    let act = BasicActor;
    let instant = Instant::now();

    Context::new().run(act);
    actix_rt::Arbiter::local_join().await;

    assert!(instant.elapsed() >= Duration::from_secs(2));
}
