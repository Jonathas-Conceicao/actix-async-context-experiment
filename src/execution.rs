use super::*;
use std::{
    future::Future,
    pin::Pin,
    task::{self, Poll},
};

pub(super) struct ContextFut<A>
where
    A: Actor<Context = Context<A>>,
{
    act: A,
    ctx: Context<A>,
    mailbox: Mailbox<A>,
    wait: FutureSet<Box<dyn ActorFuture<Output = (), Actor = A>>>,
    spawn: FutureSet<Box<dyn ActorFuture<Output = (), Actor = A>>>,
}

impl<A> ContextFut<A>
where
    A: Actor<Context = Context<A>>,
{
    pub(super) fn new(ctx: Context<A>, act: A, mailbox: Mailbox<A>) -> Self {
        ContextFut { act, ctx, mailbox, wait: FutureSet::default(), spawn: FutureSet::default() }
    }

    fn merge(&mut self) {
        self.wait.append(&mut self.ctx.wait);
        self.spawn.append(&mut self.ctx.spawn);
    }

    fn alive(&mut self) -> bool {
        if self.ctx.state == ActorState::Stopped {
            return false;
        }
        self.mailbox.connected() || !self.spawn.is_empty() || !self.wait.is_empty()
    }
}

impl<A> Future for ContextFut<A>
where
    A: Actor<Context = Context<A>>,
{
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();

        if let ActorState::Started = this.ctx.state {
            Actor::started(&mut this.act, &mut this.ctx);
            this.ctx.state = ActorState::Running;

            this.merge();
        }

        'outter: loop {
            while !this.wait.is_empty() && this.ctx.state == ActorState::Running {
                if let Some(item) = this.wait.next_mut() {
                    match Pin::new(item).poll(&mut this.act, &mut this.ctx, cx) {
                        Poll::Ready(_) => (),
                        Poll::Pending => return Poll::Pending,
                    }
                    this.wait.pop();
                }
            }

            this.mailbox.poll(&mut this.act, &mut this.ctx, cx);
            this.merge();
            if !this.wait.is_empty() && this.ctx.state == ActorState::Running {
                continue;
            }

            if this.ctx.state == ActorState::Running {
                let mut idx = 0;
                while idx < this.spawn.len() && this.ctx.state == ActorState::Running {
                    match Pin::new(&mut this.spawn[idx]).poll(&mut this.act, &mut this.ctx, cx) {
                        Poll::Pending => {}
                        Poll::Ready(()) => {
                            this.spawn.remove(idx);
                        }
                    }
                    this.merge();
                    if !this.wait.is_empty() && this.ctx.state == ActorState::Running {
                        // Polled future has request a new wait;
                        continue 'outter;
                    }
                    idx += 1;
                }
            }

            let should_stop = match this.ctx.state {
                ActorState::Stopping => {
                    Actor::stopping(&mut this.act, &mut this.ctx) == Running::Stop
                }
                ActorState::Running => {
                    !this.alive() && Actor::stopping(&mut this.act, &mut this.ctx) == Running::Stop
                }
                _ => false,
            };

            if should_stop {
                this.ctx.state = ActorState::Stopped;
                Actor::stopped(&mut this.act, &mut this.ctx);
                return Poll::Ready(());
            }

            cx.waker().wake_by_ref();
            return Poll::Pending;
        }
    }
}
