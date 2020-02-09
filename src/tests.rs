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

        let _ = Context::new().run(act);
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
    struct WaitSleep(Duration);

    impl Handler<WaitSleep> for BasicActor {
        type Result = ();
        fn handle(&mut self, msg: WaitSleep, ctx: &mut Context<Self>) {
            ctx.wait(delay_for(msg.0).into_actor(self));
        }
    }

    #[derive(Message)]
    #[rtype(result = "()")]
    struct SpawnSleep(Duration);

    impl Handler<SpawnSleep> for BasicActor {
        type Result = ();
        fn handle(&mut self, msg: SpawnSleep, ctx: &mut Context<Self>) {
            ctx.spawn(delay_for(msg.0).into_actor(self));
        }
    }

    #[actix_rt::test]
    async fn handle_with_ctx_wait() {
        let act = BasicActor;
        let instant = Instant::now();

        {
            // Scope used to eagerly drop address so actor can automatically stop
            let addr = Context::new().run(act);
            addr.send(WaitSleep(Duration::from_secs(2))).await.unwrap();
        }
        actix_rt::Arbiter::local_join().await;

        assert!(instant.elapsed() >= Duration::from_secs(2));
    }

    #[actix_rt::test]
    async fn handle_with_ctx_spawn() {
        let act = BasicActor;
        let instant = Instant::now();

        {
            // Scope used to eagerly drop address so actor can automatically stop
            let addr = Context::new().run(act);
            addr.send(SpawnSleep(Duration::from_secs(2))).await.unwrap();
        }
        actix_rt::Arbiter::local_join().await;

        assert!(instant.elapsed() >= Duration::from_secs(2));
    }
}
