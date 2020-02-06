#![allow(unused_variables, dead_code)]

mod execution;
mod storage;
#[cfg(test)]
mod tests;

use execution::ContextFut;
use storage::FutureSet;

use actix::dev::{
    Actor, ActorContext, ActorFuture, ActorState, Addr, AsyncContext, Mailbox, SpawnHandle,
};

pub struct Context<A>
where
    A: Actor<Context = Self>,
{
    mailbox: Mailbox<A>,
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

    fn run(self, act: A) -> Addr<A> {
        let addr = self.address();
        let fut = ContextFut::new(self, act);
        actix_rt::spawn(fut);
        addr
    }
}

impl<A> Default for Context<A>
where
    A: Actor<Context = Self>,
{
    fn default() -> Self {
        Context {
            mailbox: Mailbox::default(),
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
        self.mailbox.address()
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
        self.wait.is_empty()
            || self.state == ActorState::Stopping
            || self.state == ActorState::Stopped
    }

    fn cancel_future(&mut self, handle: SpawnHandle) -> bool {
        todo!("impl this")
    }
}
