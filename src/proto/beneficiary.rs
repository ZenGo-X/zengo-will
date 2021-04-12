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
#[doc = r" Generated server implementations."]
pub mod beneficiary_api_server {
    #![allow(unused_variables, dead_code, missing_docs)]
    use tonic::codegen::*;
    #[doc = "Generated trait containing gRPC methods that should be implemented for use with BeneficiaryApiServer."]
    #[async_trait]
    pub trait BeneficiaryApi: Send + Sync + 'static {
        async fn verify_server_share(
            &self,
            request: tonic::Request<super::VerifyServerShareRequest>,
        ) -> Result<tonic::Response<super::VerifyServerShareResponse>, tonic::Status>;
        async fn get_challenge(
            &self,
            request: tonic::Request<super::GetChallengeRequest>,
        ) -> Result<tonic::Response<super::Challenge>, tonic::Status>;
        async fn obtain_server_secret_share(
            &self,
            request: tonic::Request<super::ObtainServerSecretShareRequest>,
        ) -> Result<tonic::Response<super::ObtainServerSecretShareResponse>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct BeneficiaryApiServer<T: BeneficiaryApi> {
        inner: _Inner<T>,
    }
    struct _Inner<T>(Arc<T>, Option<tonic::Interceptor>);
    impl<T: BeneficiaryApi> BeneficiaryApiServer<T> {
        pub fn new(inner: T) -> Self {
            let inner = Arc::new(inner);
            let inner = _Inner(inner, None);
            Self { inner }
        }
        pub fn with_interceptor(inner: T, interceptor: impl Into<tonic::Interceptor>) -> Self {
            let inner = Arc::new(inner);
            let inner = _Inner(inner, Some(interceptor.into()));
            Self { inner }
        }
    }
    impl<T, B> Service<http::Request<B>> for BeneficiaryApiServer<T>
    where
        T: BeneficiaryApi,
        B: HttpBody + Send + Sync + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = Never;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/beneficiary.BeneficiaryAPI/VerifyServerShare" => {
                    #[allow(non_camel_case_types)]
                    struct VerifyServerShareSvc<T: BeneficiaryApi>(pub Arc<T>);
                    impl<T: BeneficiaryApi>
                        tonic::server::UnaryService<super::VerifyServerShareRequest>
                        for VerifyServerShareSvc<T>
                    {
                        type Response = super::VerifyServerShareResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::VerifyServerShareRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).verify_server_share(request).await };
                            Box::pin(fut)
                        }
                    }
                    let inner = self.inner.clone();
                    let fut = async move {
                        let interceptor = inner.1.clone();
                        let inner = inner.0;
                        let method = VerifyServerShareSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = if let Some(interceptor) = interceptor {
                            tonic::server::Grpc::with_interceptor(codec, interceptor)
                        } else {
                            tonic::server::Grpc::new(codec)
                        };
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/beneficiary.BeneficiaryAPI/GetChallenge" => {
                    #[allow(non_camel_case_types)]
                    struct GetChallengeSvc<T: BeneficiaryApi>(pub Arc<T>);
                    impl<T: BeneficiaryApi> tonic::server::UnaryService<super::GetChallengeRequest>
                        for GetChallengeSvc<T>
                    {
                        type Response = super::Challenge;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetChallengeRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).get_challenge(request).await };
                            Box::pin(fut)
                        }
                    }
                    let inner = self.inner.clone();
                    let fut = async move {
                        let interceptor = inner.1.clone();
                        let inner = inner.0;
                        let method = GetChallengeSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = if let Some(interceptor) = interceptor {
                            tonic::server::Grpc::with_interceptor(codec, interceptor)
                        } else {
                            tonic::server::Grpc::new(codec)
                        };
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/beneficiary.BeneficiaryAPI/ObtainServerSecretShare" => {
                    #[allow(non_camel_case_types)]
                    struct ObtainServerSecretShareSvc<T: BeneficiaryApi>(pub Arc<T>);
                    impl<T: BeneficiaryApi>
                        tonic::server::UnaryService<super::ObtainServerSecretShareRequest>
                        for ObtainServerSecretShareSvc<T>
                    {
                        type Response = super::ObtainServerSecretShareResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ObtainServerSecretShareRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut =
                                async move { (*inner).obtain_server_secret_share(request).await };
                            Box::pin(fut)
                        }
                    }
                    let inner = self.inner.clone();
                    let fut = async move {
                        let interceptor = inner.1.clone();
                        let inner = inner.0;
                        let method = ObtainServerSecretShareSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = if let Some(interceptor) = interceptor {
                            tonic::server::Grpc::with_interceptor(codec, interceptor)
                        } else {
                            tonic::server::Grpc::new(codec)
                        };
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => Box::pin(async move {
                    Ok(http::Response::builder()
                        .status(200)
                        .header("grpc-status", "12")
                        .header("content-type", "application/grpc")
                        .body(tonic::body::BoxBody::empty())
                        .unwrap())
                }),
            }
        }
    }
    impl<T: BeneficiaryApi> Clone for BeneficiaryApiServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self { inner }
        }
    }
    impl<T: BeneficiaryApi> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone(), self.1.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: BeneficiaryApi> tonic::transport::NamedService for BeneficiaryApiServer<T> {
        const NAME: &'static str = "beneficiary.BeneficiaryAPI";
    }
}
