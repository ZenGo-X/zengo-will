/// Ping-Pong
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PingRequest {}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PongResponse {}
/// SaveServerShare
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SaveServerShareRequest {
    #[prost(bytes = "vec", tag = "1")]
    pub public_key: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "2")]
    pub server_secret_share: ::prost::alloc::vec::Vec<u8>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SaveServerShareResponse {}
#[doc = r" Generated client implementations."]
pub mod testator_api_client {
    #![allow(unused_variables, dead_code, missing_docs)]
    use tonic::codegen::*;
    pub struct TestatorApiClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl TestatorApiClient<tonic::transport::Channel> {
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
    impl<T> TestatorApiClient<T>
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
        pub async fn ping(
            &mut self,
            request: impl tonic::IntoRequest<super::PingRequest>,
        ) -> Result<tonic::Response<super::PongResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/testator.TestatorAPI/Ping");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn save_server_share(
            &mut self,
            request: impl tonic::IntoRequest<super::SaveServerShareRequest>,
        ) -> Result<tonic::Response<super::SaveServerShareResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/testator.TestatorAPI/SaveServerShare");
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
    impl<T: Clone> Clone for TestatorApiClient<T> {
        fn clone(&self) -> Self {
            Self {
                inner: self.inner.clone(),
            }
        }
    }
    impl<T> std::fmt::Debug for TestatorApiClient<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "TestatorApiClient {{ ... }}")
        }
    }
}
