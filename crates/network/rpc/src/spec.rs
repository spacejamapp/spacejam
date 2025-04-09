//! JSON RPC specification.
//!
//! https://www.jsonrpc.org/specification

use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;
use std::borrow::Cow;

/// A rpc call is represented by sending a Request object to a Server.
#[derive(Serialize, Deserialize)]
pub struct Request<'req> {
    /// A String specifying the version of the JSON-RPC protocol. MUST be exactly "2.0".
    pub jsonrpc: Cow<'req, str>,

    /// A String containing the name of the method to be invoked. Method names that begin
    /// with the word rpc followed by a period character (U+002E or ASCII 46) are reserved
    /// for rpc-internal methods and extensions and MUST NOT be used for anything else.
    pub method: Cow<'req, str>,

    /// A Structured value that holds the parameter values to be used during the invocation
    /// of the method. This member MAY be omitted.
    pub params: Option<Cow<'req, RawValue>>,

    /// An identifier established by the Client that MUST contain a String, Number,
    /// or NULL value if included.
    ///
    /// If it is not included it is assumed to be a notification. The value SHOULD
    /// normally not be Null [1] and Numbers SHOULD NOT contain fractional parts [2]
    pub id: Option<Id<'req>>,
}

/// When a rpc call is made, the Server MUST reply with a Response, except for in the case of Notifications.
#[derive(Serialize, Deserialize)]
pub struct Response<'res> {
    /// A String specifying the version of the JSON-RPC protocol. MUST be exactly "2.0".
    pub jsonrpc: Cow<'res, str>,

    /// This member is REQUIRED on success.
    /// This member MUST NOT exist if there was an error invoking the method.
    /// The value of this member is determined by the method invoked on the Server.
    pub result: Option<Cow<'res, RawValue>>,

    ///This member is REQUIRED on error.
    /// This member MUST NOT exist if there was no error triggered during invocation.
    /// The value for this member MUST be an Object as defined in section 5.1.
    pub error: Option<Error>,

    /// This member is REQUIRED.
    /// It MUST be the same as the value of the id member in the Request Object.
    /// If there was an error in detecting the id in the Request object (e.g. Parse error/Invalid Request), it MUST be Null.
    pub id: Option<Id<'res>>,
}

/// When a rpc call encounters an error, the Response Object MUST contain the error.
#[derive(Serialize, Deserialize)]
pub struct Error {
    /// A Number that indicates the error type that occurred.
    /// This MUST be an integer.
    pub code: i32,

    /// A String providing a short description of the error.
    /// The message SHOULD be limited to a concise single sentence.
    pub message: String,

    /// A Primitive or Structured value that contains additional information about the error.
    /// This may be omitted.
    pub data: Option<String>,
}

/// String or Number for the id field.
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum Id<'id> {
    /// A String.
    String(Cow<'id, str>),

    /// A Number.
    Number(usize),
}
