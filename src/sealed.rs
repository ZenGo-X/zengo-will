use crate::persistent_store::Challenge;

/// Seals server's secret share
///
/// Prevents share from misusing by providing limited interface of obtaining secret's value
pub struct Sealed<P, S> {
    public_key: P,
    server_share: S,
}

impl<P, S> Sealed<P, S> {
    pub fn new(public_key: P, server_secret: S) -> Self {
        Self {
            public_key,
            server_share: server_secret,
        }
    }

    /// Verifies that client share matches server share
    pub fn verify(&self, client_share: &[u8]) -> bool {
        todo!()
    }

    /// Tries to open sealed secret share
    ///
    /// Secret share will only be obtained if it matches a client's share and client provided
    /// correct challenge solution.
    pub fn open(
        self,
        current_ping_counter: u128,
        current_challenge: &[u8],
        client_share: &[u8],
        challenge_solution: &[u8],
    ) -> Result<S, OpenError> {
        todo!()
    }

    /// Exposes underlying secret share. For tests only.
    #[cfg(test)]
    pub fn secret_share(&self) -> &S {
        &self.server_share
    }
}

pub enum OpenError {
    ClientShareDoesntMatchServerShare,
    IncorrectSolution,
}
