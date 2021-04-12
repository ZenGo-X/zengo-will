mod sled;

use std::path::PathBuf;
use std::{fmt, io};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use curv::elliptic::curves::traits::ECPoint;

use crate::sealed::Sealed;

#[async_trait]
pub trait PersistentStore<P: ECPoint>: Clone + Sync + Send {
    type Error;

    /// Opens existing persistent_store or creates a new one on file system.
    async fn open(path: PathBuf) -> Result<Self, Self::Error>;

    /// Adds a server's secret share to the persistent_store.
    ///
    /// Returns `Error::AlreadyExist` if there is a share associated with given `public_key`.
    async fn add_server_secret_share(
        &self,
        public_key: P,
        server_secret_share: P::Scalar,
    ) -> Result<(), Self::Error>;

    /// Returns a server's secret share associated with given `public_key`
    async fn get_server_secret_share(
        &self,
        public_key: P,
    ) -> Result<Option<Sealed<P>>, Self::Error>;

    /// Increases ping counter by 1
    ///
    /// This will reset challenge, i.e. `db.get_challenge().await` will return `Ok(None)` until
    /// new challenge is set.
    ///
    /// Returns increased ping counter.
    async fn increase_ping_counter(&self) -> Result<u128, Self::Error>;

    /// Returns ping counter
    async fn get_ping_counter(&self) -> Result<u128, Self::Error>;

    /// Sets a new challenge that will be valid until receiving new ping.
    ///
    /// ## Errors
    /// * [SetChallengeError::AlreadySet] is returned if challenge with the same id is already set
    /// * [SetChallengeError::Outdated] is returned if `challenge.id < db.get_ping_counter()`
    /// * [SetChallengeError::Io] indicates that some underlying error happened
    async fn set_challenge(
        &self,
        challenge: Challenge,
    ) -> Result<(), SetChallengeError<Self::Error>>;

    /// Returns the latest set challenge
    ///
    /// Challenge is guaranteed to be up-to-date, i.e. `challenge.id == db.get_ping_counter()`
    async fn get_challenge(&self) -> Result<Option<Challenge>, Self::Error>;
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct Challenge {
    pub id: u128,
    pub challenge: vdf::UnsolvedVDF,
}

#[derive(Debug)]
pub enum SetChallengeError<E> {
    AlreadySet(Challenge),
    Outdated,
    MismatchedId,
    Store(E),
}

impl<E> fmt::Display for SetChallengeError<E>
where
    E: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SetChallengeError::AlreadySet(..) => write!(f, "challenge is already set"),
            SetChallengeError::Outdated => write!(f, "testator is not offline"),
            SetChallengeError::MismatchedId => write!(f, "id doesn't match current ping counter"),
            SetChallengeError::Store(e) => write!(f, "{}", e),
        }
    }
}

impl<E> std::error::Error for SetChallengeError<E>
where
    E: fmt::Debug + fmt::Display + std::error::Error + 'static,
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            SetChallengeError::Store(e) => Some(e),
            SetChallengeError::Outdated
            | SetChallengeError::AlreadySet(..)
            | SetChallengeError::MismatchedId => None,
        }
    }
}
