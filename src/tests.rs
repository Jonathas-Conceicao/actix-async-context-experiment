use super::*;
use actix::fut::WrapFuture;
use std::time::{Duration, Instant};
use tokio::time::delay_for;

mod context_wait {
    use super::*;

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
}

mod message_handling {
    use super::*;

    struct BasicActor;

    impl Actor for BasicActor {
        type Context = Context<Self>;
    }

    #[derive(Message)]
    #[rtype(result = "()")]
    struct Sleep(Duration);

    impl Handler<Sleep> for BasicActor {
        type Result = ();
        fn handle(&mut self, msg: Sleep, ctx: &mut Context<Self>) {
            ctx.wait(delay_for(msg.0).into_actor(self))
        }
    }

    #[actix_rt::test]
    async fn send_message() {
        let act = BasicActor;
        let instant = Instant::now();

        let addr = Context::new().run(act);
        addr.send(Sleep(Duration::from_secs(2))).await.unwrap();
        actix_rt::Arbiter::local_join().await;

        assert!(instant.elapsed() >= Duration::from_secs(2));
    }
}
