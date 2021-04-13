use std::fmt;
use std::marker::PhantomData;
use std::mem::size_of;

use async_trait::async_trait;

use curv::arithmetic::{Converter, Zero};
use curv::elliptic::curves::traits::{ECPoint, ECScalar};
use curv::BigInt;
use tonic::{Request, Response, Status};

use crate::persistent_store::{PersistentStore, SetChallengeError};
use crate::proto::beneficiary::{
    beneficiary_api_server::BeneficiaryApi, Challenge, GetChallengeRequest,
    ObtainServerSecretShareRequest, ObtainServerSecretShareResponse, VerifyServerShareRequest,
    VerifyServerShareResponse,
};
use crate::proto::testator::{
    testator_api_server::TestatorApi, PingRequest, PongResponse, SaveServerShareRequest,
    SaveServerShareResponse,
};
use crate::sealed::OpenError;

pub struct BeneficiaryServer<S, P> {
    vdf_setup: vdf::SetupForVDF,
    store: S,
    _ph: PhantomData<fn() -> P>,
}

impl<S, P> BeneficiaryServer<S, P>
where
    S: PersistentStore<P>,
    P: ECPoint,
{
    pub fn new(vdf_setup: vdf::SetupForVDF, persistent_store: S) -> Self {
        Self {
            vdf_setup,
            store: persistent_store,
            _ph: PhantomData,
        }
    }
}

#[async_trait]
impl<S, P> BeneficiaryApi for BeneficiaryServer<S, P>
where
    P: ECPoint + Send + 'static,
    P::Scalar: Clone,
    S: PersistentStore<P> + 'static,
    S::Error: fmt::Display,
{
    async fn verify_server_share(
        &self,
        request: Request<VerifyServerShareRequest>,
    ) -> Result<Response<VerifyServerShareResponse>, Status> {
        let request = request.into_inner();
        let public_key = match P::from_bytes(&request.public_key) {
            Ok(pk) => pk,
            Err(_) => return Err(Status::invalid_argument("invalid joint public key")),
        };
        let client_public_share = match P::from_bytes(&request.client_public_share) {
            Ok(ps) => ps,
            Err(_) => return Err(Status::invalid_argument("invalid client public share")),
        };

        let server_share = match self.store.get_server_secret_share(public_key).await {
            Ok(Some(ss)) => ss,
            Ok(None) => return Err(Status::not_found("not found")),
            Err(e) => {
                return Err(Status::internal(format!(
                    "getting server share from persistent store resulted in error: {}",
                    e
                )))
            }
        };
        let proof = match server_share.verify_and_proof(client_public_share) {
            Some(p) => p,
            None => return Err(Status::not_found("not found")),
        };
        let proof_bytes = proof.pk_to_key_slice();
        Ok(Response::new(VerifyServerShareResponse {
            server_public_share: proof_bytes,
        }))
    }

    async fn get_challenge(
        &self,
        _request: Request<GetChallengeRequest>,
    ) -> Result<Response<Challenge>, Status> {
        match self.store.get_challenge().await {
            Ok(Some(challenge)) => {
                let id = challenge.id.to_le_bytes().to_vec();
                let challenge = serde_json::to_vec(&challenge)
                    .map_err(|e| Status::internal(format!("serialize challenge: {}", e)))?;
                return Ok(Response::new(Challenge { id, challenge }));
            }
            Err(e) => {
                return Err(Status::internal(format!(
                    "retrieving challenge resulted in error: {}",
                    e
                )))
            }
            Ok(None) => (),
        }
        let id = self.store.get_ping_counter().await.map_err(|e| {
            Status::internal(format!("retrieving ping counter resulted in error: {}", e))
        })?;
        let challenge = crate::persistent_store::Challenge {
            id,
            challenge: vdf::SetupForVDF::pick_challenge(&self.vdf_setup),
        };
        let challenge = match self.store.set_challenge(challenge.clone()).await {
            Ok(()) => challenge,
            Err(SetChallengeError::AlreadySet(challenge)) => challenge,
            Err(SetChallengeError::Outdated) => {
                return Err(Status::failed_precondition("ZenGo server is online"))
            }
            Err(SetChallengeError::MismatchedId) => {
                return Err(Status::internal("challenge.id > ping_counter"))
            }
            Err(SetChallengeError::Store(e)) => {
                return Err(Status::internal(format!(
                    "setting challenge resulted in error: {}",
                    e
                )))
            }
        };

        let id = challenge.id.to_le_bytes().to_vec();
        let challenge = serde_json::to_vec(&challenge)
            .map_err(|e| Status::internal(format!("serialize challenge: {}", e)))?;
        return Ok(Response::new(Challenge { id, challenge }));
    }

    async fn obtain_server_secret_share(
        &self,
        request: Request<ObtainServerSecretShareRequest>,
    ) -> Result<Response<ObtainServerSecretShareResponse>, Status> {
        let request = request.into_inner();

        let public_key = P::from_bytes(&request.public_key)
            .map_err(|_e| Status::invalid_argument("invalid public key"))?;
        let client_public_share = P::from_bytes(&request.client_public_share)
            .map_err(|_e| Status::invalid_argument("invalid public key"))?;

        let solved_challenge = request
            .solved_challenge
            .ok_or_else(|| Status::invalid_argument("solved challenge is not provided"))?;
        let solved_challenge_id = [0u8; size_of::<u128>()];
        if solved_challenge.id.len() != solved_challenge_id.len() {
            return Err(Status::invalid_argument("invalid solved challenge id"));
        }
        let solved_challenge_id = u128::from_le_bytes(solved_challenge_id);
        let solved_challenge = serde_json::from_slice(&solved_challenge.challenge)
            .map_err(|_e| Status::invalid_argument("invalid solved challenge"))?;
        let solved_challenge = crate::persistent_store::Challenge {
            id: solved_challenge_id,
            challenge: solved_challenge,
        };

        let challenge_solution = serde_json::from_slice(&request.solution)
            .map_err(|_e| Status::invalid_argument("invalid solution"))?;

        let current_challenge = self
            .store
            .get_challenge()
            .await
            .map_err(|e| {
                Status::internal(format!(
                    "retrieving current challenge resulted in error: {}",
                    e
                ))
            })?
            .ok_or_else(|| Status::failed_precondition("ZenGo server is online"))?;
        let secret = self
            .store
            .get_server_secret_share(public_key)
            .await
            .map_err(|e| {
                Status::internal(format!(
                    "retrieving server secret share resulted in error: {}",
                    e
                ))
            })?
            .ok_or_else(|| Status::not_found("not found"))?;

        match secret.open(
            &current_challenge,
            &solved_challenge,
            challenge_solution,
            client_public_share,
        ) {
            Ok(server_share) => Ok(Response::new(ObtainServerSecretShareResponse {
                server_secret_share: server_share.to_big_int().to_bytes(),
            })),
            Err(OpenError::ClientShareDoesntMatchServerShare) => {
                Err(Status::not_found("not found"))
            }
            Err(OpenError::OldChallenge) => {
                Err(Status::failed_precondition("ZenGo server is online"))
            }
            Err(OpenError::InvalidChallenge) => Err(Status::invalid_argument(
                "solved challenge is different from what was required to solve",
            )),
            Err(OpenError::IncorrectSolution(_e)) => {
                Err(Status::invalid_argument("incorrect solution"))
            }
        }
    }
}

