use crate::requests::Response;
use eva::{True, service};

#[service(result = Response)]
pub trait Sys {
    #[args(derive(Default), non_exhaustive)]
    fn healthcheck() -> True;
}
