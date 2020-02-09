#![allow(unused_variables, dead_code)]

mod execution;
mod storage;
#[cfg(test)]
mod tests;

use execution::ContextFut;
use storage::FutureSet;

use actix::dev::{
    channel::AddressSenderProducer, Actor, ActorContext, ActorFuture, ActorState, Addr,
    AsyncContext, Envelope, Handler, Mailbox, Message, Running, SpawnHandle, SyncSender,
    ToEnvelope,
};

pub struct Context<A>
where
    A: Actor<Context = Self>,
{
    addr_producer: AddressSenderProducer<A>,
    mailbox: Option<Mailbox<A>>,
    state: ActorState,
    wait: FutureSet<Box<dyn ActorFuture<Output = (), Actor = A>>>,
    spawn: FutureSet<Box<dyn ActorFuture<Output = (), Actor = A>>>,
}

impl<A> Context<A>
where
    A: Actor<Context = Self>,
{
    fn new() -> Self {
        Self::default()
    }

    fn run(mut self, act: A) -> Addr<A> {
        let addr = self.address();
        let mailbox = self.mailbox.take().unwrap();
        let fut = ContextFut::new(self, act, mailbox);
        actix_rt::spawn(fut);
        addr
    }
}

impl<A, M> ToEnvelope<A, M> for Context<A>
where
    A: Actor<Context = Context<A>> + Handler<M>,
    M: Message + Send + 'static,
    M::Result: Send,
{
    fn pack(msg: M, tx: Option<SyncSender<M::Result>>) -> Envelope<A> {
        Envelope::new(msg, tx)
    }
}

impl<A> Default for Context<A>
where
    A: Actor<Context = Self>,
{
    fn default() -> Self {
        let mb = Mailbox::default();
        Context {
            addr_producer: mb.sender_producer(),
            mailbox: Some(mb),
            state: ActorState::Started,
            wait: FutureSet::default(),
            spawn: FutureSet::default(),
        }
    }
}

impl<A> ActorContext for Context<A>
where
    A: Actor<Context = Self>,
{
    fn stop(&mut self) {
        if let ActorState::Running = self.state {
            self.state = ActorState::Stopping;
        }
    }

    fn terminate(&mut self) {
        self.state = ActorState::Stopped;
    }

    fn state(&self) -> ActorState {
        self.state
    }
}

impl<A> AsyncContext<A> for Context<A>
where
    A: Actor<Context = Self>,
{
    fn address(&self) -> Addr<A> {
        Addr::new(self.addr_producer.sender())
    }

    fn spawn<F>(&mut self, fut: F) -> SpawnHandle
    where
        F: ActorFuture<Output = (), Actor = A> + 'static,
    {
        let _real_index = self.spawn.insert(Box::new(fut));
        // FIXME: construct spawn handle from real index;
        SpawnHandle::default()
    }

    fn wait<F>(&mut self, fut: F)
    where
        F: ActorFuture<Output = (), Actor = A> + 'static,
    {
        self.wait.insert(Box::new(fut));
    }

    fn waiting(&self) -> bool {
        !self.wait.is_empty()
            || self.state == ActorState::Stopping
            || self.state == ActorState::Stopped
    }

    fn cancel_future(&mut self, handle: SpawnHandle) -> bool {
        todo!("impl this")
    }
}
