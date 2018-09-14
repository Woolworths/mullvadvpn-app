use std::thread;

use error_chain::ChainedError;
use futures::sync::{mpsc, oneshot};
use futures::{Async, Future, Stream};

use talpid_types::tunnel::{ActionAfterDisconnect, BlockReason};

use super::{
    BlockedState, ConnectingState, DisconnectedState, EventConsequence, ResultExt,
    SharedTunnelStateValues, TunnelCommand, TunnelParameters, TunnelState, TunnelStateTransition,
    TunnelStateWrapper,
};
use tunnel::CloseHandle;

/// This state is active from when we manually trigger a tunnel kill until the tunnel wait
/// operation (TunnelExit) returned.
pub struct DisconnectingState {
    exited: oneshot::Receiver<()>,
    after_disconnect: AfterDisconnect,
}

impl DisconnectingState {
    fn handle_commands(
        mut self,
        commands: &mut mpsc::UnboundedReceiver<TunnelCommand>,
    ) -> EventConsequence<Self> {
        use self::AfterDisconnect::*;

        let event = try_handle_event!(self, commands.poll());
        let after_disconnect = self.after_disconnect;

        self.after_disconnect = match after_disconnect {
            AfterDisconnect::Nothing => match event {
                Ok(TunnelCommand::Connect(parameters)) => Reconnect(parameters),
                Ok(TunnelCommand::Block(reason, allow_lan)) => Block(reason, allow_lan),
                _ => Nothing,
            },
            AfterDisconnect::Block(reason, allow_lan) => match event {
                Ok(TunnelCommand::Connect(parameters)) => Reconnect(parameters),
                Ok(TunnelCommand::Disconnect) => Nothing,
                Ok(TunnelCommand::Block(new_reason, new_allow_lan)) => {
                    Block(new_reason, new_allow_lan)
                }
                _ => Block(reason, allow_lan),
            },
            AfterDisconnect::Reconnect(mut tunnel_parameters) => match event {
                Ok(TunnelCommand::AllowLan(allow_lan)) => {
                    tunnel_parameters.allow_lan = allow_lan;
                    Reconnect(tunnel_parameters)
                }
                Ok(TunnelCommand::Connect(parameters)) => Reconnect(parameters),
                Ok(TunnelCommand::Disconnect) | Err(_) => Nothing,
                Ok(TunnelCommand::Block(reason, allow_lan)) => Block(reason, allow_lan),
            },
        };

        EventConsequence::SameState(self)
    }

    fn handle_exit_event(
        mut self,
        shared_values: &mut SharedTunnelStateValues,
    ) -> EventConsequence<Self> {
        use self::EventConsequence::*;

        match self.exited.poll() {
            Ok(Async::NotReady) => NoEvents(self),
            Ok(Async::Ready(_)) | Err(_) => NewState(self.after_disconnect(shared_values)),
        }
    }

    fn after_disconnect(
        self,
        shared_values: &mut SharedTunnelStateValues,
    ) -> (TunnelStateWrapper, TunnelStateTransition) {
        match self.after_disconnect {
            AfterDisconnect::Nothing => DisconnectedState::enter(shared_values, ()),
            AfterDisconnect::Block(reason, allow_lan) => {
                BlockedState::enter(shared_values, (reason, allow_lan))
            }
            AfterDisconnect::Reconnect(tunnel_parameters) => {
                ConnectingState::enter(shared_values, tunnel_parameters)
            }
        }
    }
}

impl TunnelState for DisconnectingState {
    type Bootstrap = (CloseHandle, oneshot::Receiver<()>, AfterDisconnect);

    fn enter(
        _: &mut SharedTunnelStateValues,
        (close_handle, exited, after_disconnect): Self::Bootstrap,
    ) -> (TunnelStateWrapper, TunnelStateTransition) {
        thread::spawn(move || {
            let close_result = close_handle
                .close()
                .chain_err(|| "Failed to close the tunnel");

            if let Err(error) = close_result {
                error!("{}", error.display_chain());
            }
        });

        let action_after_disconnect = after_disconnect.action();

        (
            TunnelStateWrapper::from(DisconnectingState {
                exited,
                after_disconnect,
            }),
            TunnelStateTransition::Disconnecting(action_after_disconnect),
        )
    }

    fn handle_event(
        self,
        commands: &mut mpsc::UnboundedReceiver<TunnelCommand>,
        shared_values: &mut SharedTunnelStateValues,
    ) -> EventConsequence<Self> {
        self.handle_commands(commands)
            .or_else(Self::handle_exit_event, shared_values)
    }
}

/// Which state should be transitioned to after disconnection is complete.
pub enum AfterDisconnect {
    Nothing,
    Block(BlockReason, bool),
    Reconnect(TunnelParameters),
}

impl AfterDisconnect {
    /// Build event representation of the action that will be taken after the disconnection.
    pub fn action(&self) -> ActionAfterDisconnect {
        match self {
            AfterDisconnect::Nothing => ActionAfterDisconnect::Nothing,
            AfterDisconnect::Block(..) => ActionAfterDisconnect::Block,
            AfterDisconnect::Reconnect(_) => ActionAfterDisconnect::Reconnect,
        }
    }
}
