/// VerifyServerShare
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct VerifyServerShareRequest {
    #[prost(bytes = "vec", tag = "1")]
    pub public_key: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "2")]
    pub client_public_share: ::prost::alloc::vec::Vec<u8>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct VerifyServerShareResponse {
    #[prost(bytes = "vec", tag = "1")]
    pub server_public_share: ::prost::alloc::vec::Vec<u8>,
}
/// GetChallenge
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetChallengeRequest {}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Challenge {
    #[prost(bytes = "vec", tag = "1")]
    pub id: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "2")]
    pub challenge: ::prost::alloc::vec::Vec<u8>,
}
/// ObtainServerSecretShare
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ObtainServerSecretShareRequest {
    #[prost(bytes = "vec", tag = "1")]
    pub public_key: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "2")]
    pub client_public_share: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "3")]
    pub solved_challenge: ::core::option::Option<Challenge>,
    #[prost(bytes = "vec", tag = "4")]
    pub solution: ::prost::alloc::vec::Vec<u8>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ObtainServerSecretShareResponse {
    #[prost(bytes = "vec", tag = "1")]
    pub server_secret_share: ::prost::alloc::vec::Vec<u8>,
}
#[doc = r" Generated client implementations."]
pub mod beneficiary_api_client {
    #![allow(unused_variables, dead_code, missing_docs)]
    use tonic::codegen::*;
    pub struct BeneficiaryApiClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl BeneficiaryApiClient<tonic::transport::Channel> {
        #[doc = r" Attempt to create a new client by connecting to a given endpoint."]
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: std::convert::TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> BeneficiaryApiClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::ResponseBody: Body + HttpBody + Send + 'static,
        T::Error: Into<StdError>,
        <T::ResponseBody as HttpBody>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_interceptor(inner: T, interceptor: impl Into<tonic::Interceptor>) -> Self {
            let inner = tonic::client::Grpc::with_interceptor(inner, interceptor);
            Self { inner }
        }
        pub async fn verify_server_share(
            &mut self,
            request: impl tonic::IntoRequest<super::VerifyServerShareRequest>,
        ) -> Result<tonic::Response<super::VerifyServerShareResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/beneficiary.BeneficiaryAPI/VerifyServerShare",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn get_challenge(
            &mut self,
            request: impl tonic::IntoRequest<super::GetChallengeRequest>,
        ) -> Result<tonic::Response<super::Challenge>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/beneficiary.BeneficiaryAPI/GetChallenge");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn obtain_server_secret_share(
            &mut self,
            request: impl tonic::IntoRequest<super::ObtainServerSecretShareRequest>,
        ) -> Result<tonic::Response<super::ObtainServerSecretShareResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/beneficiary.BeneficiaryAPI/ObtainServerSecretShare",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
    impl<T: Clone> Clone for BeneficiaryApiClient<T> {
        fn clone(&self) -> Self {
            Self {
                inner: self.inner.clone(),
            }
        }
    }
    impl<T> std::fmt::Debug for BeneficiaryApiClient<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "BeneficiaryApiClient {{ ... }}")
        }
    }
}
