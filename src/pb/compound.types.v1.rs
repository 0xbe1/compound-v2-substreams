#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Token {
    #[prost(string, tag = "1")]
    pub id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub name: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub symbol: ::prost::alloc::string::String,
    #[prost(uint64, tag = "4")]
    pub decimals: u64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Tokens {
    #[prost(message, repeated, tag = "1")]
    pub tokens: ::prost::alloc::vec::Vec<Token>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Market {
    #[prost(string, tag = "1")]
    pub id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub name: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "3")]
    pub input_token: ::core::option::Option<Token>,
    #[prost(message, optional, tag = "4")]
    pub output_token: ::core::option::Option<Token>,
}
