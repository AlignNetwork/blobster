use eyre::Result;
use futures::{Future, FutureExt};
use reth_exex::{ExExContext, ExExEvent, ExExNotification};
use reth_node_api::FullNodeComponents;
use reth_tracing::tracing::{error, info};
use std::{
    pin::Pin,
    task::{ready, Context, Poll},
};

use crate::network::Libp2pExEx;

pub struct ExEx<Node: FullNodeComponents> {
    exex: ExExContext<Node>,
    libp2p: Libp2pExEx,
}

impl<Node: FullNodeComponents> ExEx<Node> {
    pub fn new(exex: ExExContext<Node>, libp2p: Libp2pExEx) -> Self {
        Self { exex, libp2p }
    }

    async fn handle_chain_update(&mut self, message: String) {
        info!("Chain update: {}", message);
    }
}

impl<Node: FullNodeComponents> Future for ExEx<Node> {
    type Output = Result<()>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Poll the Libp2p future
        if let Poll::Ready(result) = Box::pin(self.libp2p.run()).poll_unpin(cx) {
            return Poll::Ready(result);
        }

        // Poll ExExContext notifications
        loop {
            match self.exex.notifications.poll_recv(cx) {
                Poll::Ready(Some(notification)) => match notification {
                    ExExNotification::ChainCommitted { new } => {
                        info!(committed_chain = ?new.range(), "Received commit");
                        let message = format!("Chain committed: {:?}", new.range());
                        let mut pinned_future = Box::pin(self.handle_chain_update(message));
                        let _ = pinned_future.as_mut().poll(cx);
                    }
                    ExExNotification::ChainReorged { old, new } => {
                        info!(from_chain = ?old.range(), to_chain = ?new.range(), "Received reorg");
                        let message =
                            format!("Chain reorged: {:?} -> {:?}", old.range(), new.range());
                        let mut pinned_future = Box::pin(self.handle_chain_update(message));
                        let _ = pinned_future.as_mut().poll(cx);
                    }
                    ExExNotification::ChainReverted { old } => {
                        info!(reverted_chain = ?old.range(), "Received revert");
                        let message = format!("Chain reverted: {:?}", old.range());
                        let mut pinned_future = Box::pin(self.handle_chain_update(message));
                        let _ = pinned_future.as_mut().poll(cx);
                    }
                },
                Poll::Ready(None) => return Poll::Ready(Ok(())),
                Poll::Pending => break,
            }
        }

        Poll::Pending
    }
}