pub struct TestatorServer<S, P> {
    store: S,
    _ph: PhantomData<fn() -> P>,
}

impl<S, P> TestatorServer<S, P> {
    pub fn new(persistent_store: S) -> Self {
        Self {
            store: persistent_store,
            _ph: PhantomData,
        }
    }
}

#[async_trait]
impl<S, P> TestatorApi for TestatorServer<S, P>
where
    P: ECPoint + Send + 'static,
    P::Scalar: Send,
    S: PersistentStore<P> + 'static,
    S::Error: fmt::Display,
{
    async fn ping(&self, _request: Request<PingRequest>) -> Result<Response<PongResponse>, Status> {
        if let Err(e) = self.store.increase_ping_counter().await {
            Err(Status::internal(format!(
                "increasing of ping counter resulted in error: {}",
                e
            )))
        } else {
            Ok(Response::new(PongResponse {}))
        }
    }

    async fn save_server_share(
        &self,
        request: Request<SaveServerShareRequest>,
    ) -> Result<Response<SaveServerShareResponse>, Status> {
        let request = request.into_inner();
        let public_key = match P::from_bytes(&request.public_key) {
            Ok(pk) => pk,
            Err(_) => return Err(Status::invalid_argument("invalid public key")),
        };
        let server_secret_share = BigInt::from_bytes(&request.server_secret_share);
        if BigInt::zero() <= server_secret_share {
            return Err(Status::invalid_argument("invalid secret share"));
        }
        let server_secret_share = <P::Scalar as ECScalar>::from(&server_secret_share);

        if let Err(e) = self
            .store
            .add_server_secret_share(public_key, server_secret_share)
            .await
        {
            return Err(Status::internal(format!(
                "adding share to persistent store resulted in error: {}",
                e
            )));
        }

        Ok(Response::new(SaveServerShareResponse {}))
    }
}
