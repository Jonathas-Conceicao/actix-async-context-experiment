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

        loop {
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

            break;
        }

        Poll::Ready(())
    }
}
