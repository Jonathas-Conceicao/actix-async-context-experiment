use super::*;
use actix::fut::WrapFuture;
use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::time::delay_for;

mod methods_override {
    use super::*;

    type Data = Arc<Mutex<usize>>;

    struct IncOnStart(Data);

    impl Actor for IncOnStart {
        type Context = Context<Self>;

        fn started(&mut self, ctx: &mut Context<Self>) {
            *self.0.lock().unwrap() += 313;
        }
    }

    #[actix_rt::test]
    async fn started() {
        let data = Arc::new(Mutex::new(0));
        let act = IncOnStart(data.clone());

        Context::new().run(act);
        actix_rt::Arbiter::local_join().await;

        assert_eq!(*data.lock().unwrap(), 313);
    }

    struct IncOnStop(Data);

    impl Actor for IncOnStop {
        type Context = Context<Self>;

        fn stopped(&mut self, ctx: &mut Context<Self>) {
            *self.0.lock().unwrap() += 313;
        }
    }

    #[actix_rt::test]
    async fn stopped() {
        let data = Arc::new(Mutex::new(0));
        let act = IncOnStop(data.clone());

        Context::new().run(act);
        actix_rt::Arbiter::local_join().await;

        assert_eq!(*data.lock().unwrap(), 313);
    }

    struct Stubborn(Data);

    impl Actor for Stubborn {
        type Context = Context<Self>;

        fn stopping(&mut self, ctx: &mut Context<Self>) -> Running {
            let mut data = self.0.lock().unwrap();
            if *data < 313 {
                *data += 100;
                return Running::Continue;
            }

            Running::Stop
        }
    }

    #[actix_rt::test]
    async fn stopping() {
        let data = Arc::new(Mutex::new(0));
        let act = Stubborn(data.clone());

        Context::new().run(act);
        actix_rt::Arbiter::local_join().await;

        assert_eq!(*data.lock().unwrap(), 400);
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
