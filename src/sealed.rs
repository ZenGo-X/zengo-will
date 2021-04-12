use curv::elliptic::curves::traits::ECPoint;

use crate::persistent_store::Challenge;

/// Seals server's secret share
///
/// Prevents share from misusing by providing limited interface of obtaining secret's value
pub struct Sealed<P: ECPoint> {
    public_key: P,
    server_share: P::Scalar,
}

impl<P> Sealed<P>
where
    P: ECPoint,
    P::Scalar: Clone,
{
    pub fn new(public_key: P, server_secret: P::Scalar) -> Self {
        Self {
            public_key,
            server_share: server_secret,
        }
    }

    /// Verifies that client share matches server share
    pub fn verify(&self, client_share_pk: P) -> bool {
        client_share_pk * self.server_share.clone() == self.public_key
    }

    /// Tries to open sealed secret share
    ///
    /// Secret share will only be obtained if it matches a client's share and client provided
    /// correct challenge solution.
    pub fn open(
        self,
        current_challenge: &Challenge,
        solved_challenge: &Challenge,
        challenge_solution: vdf::SolvedVDF,
        client_share_pk: P,
    ) -> Result<P::Scalar, OpenError> {
        if current_challenge.id > solved_challenge.id {
            Err(OpenError::OldChallenge)
        } else if current_challenge != solved_challenge {
            Err(OpenError::InvalidChallenge)
        } else if let Err(reason) = challenge_solution.verify(&current_challenge.challenge) {
            Err(OpenError::IncorrectSolution(reason))
        } else if !self.verify(client_share_pk) {
            Err(OpenError::ClientShareDoesntMatchServerShare)
        } else {
            Ok(self.server_share)
        }
    }

    /// Exposes underlying secret share. For tests only.
    #[cfg(test)]
    pub fn secret_share(&self) -> &P::Scalar {
        &self.server_share
    }
}

pub enum OpenError {
    ClientShareDoesntMatchServerShare,
    IncorrectSolution(vdf::utilities::ErrorReason),
    OldChallenge,
    InvalidChallenge,
}
