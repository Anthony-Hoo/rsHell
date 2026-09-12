//! Never retain command payloads: secrets drop inside try_send, before recording.
use rshell_core::{
    HostKeyDecision, InteractionId, InteractionResponse, SessionId, UiCommand, UiCommandPort,
    UiPortError,
};
use std::sync::{
    Mutex,
    atomic::{AtomicBool, Ordering},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Response {
    Reject,
    Accept,
    Secret,
    Answers,
    Cancel,
}

#[derive(Default)]
pub(super) struct Port {
    responses: Mutex<Vec<(SessionId, InteractionId, Response)>>,
    busy: AtomicBool,
}

impl Port {
    pub(super) fn reject_next(&self) {
        self.busy.store(true, Ordering::SeqCst);
    }
    pub(super) fn verify(
        &self,
        session: SessionId,
        interaction: InteractionId,
        expected: &[Response],
    ) {
        let actual = self.responses.lock().unwrap();
        assert_eq!(
            actual.len(),
            expected.len(),
            "exact Respond count, no repeated submission"
        );
        for (record, response) in actual.iter().zip(expected) {
            assert_eq!(*record, (session, interaction, *response));
        }
        println!(
            "SYNTHETIC_RESPOND session={session:?} interaction={interaction:?} categories={expected:?} count={}",
            actual.len()
        );
    }
}

impl UiCommandPort for Port {
    fn try_send(&self, command: UiCommand) -> Result<(), UiPortError> {
        if let UiCommand::Respond {
            session,
            interaction,
            response,
        } = command
        {
            let category = match response {
                InteractionResponse::HostKey(HostKeyDecision::Reject) => Response::Reject,
                InteractionResponse::HostKey(HostKeyDecision::AcceptAndStore) => Response::Accept,
                InteractionResponse::Secret(_) => Response::Secret,
                InteractionResponse::Answers(_) => Response::Answers,
                InteractionResponse::Cancel => Response::Cancel,
            };
            self.responses
                .lock()
                .unwrap()
                .push((session, interaction, category));
            if self.busy.swap(false, Ordering::SeqCst) {
                return Err(UiPortError::Busy);
            }
        }
        Ok(())
    }
}
